use anyhow::{Context, anyhow};
use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse, routing::get, Extension};
use serde::{Deserialize, Serialize};
use sqlx::Transaction;
use uuid::Uuid;
use super::{
    examiners::Examiner,
    candidates::{Candidate, CandidatesBySlot, create_fill},
    stations::Station,
    slots::{Slot, SlotTime},
    circuits::Circuit,
    users::AccessClaims,
    AppState, SomethingID};
use crate::{
    error::AppError,
    allocation::{allocate_stations, StationAllocation},
};
use tracing::trace;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/generate", get(gen_new))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Allocation {
    pub id: Uuid,
    pub slot_id: Uuid,
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub modified_at: time::OffsetDateTime
}

pub struct AllocationPayload {
    pub slot_id: Uuid,
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllocationHistory {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub slot_id: Uuid,
    pub circuit_id: Uuid,
    pub station_id: Uuid,
    pub candidate_1: Uuid,
    pub candidate_2: Uuid,
    pub examiner: Uuid,
    pub modified_by: Uuid,
    pub auto_gen: bool,
    #[serde(with = "time::serde::iso8601")]
    pub modified_at: time::OffsetDateTime
}


async fn gen_new(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(req): Json<SomethingID> // session id
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    
    let am_slot = Slot::get_by_session(&pool, &req.id, SlotTime::AM).await?;
    let pm_slot = Slot::get_by_session(&pool, &req.id, SlotTime::PM).await?;

    let am_circuit = Circuit::get_by_slot(&pool, &am_slot.id).await?;
    let pm_circuit = Circuit::get_by_slot(&pool, &pm_slot.id).await?;

    let stations = Station::get_by_session(&pool, &req.id).await?;

    let candidates_result = sqlx::query_as!(
        Candidate,
        r#"
        SELECT * FROM people.candidates WHERE session_id = $1 ORDER BY slot
        "#,
        &req.id
    )
    .fetch_all(&pool)
    .await
    .with_context(|| format!("Cannot get all candidates with session_id: {}", req.id))?;

    let mut candidates_no_slot: Vec<Candidate> = Vec::new();
    let mut candidates_by_slot: Vec<CandidatesBySlot> = Vec::new();
    let mut current_slot: Option<String> = None;
    for candidate in candidates_result {
        if current_slot != candidate.slot {
            if let Some(can_slot) = candidate.slot.clone() {
                candidates_by_slot.push(CandidatesBySlot {
                    slot: can_slot,
                    candidates: Vec::new(),
                });
            } else {
                candidates_no_slot.push(candidate.clone());
                continue;
            }
            current_slot = candidate.slot.clone();
        }
        candidates_by_slot.last_mut().unwrap().candidates.push(candidate);
    }

    let mut am_candidates = candidates_by_slot
        .iter_mut()
        .find(|s| s.slot == "AM")
        .map(|s| std::mem::take(&mut s.candidates))
        .unwrap_or_default();
    let mut pm_candidates = candidates_by_slot
        .iter_mut()
        .find(|s| s.slot == "PM")
        .map(|s| std::mem::take(&mut s.candidates))
        .unwrap_or_default();

    let no_slot_count = candidates_no_slot.len();
    let am_count = am_candidates.len();
    let pm_count = pm_candidates.len();
    let total_count = no_slot_count + am_count + pm_count;
    
    trace!("Candidates: no slot = {}, AM = {}, PM = {}. Total = {}", no_slot_count, am_count, pm_count, total_count);

    if total_count % 2 != 0 {
        candidates_no_slot.push(create_fill(req.id, &pool).await?);
    }

    if no_slot_count % 2 != 0 {
        candidates_no_slot.push(create_fill(req.id, &pool).await?);
    }
    if am_count % 2 != 0 {
        if !candidates_no_slot.is_empty() {
            let mut candidate = candidates_no_slot.remove(0);
            candidate.slot = Some("AM".to_string());
            am_candidates.push(candidate);
        } else {
            am_candidates.push(create_fill(req.id, &pool).await?);
        }
    }
    if pm_count % 2 != 0 {
        if !candidates_no_slot.is_empty() {
            let mut candidate = candidates_no_slot.remove(0);
            candidate.slot = Some("PM".to_string());
            pm_candidates.push(candidate);
        } else {
            pm_candidates.push(create_fill(req.id, &pool).await?);
        }
    }
    
    let am_count = am_candidates.len();
    let pm_count = pm_candidates.len();

    trace!("After 2 checks; no slot = {}, AM = {}, PM = {}. Total = {}", no_slot_count, am_count, pm_count, total_count);

    if am_count % 2 == 0 && pm_count % 2 == 0 && !candidates_no_slot.is_empty() {
        let target_count = (am_count + pm_count + candidates_no_slot.len()) / 2;
        while am_count < target_count && !candidates_no_slot.is_empty() {
            let mut candidate = candidates_no_slot.remove(0);
            candidate.slot = Some("AM".to_string());
            am_candidates.push(candidate);
        }
        while pm_count < target_count && !candidates_no_slot.is_empty() {
            let mut candidate = candidates_no_slot.remove(0);
            candidate.slot = Some("PM".to_string());
            pm_candidates.push(candidate);
        }
    }

    let am_count = am_candidates.len();
    let pm_count = pm_candidates.len();
    let no_slot_count = candidates_no_slot.len();
    let total_count = no_slot_count + am_count + pm_count;
    trace!("FINAL; no slot = {}, AM = {}, PM = {}. Total = {}", no_slot_count, am_count, pm_count, total_count);

    let am_examiners = Examiner::get_all_by_session_slot(&pool, req.id, SlotTime::AM).await?;
    let pm_examiners = Examiner::get_all_by_session_slot(&pool, req.id, SlotTime::PM).await?;
    
    let am_allocations = allocate_stations(&am_circuit, &stations, &am_candidates, &am_examiners)?;
    let pm_allocations = allocate_stations(&pm_circuit, &stations, &pm_candidates, &pm_examiners)?;

    let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;
    let batch_id = Uuid::new_v4();
    
    Allocation::add_by_slot(&mut transaction, am_allocations, &batch_id, &am_slot.id, &claim.id).await?;
    Allocation::add_by_slot(&mut transaction, pm_allocations, &batch_id, &pm_slot.id, &claim.id).await?;

    transaction.commit().await.map_err(|e| AppError::from(anyhow!("Failed to commit transaction: {}", e)))?;

    Ok(StatusCode::OK.into_response())
}

impl Allocation {
    pub async fn add_by_slot(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        allocations: Vec<StationAllocation>,
        batch_id: &Uuid,
        slot_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> { // for 1 SLOT at a time
        sqlx::query!(
            r#"
            DELETE FROM records.allocations
            WHERE slot_id = $1
            "#,
            slot_id
        )
        .execute(&mut **tx)
        .await
        .with_context(|| "Failed to delete existing allocations")?;

        for allocation in &allocations {
            sqlx::query!(
                r#"
                INSERT INTO records.allocations (
                    slot_id, circuit_id, station_id, candidate_1, candidate_2, examiner
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                slot_id,
                allocation.circuit_id,
                allocation.station_id,
                allocation.candidate_1,
                allocation.candidate_2,
                allocation.examiner
            )
            .execute(&mut **tx)
            .await
            .with_context(|| "Failed to insert allocation")?;

            sqlx::query!(
                r#"
                INSERT INTO records.allocations_history (
                    batch_id, slot_id, circuit_id, station_id, candidate_1, candidate_2, examiner, modified_by, auto_gen
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                batch_id,
                slot_id,
                allocation.circuit_id,
                allocation.station_id,
                allocation.candidate_1,
                allocation.candidate_2,
                allocation.examiner,
                user_id,
                true
            )
            .execute(&mut **tx)
            .await
            .with_context(|| "Failed to insert allocation history")?;
        }
        Ok(())
    }
}
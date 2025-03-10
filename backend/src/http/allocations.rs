use anyhow::{Context, anyhow};
use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse, routing::get, Extension};
use serde::{Deserialize, Serialize};
use sqlx::Transaction;
use uuid::Uuid;
use super::{
    candidates::{create_fill, Candidate}, circuits::Circuit, examiners::Examiner, runs::{Run, RunTime}, slots::Slot, stations::Station, users::AccessClaims, AppState, SomethingID};
use crate::{
    allocation_algo::{allocate_by_slot, allocate_by_time, SlotAllocation, TimeAllocation}, error::AppError
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

#[derive(Debug, Serialize)]
pub struct Availability {
    pub am: bool,
    pub pm: bool,
}

async fn gen_new(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(req): Json<SomethingID> // session id
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }

    let slots = Slot::get_all_by_session(&pool, &req.id).await?;
    let stations = Station::get_by_session(&pool, &req.id).await?;
    let mut candidates_all = Candidate::get_ava_all(&pool, &req.id, Availability { am: true, pm: true }).await?;

    let batch_id = Uuid::new_v4();
    let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

    match slots.len() {
        1 => { // as only 1 slot, check if any runs are in AM or PM, as all candidates need to be there for its full duration
            let cur_slot = &slots[0];
            let circuits = Circuit::get_by_slot(&pool, &cur_slot.id).await?;
            let am_runs = Run::get_by_time(&pool, &cur_slot.id, RunTime::AM).await?;
            let pm_runs = Run::get_by_time(&pool, &cur_slot.id, RunTime::PM).await?;

            let am_runs_count = am_runs.len();
            let pm_runs_count = pm_runs.len();

            if candidates_all.len() % 2 != 0 {
                candidates_all.push(create_fill(req.id.clone(), &pool, None).await?);
            }

            // EXAMINER ALLOCATIONS
            let mut examiners = Vec::new();
            if am_runs_count > 0 && pm_runs_count == 0 { // Slot is AM runs only
                examiners = Examiner::get_all_by_time(&pool, &req.id, RunTime::AM).await?;
            } else if am_runs_count == 0 && pm_runs_count > 0 { // Slot is PM runs only
                examiners = Examiner::get_all_by_time(&pool, &req.id, RunTime::PM).await?;
            } else if am_runs_count > 0 && pm_runs_count > 0 { // Slot has runs in both AM and PM
                let am_examiners = Examiner::get_all_by_time(&pool, &req.id, RunTime::AM).await?;
                let pm_examiners = Examiner::get_all_by_time(&pool, &req.id, RunTime::PM).await?;
                let am_allocations = allocate_by_time(&circuits, &stations, &am_examiners)?;
                let pm_allocations = allocate_by_time(&circuits, &stations, &pm_examiners)?;

                Allocation::add_by_time(&mut transaction, am_allocations, &batch_id, &cur_slot.id, &claim.id).await?;
                Allocation::add_by_time(&mut transaction, pm_allocations, &batch_id, &cur_slot.id, &claim.id).await?;
                transaction.commit().await.map_err(|e| AppError::from(anyhow!("Failed to commit transaction: {}", e)))?;
                return Ok(StatusCode::OK.into_response());
            } else {
                return Err(AppError::from(anyhow!("Slot has no runs")));
            }
            let allocations = allocate_by_time(&circuits, &stations, &examiners)?;
            
            Allocation::add_by_time(&mut transaction, allocations, &batch_id, &cur_slot.id, &claim.id).await?;
            transaction.commit().await.map_err(|e| AppError::from(anyhow!("Failed to commit transaction: {}", e)))?;        
        },
        2 => {
            let candidates_am = Candidate::get_ava_all(&pool, &req.id, Availability { am: true, pm: false }).await?;
            let candidates_pm = Candidate::get_ava_all(&pool, &req.id, Availability { am: false, pm: true }).await?;
            
            let can_all_count = candidates_all.len();
            let can_am_count = candidates_am.len();
            let can_pm_count = candidates_pm.len();
            let total_candidates = can_all_count + can_am_count + can_pm_count;
            
            trace!("Candidates: Both = {}, AM = {}, PM = {}. TOTAL: {}", can_all_count, can_am_count, can_pm_count, total_candidates);
        
            // make even balance across slots
            if total_candidates % 2 != 0 { // either 1 odd or ALL odd
                if can_all_count % 2 != 0 {
                    if can_am_count % 2 != 0 {
                        // all odds
                    } else {
                        // only all_count is odd
                    }
                } else {
                    if can_pm_count % 2 != 0 {
                        // only pm_count is odd
                    } else {
                        // only am_count is odd
                    }
                }
            } else { // either 2 odds or ALL even
                if can_all_count % 2 != 0 {
                    if can_am_count % 2 != 0 {
                        // all_count, am_count = odd
                    } else {
                        // all_count, pm_count = odd
                    }
                } else {
                    if can_pm_count % 2 != 0 {
                        // am_count, pm_count = odd
                    } else {
                        // all even
                    }
                }
            }
        },
        _ => {
            // find slot where ALL runs are in AM, same for PM
        },
    };

    Ok(StatusCode::OK.into_response())
}

impl Allocation {
    pub async fn add_by_slot(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        allocations: Vec<SlotAllocation>,
        batch_id: &Uuid,
        slot_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> { // for candidates ONLY
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
                    slot_id, circuit_id, station_id, candidate_1, candidate_2
                ) VALUES ($1, $2, $3, $4, $5)
                "#,
                slot_id,
                allocation.circuit_id,
                allocation.station_id,
                allocation.candidate_1,
                allocation.candidate_2,
            )
            .execute(&mut **tx)
            .await
            .with_context(|| "Failed to insert allocation")?;

            sqlx::query!(
                r#"
                INSERT INTO records.allocations_history (
                    batch_id, slot_id, circuit_id, station_id, candidate_1, candidate_2, modified_by, auto_gen
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                batch_id,
                slot_id,
                allocation.circuit_id,
                allocation.station_id,
                allocation.candidate_1,
                allocation.candidate_2,
                user_id,
                true
            )
            .execute(&mut **tx)
            .await
            .with_context(|| "Failed to insert allocation history")?;
        }
        Ok(())
    }

    pub async fn add_by_time(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        allocations: Vec<TimeAllocation>,
        batch_id: &Uuid,
        slot_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> { // for examiners ONLY
        Ok(())
    }
}
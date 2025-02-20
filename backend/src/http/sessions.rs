use anyhow::{Context, anyhow};
use axum::{extract::{Json, State}, response::IntoResponse, routing::{get, post}, Extension, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::AccessClaims, AppState};
use crate::error::AppError;
use sqlx::postgres::types::PgInterval;
use tracing::{instrument, trace, warn};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/test", get(test_function))
        .route("/create", post(Session::create))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionPayload {
    pub session: SessionPayload,
    pub stations: Vec<StationPayload>,
    pub slots: Vec<SlotPayload> // runs and circuits inside slots
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionPayload {
    pub organisation: String,
    pub scheduled_date: time::Date,
    pub location: String,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StationPayload {
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlotPayload {
    pub slot_time: String,
    pub runs: Vec<RunPayload>,
    pub circuits: Vec<CircuitPayload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunPayload {
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_start: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_end: time::OffsetDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitPayload {
    pub key: String, // A-Z
    pub female_only: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub organiser_id: Uuid,
    pub organisation: String,
    pub scheduled_date: time::Date,
    pub location: String,
    pub total_stations: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: Uuid,
    pub session_id: Uuid,
    pub slot_time: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub slot_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_start: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_end: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_start: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_end: Option<time::OffsetDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Circuit {
    pub id: Uuid,
    pub session_id: Uuid,
    pub slot_id: Uuid,
    pub key: String,
    pub female_only: bool,
    pub current_rotation: Option<i16>,
    pub status: String,
    pub intermission: bool,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_start: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_end: Option<time::OffsetDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Station {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}

async fn test_function(
    Extension(user): Extension<AccessClaims>
) -> impl IntoResponse {
    trace!("Successful test function, user: {:?}", user);
    "This worked".into_response()
}

impl Session {
    #[instrument(name = "create_session", level = "TRACE")]
    pub async fn create(
        State(pool): State<sqlx::PgPool>,
        Extension(user): Extension<AccessClaims>,
        Json(req): Json<CreateSessionPayload>,
    ) -> Result<impl IntoResponse, AppError> {
        let session_payload = req.session;
        let total_stations = req.stations.len() as i16;

        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        let session_result = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO records.sessions (organiser_id, organisation, scheduled_date, location, total_stations, intermission_duration, static_at_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            &user.id,
            session_payload.organisation,
            session_payload.scheduled_date,
            session_payload.location,
            &total_stations,
            session_payload.intermission_duration,
            session_payload.static_at_end)
            .fetch_one(&mut *transaction)
            .await;

        
        if let Err(e) = session_result {
            transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding session. Failed transaction: {}", e))?;
            return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding session: {}", e)));
        }

        let session_result = session_result.unwrap();
        
        for station in &req.stations {
            let station_result = sqlx::query_as!(
                Station,
                r#"
                INSERT INTO records.stations (session_id, title, index, duration)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#,
                &session_result.id,
                station.title,
                station.index,
                station.duration)
                .fetch_one(&mut *transaction)
                .await;

            if let Err(e) = station_result {
                transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding station. Failed transaction: {}", e))?;
                return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding station: {}", e)));
            }
        }

        for slot in &req.slots {
            let slot_result = sqlx::query_as!(
                Slot,
                r#"
                INSERT INTO records.slots (session_id, slot_time)
                VALUES ($1, $2)
                RETURNING *
                "#,
                &session_result.id,
                slot.slot_time)
                .fetch_one(&mut *transaction)
                .await;
            if let Err(e) = slot_result {
                transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding slot. Failed transaction: {}", e))?;
                return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding slot: {}", e)));
            }
            let slot_result = slot_result.unwrap();
            for run in &slot.runs {
                let run_result = sqlx::query_as!(
                    Run,
                    r#"
                    INSERT INTO records.runs (slot_id, scheduled_start, scheduled_end)
                    VALUES ($1, $2, $3)
                    RETURNING *
                    "#,
                    &slot_result.id,
                    run.scheduled_start,
                    run.scheduled_end)
                    .fetch_one(&mut *transaction)
                    .await;
    
                if let Err(e) = run_result {
                    transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding run. Failed transaction: {}", e))?;
                    return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding run: {}", e)));
                }
            }
            for circuit in &slot.circuits {
                let circuit_result = sqlx::query_as!(
                    Circuit,
                    r#"
                    INSERT INTO records.circuits (session_id, slot_id, key, female_only, intermission)
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING *
                    "#,
                    &session_result.id,
                    &slot_result.id,
                    circuit.key,
                    circuit.female_only,
                    false)
                    .fetch_one(&mut *transaction)
                    .await;
    
                if let Err(e) = circuit_result {
                    transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding circuit. Failed transaction: {}", e))?;
                    return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding circuit: {}", e)));
                }
            }
        }

        transaction.commit().await.with_context(|| format!("Rolled back successful. Transaction failed to commit"))?;
        
        Ok((StatusCode::CREATED, Json(session_result)))
    }
}
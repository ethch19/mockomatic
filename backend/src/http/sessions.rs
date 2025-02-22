use anyhow::{Context, anyhow};
use axum::{extract::{Json, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::AccessClaims, AppState, SomethingID, SomethingMultipleID};
use crate::error::AppError;
use sqlx::postgres::types::PgInterval;
use tracing::{instrument, trace};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/test", get(test_function))
        .route("/create", post(Session::create))
        .route("/update", post(Session::update))
        .route("/delete", post(Session::delete))
        .route("/get", post(Session::get))
        .route("/get-page", get(Session::get_page))
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

#[derive(Debug, Deserialize)]
pub struct SessionChange {
    pub id: Uuid,
    pub organiser_id: Uuid,
    pub organisation: Option<String>,
    pub scheduled_date: Option<time::Date>,
    pub location: Option<String>,
    pub total_stations: Option<i16>,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub intermission_duration: Option<PgInterval>,
    pub static_at_end: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub first: i64, // Offset (starting position)
    pub rows: i64, // Limit (number of rows per page)
    #[serde(rename = "sortField")]
    pub sort_field: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>, // 1 for ascending, -1 for descending
}

#[derive(Debug, Serialize)]
struct PaginationResponse {
    sessions: Vec<Session>,
    total: i64,
}

async fn test_function(
    Extension(user): Extension<AccessClaims>
) -> impl IntoResponse {
    trace!("Successful test function, user: {:?}", user);
    "This worked".into_response()
}

impl Session {
    #[instrument(name = "create_session", level = "TRACE", skip(user))]
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

    pub async fn get(
        State(pool): State<sqlx::PgPool>,
        Json(session): Json<SomethingID>,
    ) -> Result<impl IntoResponse, AppError> {
        let result = sqlx::query_as!(
            Session,
            r#"
            SELECT * FROM records.sessions WHERE id = $1
            "#,
            session.id
        )
        .fetch_one(&pool)
        .await
        .with_context(|| format!("Cannot get session with specific id"))?;

        Ok((StatusCode::OK, Json(result)).into_response())
    }

    #[instrument(name = "get_page", level = "TRACE")]
    pub async fn get_page(
        State(pool): State<sqlx::PgPool>,
        Query(params): Query<PaginationParams>
    ) -> Result<impl IntoResponse, AppError> {
        let first = params.first.max(0);
        let rows = params.rows.clamp(1, 100);

        let sort_field = params.sort_field.unwrap_or_else(|| "scheduled_date".to_string());
        let sort_order = params.sort_order.unwrap_or(-1);
        let is_ascending = sort_order == 1;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM records.sessions")
            .fetch_one(&pool)
            .await
            .with_context(|| format!("Cannot get sessions count"))?;

        let sessions = match (sort_field.as_str(), is_ascending) {
            ("created_at", true) => sqlx::query_as!(
                Session,
                r#"
                SELECT *
                FROM records.sessions
                ORDER BY created_at ASC
                OFFSET $1 LIMIT $2
                "#,
                first,
                rows
            )
            .fetch_all(&pool)
            .await
            .with_context(|| format!("Cannot get sessions page"))?,
    
            ("created_at", false) => sqlx::query_as!(
                Session,
                r#"
                SELECT *
                FROM records.sessions
                ORDER BY created_at DESC
                OFFSET $1 LIMIT $2
                "#,
                first,
                rows
            )
            .fetch_all(&pool)
            .await
            .with_context(|| format!("Cannot get sessions page"))?,
    
            // Default to scheduled_date (both true and false cases)
            (_, true) => sqlx::query_as!(
                Session,
                r#"
                SELECT *
                FROM records.sessions
                ORDER BY scheduled_date ASC
                OFFSET $1 LIMIT $2
                "#,
                first,
                rows
            )
            .fetch_all(&pool)
            .await
            .with_context(|| format!("Cannot get sessions page"))?,
    
            (_, false) => sqlx::query_as!(
                Session,
                r#"
                SELECT *
                FROM records.sessions
                ORDER BY scheduled_date DESC
                OFFSET $1 LIMIT $2
                "#,
                first,
                rows
            )
            .fetch_all(&pool)
            .await
            .with_context(|| format!("Cannot get sessions page"))?,
        };

        Ok((StatusCode::OK, Json(PaginationResponse { sessions, total })).into_response())
    }

    pub async fn update(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(session): Json<SessionChange>,
    ) -> Result<impl IntoResponse, AppError> {
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        let _ = sqlx::query!(
            r#"
            UPDATE records.sessions
            SET
                organisation = COALESCE($3, organisation),
                scheduled_date = COALESCE($4, scheduled_date),
                location = COALESCE($5, location),
                intermission_duration = COALESCE($6, intermission_duration),
                static_at_end = COALESCE($7, static_at_end)
            WHERE id = $1 AND organiser_id = $2
            "#,
            session.id,
            session.organiser_id,
            session.organisation,
            session.scheduled_date,
            session.location,
            session.intermission_duration,
            session.static_at_end
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot update session: {}", session.id))?;

        Ok(StatusCode::OK.into_response())
    }

    pub async fn delete(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(session): Json<SomethingMultipleID>,
    ) -> Result<impl IntoResponse, AppError> {
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }

        for session_id in &session.ids {
            let _ = sqlx::query!(
                r#"
                DELETE FROM records.sessions
                WHERE id = $1
                "#,
                session_id
            )
            .execute(&pool)
            .await
            .with_context(|| format!("Cannot delete session with ID: {}", session_id))?;
        }

        Ok(StatusCode::OK.into_response())
    }
}
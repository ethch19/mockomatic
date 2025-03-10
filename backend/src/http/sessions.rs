use anyhow::{Context, anyhow};
use axum::{extract::{Json, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::AccessClaims, AppState, SomethingID, SomethingMultipleID, circuits::Circuit, runs::Run, slots::{Slot, SlotPayload}, stations::{Station, StationPayload}};
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
pub struct Session {
    pub id: Uuid,
    pub organiser_id: Uuid,
    pub organisation: String,
    pub scheduled_date: time::Date,
    pub location: String,
    pub total_stations: i16,
    pub feedback: bool,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionPayload {
    pub organisation: String,
    pub scheduled_date: time::Date,
    pub location: String,
    pub feedback: bool,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool
}

#[derive(Debug, Deserialize)]
pub struct SessionChange {
    pub id: Uuid,
    pub organiser_id: Uuid,
    pub organisation: Option<String>,
    pub scheduled_date: Option<time::Date>,
    pub location: Option<String>,
    pub feedback: Option<bool>,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    pub total_stations: Option<i16>,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub intermission_duration: Option<PgInterval>,
    pub static_at_end: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionPayload {
    pub session: SessionPayload,
    pub stations: Vec<StationPayload>,
    pub slots: Vec<SlotPayload> // runs and circuits inside slots
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
    #[instrument(name = "create_session", level = "TRACE", skip(claim))]
    pub async fn create(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(req): Json<CreateSessionPayload>,
    ) -> Result<impl IntoResponse, AppError> {
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }

        let session_payload = req.session;
        let total_stations = req.stations.len() as i16;

        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        let mut runtime_sec: i64 = (session_payload.intermission_duration.microseconds / 1000000) * total_stations as i64; //division of microseconds trunucates forward
        if session_payload.feedback {
            if let Some(x) = &session_payload.feedback_duration {
                runtime_sec +=  (x.microseconds / 1000000) * total_stations as i64;
            } else {
                return Err(AppError::from(anyhow!("Feedback set to true but feedback duration missing")));
            }
        } else {
            if session_payload.feedback_duration.is_some() {
                return Err(AppError::from(anyhow!("Feedback duration is given but feedback is set to false")));
            }
        }
        if let Some(sta_duration) = req.stations.first() {
            for i in 0..req.stations.len() {
                if i == req.stations.len() - 1 && session_payload.static_at_end {
                    break;
                }
                if req.stations[i].duration != sta_duration.duration {
                    return Err(AppError::from(anyhow!("Stations have different durations")));
                }
            }
            runtime_sec += (sta_duration.duration.microseconds / 1000000) * (total_stations as i64 - 1);
            if let Some(last_station) = req.stations.last() {
                runtime_sec += last_station.duration.microseconds / 1000000
            } else {
                return Err(AppError::from(anyhow!("Failed to add the duration of the last station")));
            }
        } else {
            return Err(AppError::from(anyhow!("Failed to get the first station")));
        }

        let runtime_dur = time::Duration::new(runtime_sec, 0);
        trace!("Total runtime for 1 slot is {}", runtime_sec);

        let session_result = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO records.sessions (organiser_id, organisation, scheduled_date, location, total_stations, feedback, feedback_duration, intermission_duration, static_at_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            &claim.id,
            session_payload.organisation,
            session_payload.scheduled_date,
            session_payload.location,
            &total_stations,
            session_payload.feedback,
            session_payload.feedback_duration,
            session_payload.intermission_duration,
            session_payload.static_at_end)
            .fetch_one(&mut *transaction)
            .await
            .with_context(|| format!("Failed to create session from transaction"))?;
        
        for station in &req.stations {
            Station::create_tx(&mut transaction, &session_result.id, station).await?;
        }

        for slot in &req.slots {
            let slot_result = Slot::create_tx(&mut transaction, &session_result.id, slot).await?;

            for run in &slot.runs {
                if let Some(run_endtime) = run.scheduled_start.checked_add(runtime_dur) {
                    Run::create_tx(&mut transaction, &slot_result.id, run, run_endtime).await?;
                } else {
                    return Err(AppError::from(anyhow!("Failed to create run's scheduled end")));
                }
            }
            for circuit in &slot.circuits {
                Circuit::create_tx(&mut transaction, &session_result.id, &slot_result.id, circuit).await?;
            }
        }

        transaction.commit().await.with_context(|| format!("Rolled back successful. Transaction failed to commit"))?;
        
        Ok((StatusCode::CREATED, Json(session_result)).into_response())
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

        if session.feedback == Some(true) {
            if session.feedback_duration.is_none() {
                return Err(AppError::from(anyhow!("Feedback set to true but feedback duration missing")));
            }
        }
        if session.feedback_duration.is_some() {
            if session.feedback == Some(false) {
                return Err(AppError::from(anyhow!("Feedback duration is given but feedback is set to false")));
            }
        }

        let _ = sqlx::query!(
            r#"
            UPDATE records.sessions
            SET
                organisation = COALESCE($3, organisation),
                scheduled_date = COALESCE($4, scheduled_date),
                location = COALESCE($5, location),
                feedback = COALESCE($6, feedback),
                feedback_duration = COALESCE($7, feedback_duration),
                intermission_duration = COALESCE($8, intermission_duration),
                static_at_end = COALESCE($9, static_at_end)
            WHERE id = $1 AND organiser_id = $2
            "#,
            session.id,
            session.organiser_id,
            session.organisation,
            session.scheduled_date,
            session.location,
            session.feedback,
            session.feedback_duration,
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
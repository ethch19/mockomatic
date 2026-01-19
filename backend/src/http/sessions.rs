use anyhow::{Context, anyhow};
use axum::{extract::{Json, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::{AccessClaims, User}, AppState, SomethingID, SomethingMultipleID, circuits::Circuit, runs::Run, slots::{Slot, SlotPayload}, stations::{Station, StationPayload}};
use crate::error::AppError;
use sqlx::postgres::types::PgInterval;
use tracing::{instrument, trace};
use std::ops::{Add, Sub, AddAssign, SubAssign, Mul};
use std::convert::From;
use validator::Validate;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/test", get(test_function))
        .route("/create", post(Session::create))
        .route("/update", post(Session::update))
        .route("/delete", post(Session::delete))
        .route("/get", get(Session::get))
        .route("/get-page", get(Session::get_page))
        .route("/get-all", get(Session::get_all))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub organiser_id: Uuid,
    pub organisation_id: Uuid,
    pub scheduled_date: time::Date,
    pub location: String,
    pub total_stations: i16,
    pub feedback: bool,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
    pub status: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionPayload {
    pub scheduled_date: time::Date,
    pub location: String,
    pub feedback: bool,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
    // organiser_id and organisation_id are taken from the token claims
    // status default to 'new'
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSessionPayload {
    pub session: SessionPayload,
    #[validate(length(min = 1, max = 256, message = "Must have between 1 and 256 stations"))]
    pub stations: Vec<StationPayload>,
    #[validate(length(min = 1, max = 26, message = "Must have between 1 and 26 slots"), nested)]
    pub slots: Vec<SlotPayload> // runs and circuits inside slots
}

#[derive(Debug, Deserialize)]
pub struct SessionChange {
    pub id: Uuid,
    pub organiser_id: Uuid,
    pub organisation_id: Uuid,
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

#[derive(Debug)]
pub struct PgIntervalWrapper(PgInterval); //tuple struct wrapper

impl Add for PgIntervalWrapper {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self (PgInterval {
            months: self.0.months + other.0.months,
            days: self.0.days + other.0.days,
            microseconds: self.0.microseconds + other.0.microseconds,
        })
    }
}

impl AddAssign for PgIntervalWrapper {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            0: PgInterval {
                months: self.0.months + other.0.months,
                days: self.0.days + other.0.days,
                microseconds: self.0.microseconds + other.0.microseconds,
            }
        };
    }
}

impl Sub for PgIntervalWrapper {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self (PgInterval {
            months: self.0.months - other.0.months,
            days: self.0.days - other.0.days,
            microseconds: self.0.microseconds - other.0.microseconds,
        })
    }
}

impl SubAssign for PgIntervalWrapper {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            0: PgInterval {
                months: self.0.months - other.0.months,
                days: self.0.days - other.0.days,
                microseconds: self.0.microseconds - other.0.microseconds,
            }
        };
    }
}

impl Mul<i16> for PgIntervalWrapper {
    type Output = Self;

    fn mul(self, rhs: i16) -> Self {
        Self(PgInterval {
            months: self.0.months * rhs as i32,
            days: self.0.days * rhs as i32,
            microseconds: self.0.microseconds * rhs as i64,
        })
    }
}

impl From<PgInterval> for PgIntervalWrapper {
    fn from(interval: PgInterval) -> Self {
        PgIntervalWrapper(interval)
    }
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
        if !User::is_admin(&pool, &claim.id).await? {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }

        req.validate().map_err(|e| AppError::from(anyhow!("Invalid payload: {}", e)))?;

        let session_payload = req.session;
        let total_stations = req.stations.len() as i16;

        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        let mut runtime_duration = PgIntervalWrapper::from(session_payload.intermission_duration) * total_stations;
        if session_payload.feedback {
            if let Some(x) = session_payload.feedback_duration {
                runtime_duration += PgIntervalWrapper::from(x) * total_stations;
            } else {
                return Err(AppError::from(anyhow!("Feedback set to true but feedback duration missing")));
            }
        } else {
            if session_payload.feedback_duration.is_some() {
                return Err(AppError::from(anyhow!("Feedback duration is given but feedback is set to false")));
            }
        }

        // REFACTOR: make static at end calculated in backend, not passed from frontend
        if let Some(st_duration) = req.stations.first() {
            for i in 0..req.stations.len() { // station duration checker
                if i == req.stations.len() - 1 && session_payload.static_at_end { // check if the last station is different only if static at end is true
                    break;
                }
                if req.stations[i].duration != st_duration.duration { // check all stations have the same duration
                    return Err(AppError::from(anyhow!("Stations have different durations. Try turning on static at end.")));
                }
            }
            runtime_duration += PgIntervalWrapper::from(st_duration.duration) * (total_stations - 1);
            if let Some(last_station) = req.stations.last() {
                runtime_duration += PgIntervalWrapper::from(last_station.duration); // incase static at end is true
            } else {
                return Err(AppError::from(anyhow!("No stations have been provided")));
            }
        } else {
            return Err(AppError::from(anyhow!("No stations have been provided")));
        }

        trace!("Total runtime for 1x run is {:?}", runtime_duration);

        let session_result = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO records.sessions (organiser_id, organisation_id, scheduled_date, location, total_stations, feedback, feedback_duration, intermission_duration, static_at_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            &claim.id,
            &claim.organisation_id,
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

        let slot_keys: &[char] = &('A'..='Z').collect::<Vec<char>>()[..req.slots.len()];
        trace!("Slot keys generated: {:?}", slot_keys);
        for (slot, key) in req.slots.iter().zip(slot_keys) {
            let slot_result = Slot::create_tx(&mut transaction, &session_result.id, key.to_string()).await?;

            // REFACTOR: CHECK WHETHER RUNS HAVE THE CORRECT START + END TIME, WHETHER IT OVERLAPS
            for run in &slot.runs {
                Run::create_tx(&mut transaction, &slot_result.id, run, &runtime_duration.0).await?;
            }

            let circuit_keys: &[char] = &('A'..='Z').collect::<Vec<char>>()[..slot.circuits.len()];
            for (circuit, key) in slot.circuits.iter().zip(circuit_keys) {
                Circuit::create_tx(&mut transaction, &session_result.id, &slot_result.id, circuit, key.to_string()).await?;
            }
        }

        transaction.commit().await.with_context(|| format!("Transaction failed to commit. Rolled back successful."))?;
        
        Ok((StatusCode::CREATED, Json(session_result)).into_response())
    }

    pub async fn get(
        State(pool): State<sqlx::PgPool>,
        Query(session): Query<SomethingID>,
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

    // server side pagination, no longer used
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

    pub async fn get_all( // users can only get sessions in their organisation
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
    ) -> Result<impl IntoResponse, AppError> {
        let sessions = sqlx::query_as!(
            Session,
            r#"
            SELECT * FROM records.sessions WHERE organisation_id = $1
            "#,
            &claim.organisation_id,
        )
        .fetch_all(&pool)
        .await
        .with_context(|| format!("Cannot get all sessions by organisation"))?;

        Ok((StatusCode::OK, Json(sessions)).into_response())
    }

    pub async fn update(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(session): Json<SessionChange>,
    ) -> Result<impl IntoResponse, AppError> {
        if !User::is_admin(&pool, &claim.id).await? {
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
                organisation_id = COALESCE($3, organisation_id),
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
            session.organisation_id,
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
        if !User::is_admin(&pool, &claim.id).await? {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }

        trace!("Delete Session Triggered");
        let _ = sqlx::query!(
            r#"
            DELETE FROM records.sessions
            WHERE id = ANY($1) AND organisation_id = $2
            "#,
            &session.ids,
            claim.organisation_id
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot delete sessions"))?;

        Ok(StatusCode::OK.into_response())
    }
}
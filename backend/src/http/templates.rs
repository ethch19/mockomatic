use anyhow::{Context, anyhow};
use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::{AccessClaims, User}, AppState, SomethingID, SomethingMultipleID};
use crate::error::AppError;
use sqlx::postgres::types::PgInterval;
use tracing::instrument;
use validator::Validate;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/create", post(TemplateSession::create))
        .route("/get", get(TemplateSession::get))
        .route("/get-all", get(TemplateSession::get_all))
        .route("/update", post(TemplateSession::update))
        .route("/delete", post(TemplateSession::delete))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTemplatePayload {
    #[validate(nested)]
    pub template_session: TemplateSessionPayload,
    #[validate(nested)]
    pub template_stations: Vec<TemplateStationPayload>
}

#[derive(Debug, Serialize)]
struct TemplateSessionWithStations {
    id: Uuid,
    name: String,
    total_stations: i16,
    feedback: bool, // validate that if feedback_duration given or feedback = true, then make sure either is valid
    #[serde(default, with = "crate::http::option_pg_interval")]
    feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    intermission_duration: PgInterval,
    static_at_end: bool,
    stations: Vec<TemplateStation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSession {
    pub id: Uuid,
    pub organisation_id: Uuid,
    pub name: String,
    pub total_stations: i16,
    pub feedback: bool, // validate that if feedback_duration given or feedback = true, then make sure either is valid
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TemplateSessionPayload {
    #[validate(length(min = 1))]
    pub name: String,
    pub feedback: bool, // validate that if feedback_duration given or feedback = true, then make sure either is valid
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TemplateSessionChange {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: Option<String>,
    pub feedback: Option<bool>, // validate that if feedback_duration given or feedback = true, then make sure either is valid
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub feedback_duration: Option<PgInterval>,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub intermission_duration: Option<PgInterval>,
    pub static_at_end: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateStation {
    pub id: Uuid,
    pub template_id: Uuid,
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TemplateStationChange {
    pub id: Uuid,
    pub template_id: Uuid,
    #[validate(length(min = 1))]
    pub title: Option<String>,
    pub index: Option<i16>,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub duration: Option<PgInterval>,
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TemplateStationPayload {
    #[validate(length(min = 1))]
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}


impl TemplateSession {
    #[instrument(name = "create_template", level = "TRACE", skip(claim))]
    pub async fn create(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(req): Json<CreateTemplatePayload>,
    ) -> Result<impl IntoResponse, AppError> { // validate that if static_at_end is on, there should only be 1 station that has different times than others
        if !User::is_admin(&pool, &claim.id).await? {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        req.validate().with_context(|| "Incorrect formatting")?;
        let session_payload = req.template_session;
        let total_stations = req.template_stations.len() as i16;

        if session_payload.feedback {
            if session_payload.feedback_duration.is_none() {
                return Err(AppError::from(anyhow!("Feedback set to true but feedback duration missing")));
            }
        }
        if session_payload.feedback_duration.is_some() {
            if !session_payload.feedback {
                return Err(AppError::from(anyhow!("Feedback duration is given but feedback is set to false")));
            }
        }

        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        let session_result = sqlx::query_as!(
            TemplateSession,
            r#"
            INSERT INTO templates.sessions (organisation_id, name, total_stations, feedback, feedback_duration, intermission_duration, static_at_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            claim.organisation_id,
            session_payload.name,
            &total_stations,
            session_payload.feedback,
            session_payload.feedback_duration,
            session_payload.intermission_duration,
            session_payload.static_at_end)
            .fetch_one(&mut *transaction)
            .await;

        
        if let Err(e) = session_result {
            transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding template session. Failed transaction: {}", e))?;
            return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding template session: {}", e)));
        }

        let session_result = session_result.unwrap();
        
        // REFACTOR: check whether stations all have the same duration, or only the last one doesn't
        for station in &req.template_stations {
            let station_result = sqlx::query_as!(
                TemplateStation,
                r#"
                INSERT INTO templates.stations (template_id, title, index, duration)
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
                transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding template station. Failed transaction: {}", e))?;
                return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding template station: {}", e)));
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
            TemplateSession,
            r#"
            SELECT * FROM templates.sessions WHERE id = $1
            "#,
            session.id
        )
        .fetch_one(&pool)
        .await
        .with_context(|| format!("Cannot get template session with specific id"))?;

        let stations = sqlx::query_as!(
            TemplateStation,
            "SELECT * FROM templates.stations WHERE template_id = $1 ORDER BY index",
            result.id
        )
        .fetch_all(&pool)
        .await
        .with_context(|| format!("Cannot get template stations from session"))?;

        let session_stations = TemplateSessionWithStations {
            id: result.id,
            name: result.name,
            total_stations: result.total_stations,
            feedback: result.feedback,
            feedback_duration: result.feedback_duration,
            intermission_duration: result.intermission_duration,
            static_at_end: result.static_at_end,
            stations,
        };

        Ok((StatusCode::OK, Json(session_stations)).into_response())
    }

    pub async fn get_all(
        State(pool): State<sqlx::PgPool>,
    ) -> Result<impl IntoResponse, AppError> {
        let sessions_result = sqlx::query_as!(
            TemplateSession,
            r#"
            SELECT * FROM templates.sessions
            "#
        )
        .fetch_all(&pool)
        .await
        .with_context(|| format!("Cannot get all template session"))?;

        let session_ids: Vec<Uuid> = sessions_result.iter().map(|s| s.id).collect();
        let stations = if !session_ids.is_empty() {
            sqlx::query_as!(
            TemplateStation,
            "SELECT * FROM templates.stations WHERE template_id = ANY($1) ORDER BY template_id, index",
            &session_ids[..]
            )
            .fetch_all(&pool)
            .await
            .with_context(|| format!("Cannot get template stations from all session"))?
        } else {
            Vec::new()
        };

        let result: Vec<TemplateSessionWithStations> = sessions_result
        .into_iter()
        .map(|session| {
            let session_stations: Vec<TemplateStation> = stations
                .iter()
                .filter(|station| station.template_id == session.id)
                .cloned()
                .collect();

            TemplateSessionWithStations {
                id: session.id,
                name: session.name,
                total_stations: session.total_stations,
                feedback: session.feedback,
                feedback_duration: session.feedback_duration,
                intermission_duration: session.intermission_duration,
                static_at_end: session.static_at_end,
                stations: session_stations,
            }
        })
        .collect();

        Ok((StatusCode::OK, Json(result)).into_response())
    }

    pub async fn update(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(session): Json<TemplateSessionChange>,
    ) -> Result<impl IntoResponse, AppError> {
        if !User::is_admin(&pool, &claim.id).await? {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        session.validate().with_context(|| "Incorrect formatting")?;

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
            UPDATE templates.sessions
            SET
                name = COALESCE($2, name),
                feedback = COALESCE($3, feedback),
                feedback_duration = COALESCE($4, feedback_duration),
                intermission_duration = COALESCE($5, intermission_duration),
                static_at_end = COALESCE($6, static_at_end)
            WHERE id = $1
            "#,
            session.id,
            session.name,
            session.feedback,
            session.feedback_duration,
            session.intermission_duration,
            session.static_at_end
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot update template session: {}", session.id))?;

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

        let _ = sqlx::query!(
            r#"
            DELETE FROM templates.sessions
            WHERE id = ANY($1) AND organisation_id = $2
            "#,
            &session.ids,
            claim.organisation_id
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot delete template sessions"))?;

        Ok(StatusCode::OK.into_response())
    }
}
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
        .route("/create", post(TemplateSession::create))
        .route("/get", post(TemplateSession::get))
        .route("/get-all", post(TemplateSession::get_all))
        .route("/update", post(TemplateSession::update))
        .route("/delete", post(TemplateSession::delete))
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplatePayload {
    pub template_session: TemplateSessionPayload,
    pub template_stations: Vec<TemplateStationPayload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSession {
    pub id: Uuid,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSessionPayload {
    pub total_stations: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateStation {
    pub id: Uuid,
    pub template_id: Uuid,
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateStationPayload {
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}


impl TemplateSession {
    #[instrument(name = "create_template", level = "TRACE", skip(claims))]
    pub async fn create(
        State(pool): State<sqlx::PgPool>,
        Extension(claims): Extension<AccessClaims>,
        Json(req): Json<CreateTemplatePayload>,
    ) -> Result<impl IntoResponse, AppError> {
        let session_payload = req.template_session;
        let total_stations = req.template_stations.len() as i16;

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
    }
}
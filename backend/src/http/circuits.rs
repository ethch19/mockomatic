use axum::{extract::{State, Json, Query}, http::StatusCode, response::IntoResponse, routing::get};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;
use sqlx::Transaction;

use super::{SomethingID, AppState};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get-slot", get(get_by_slot))
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
    pub feedback: bool,
    pub intermission: bool,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_start: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_end: Option<time::OffsetDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitPayload {
    pub key: String, // A-Z
    pub female_only: bool
}

async fn get_by_slot(
    State(pool): State<sqlx::PgPool>,
    Query(slot_id): Query<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Circuit::get_by_slot(&pool, &slot_id.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

impl Circuit {
    pub async fn get_by_slot(
        pool: &sqlx::PgPool,
        slot_id: &Uuid,
    ) -> Result<Vec<Circuit>, AppError> {
        return sqlx::query_as!(
            Circuit,
            r#"
            SELECT * FROM records.circuits WHERE slot_id = $1
            "#,
            slot_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get circuits with slot_id: {}", slot_id)));
    }
    
    pub async fn get_female_slot(
        pool: &sqlx::PgPool,
        slot_id: &Uuid,
    ) -> Result<Vec<Circuit>, AppError> {
        return sqlx::query_as!(
            Circuit,
            r#"
            SELECT * FROM records.circuits WHERE slot_id = $1 AND female_only = $2
            "#,
            slot_id,
            true,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get female_only circuits with slot_id: {}", slot_id)));
    }

    pub async fn create_tx(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        session_id: &Uuid,
        slot_id: &Uuid,
        payload: &CircuitPayload,
    ) -> Result<Circuit, AppError> {
        sqlx::query_as!(
            Circuit,
            r#"
            INSERT INTO records.circuits (session_id, slot_id, key, female_only, feedback, intermission)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            session_id,
            slot_id,
            payload.key,
            payload.female_only,
            false,
            false)
            .fetch_one(&mut **tx)
            .await
            .map_err(|_| AppError::from(anyhow!("Failed to insert circuit by transaction")))
    }
}
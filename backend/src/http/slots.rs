use axum::{extract::{State, Json, Query}, http::StatusCode, response::IntoResponse, routing::get};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{circuits::CircuitPayload, runs::RunPayload, SomethingID, AppState};
use crate::error::AppError;
use sqlx::Transaction;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get-session", get(get_by_session))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlotPayload {
    pub key: String,
    pub runs: Vec<RunPayload>,
    pub circuits: Vec<CircuitPayload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: Uuid,
    pub session_id: Uuid,
    pub key: String
}

async fn get_by_session(
    State(pool): State<sqlx::PgPool>,
    Query(session_id): Query<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Slot::get_all_by_session(&pool, &session_id.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

impl Slot {
    pub async fn get_all_by_session(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
    ) -> Result<Vec<Slot>, AppError> {
        sqlx::query_as!(
            Slot,
            r#"
            SELECT * FROM records.slots WHERE session_id = $1 
            "#,
            session_id,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get slots with session_id: {}", session_id)))
    }

    pub async fn create_tx(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        session_id: &Uuid,
        payload: &SlotPayload,
    ) -> Result<Slot, AppError> {
        sqlx::query_as!(
            Slot,
            r#"
            INSERT INTO records.slots (session_id, key)
            VALUES ($1, $2)
            RETURNING *
            "#,
            session_id,
            payload.key)
            .fetch_one(&mut **tx)
            .await
            .map_err(|_| AppError::from(anyhow!("Failed to insert slot by transaction")))
    }
}
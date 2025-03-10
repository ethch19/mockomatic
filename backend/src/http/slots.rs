use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{circuits::CircuitPayload, runs::RunPayload};
use crate::error::AppError;
use sqlx::Transaction;


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
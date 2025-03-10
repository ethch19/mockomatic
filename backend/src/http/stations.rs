use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;
use sqlx::{postgres::types::PgInterval, Transaction};
use anyhow::{Context, anyhow};


#[derive(Debug, Serialize, Deserialize)]
pub struct Station {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StationPayload {
    pub title: String,
    pub index: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub duration: PgInterval,
}

impl Station {
    pub async fn get_by_session( // all circuits in a session have the same number of stations
        pool: &sqlx::PgPool,
        session_id: &Uuid,
    ) -> Result<Vec<Station>, AppError> {
        return sqlx::query_as!(
            Station,
            r#"
            SELECT * FROM records.stations WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get stations with session_id: {}", session_id)));
    }

    pub async fn create_tx(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        session_id: &Uuid,
        payload: &StationPayload,
    ) -> Result<(), AppError> {
        sqlx::query_as!(
            Station,
            r#"
            INSERT INTO records.stations (session_id, title, index, duration)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            session_id,
            payload.title,
            payload.index,
            payload.duration)
            .fetch_one(&mut **tx)
            .await
            .with_context(|| format!("Failed to insert stations by transaction"))?;
        Ok(())
    }
}
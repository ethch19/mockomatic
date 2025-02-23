use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

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
pub struct CircuitPayload {
    pub key: String, // A-Z
    pub female_only: bool
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
}
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{circuits::CircuitPayload, runs::RunPayload};
use crate::error::AppError;


#[derive(Debug, Serialize, Deserialize)]
pub struct SlotPayload {
    pub slot_time: String,
    pub runs: Vec<RunPayload>,
    pub circuits: Vec<CircuitPayload>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: Uuid,
    pub session_id: Uuid,
    pub slot_time: String
}

#[derive(Debug)]
pub enum SlotTime {
    AM,
    PM,
}

impl Slot {
    pub async fn get_by_session(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        slot_time: SlotTime,
    ) -> Result<Slot, AppError> {
        match slot_time {
            SlotTime::AM => {
                return sqlx::query_as!(
                    Slot,
                    r#"
                    SELECT * FROM records.slots WHERE session_id = $1 AND slot_time = $2
                    "#,
                    session_id,
                    "AM",
                )
                .fetch_one(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get AM slot with session_id: {}", session_id)));
            }
            SlotTime::PM => {
                return sqlx::query_as!(
                    Slot,
                    r#"
                    SELECT * FROM records.slots WHERE session_id = $1 AND slot_time = $2
                    "#,
                    session_id,
                    "PM",
                )
                .fetch_one(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get PM slot with session_id: {}", session_id)));
            }
        }
    }

}
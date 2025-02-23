use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub slot_id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_start: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_end: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_start: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub timer_end: Option<time::OffsetDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunPayload {
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_start: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_end: time::OffsetDateTime
}

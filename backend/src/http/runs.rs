use axum::{extract::{State, Json, Query}, http::StatusCode, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;
use anyhow::{Context, anyhow};
use sqlx::Transaction;

use super::{SomethingID, AppState};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get-slot", get(get_by_slot))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub slot_id: Uuid,
    pub flip_allocation: bool,
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
    pub flip_allocation: bool,
    #[serde(with = "time::serde::iso8601")]
    pub scheduled_start: time::OffsetDateTime,
}

#[derive(Debug)]
pub enum RunTime {
    AM, // runs that START before 12:00
    PM, // runs that START after 12:00
}

async fn get_by_slot(
    State(pool): State<sqlx::PgPool>,
    Query(slot_id): Query<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Run::get_all_by_slot(&pool, &slot_id.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

impl Run {
    pub async fn get_all_by_slot(
        pool: &sqlx::PgPool,
        slot_id: &Uuid,
    ) -> Result<Vec<Run>, AppError> {
        sqlx::query_as!(
            Run,
            r#"
            SELECT * FROM records.runs WHERE slot_id = $1
            "#,
            slot_id,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get slots with slot_id: {}", slot_id)))
    }

    pub async fn get_by_time(
        pool: &sqlx::PgPool,
        slot_id: &Uuid,
        run_time: RunTime,
    ) -> Result<Vec<Run>, AppError> {
        match run_time {
            RunTime::AM => {
                sqlx::query_as!(
                    Run,
                    r#"
                    SELECT * FROM records.runs WHERE slot_id = $1 AND EXTRACT(HOUR FROM scheduled_start) < 12
                    "#,
                    slot_id
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Unable to fetch all runs that start before 12pm in your timezone currently")))
            },
            RunTime::PM => {
                sqlx::query_as!(
                    Run,
                    r#"
                    SELECT * FROM records.runs WHERE slot_id = $1 AND EXTRACT(HOUR FROM scheduled_start) > 11
                    "#,
                    slot_id
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Unable to fetch all runs that start after 12pm in your timezone currently")))
            }
        }
    }

    pub async fn create_tx(
        tx: &mut Transaction<'static, sqlx::Postgres>,
        slot_id: &Uuid,
        payload: &RunPayload,
        scheduled_end: time::OffsetDateTime,
    ) -> Result<(), AppError> {
        sqlx::query_as!(
            Run,
            r#"
            INSERT INTO records.runs (slot_id, flip_allocation, scheduled_start, scheduled_end)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            slot_id,
            payload.flip_allocation,
            payload.scheduled_start,
            scheduled_end,
        )
        .fetch_one(&mut **tx)
        .await
        .with_context(|| format!("Failed to create run from transaction"))?;
        Ok(())
    }
}
use anyhow::{Context, anyhow};
use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse, routing::post, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::AccessClaims, AppState, SomethingID, SomethingMultipleID};
use crate::error::AppError;
use sqlx::postgres::types::PgInterval;
use tracing::instrument;

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

#[derive(Debug, Serialize)]
struct TemplateSessionWithStations {
    id: Uuid,
    name: String,
    total_stations: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    intermission_duration: PgInterval,
    static_at_end: bool,
    stations: Vec<TemplateStation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSession {
    pub id: Uuid,
    pub name: String,
    pub total_stations: i16,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSessionPayload {
    pub name: String,
    #[serde(default, with = "crate::http::pg_interval")]
    pub intermission_duration: PgInterval,
    pub static_at_end: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSessionChange {
    pub id: Uuid,
    pub name: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateStationChange {
    pub id: Uuid,
    pub template_id: Uuid,
    pub title: Option<String>,
    pub index: Option<i16>,
    #[serde(default, with = "crate::http::option_pg_interval")]
    pub duration: Option<PgInterval>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateStationPayload {
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
    ) -> Result<impl IntoResponse, AppError> {
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        let session_payload = req.template_session;
        let total_stations = req.template_stations.len() as i16;

        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        let session_result = sqlx::query_as!(
            TemplateSession,
            r#"
            INSERT INTO templates.sessions (name, total_stations, intermission_duration, static_at_end)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            session_payload.name,
            &total_stations,
            session_payload.intermission_duration,
            session_payload.static_at_end)
            .fetch_one(&mut *transaction)
            .await;

        
        if let Err(e) = session_result {
            transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding template session. Failed transaction: {}", e))?;
            return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding template session: {}", e)));
        }

        let session_result = session_result.unwrap();
        
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
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        let _ = sqlx::query!(
            r#"
            UPDATE templates.sessions
            SET
                name = COALESCE($2, name),
                intermission_duration = COALESCE($3, intermission_duration),
                static_at_end = COALESCE($4, static_at_end)
            WHERE id = $1
            "#,
            session.id,
            session.name,
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
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }

        for session_id in &session.ids {
            let _ = sqlx::query!(
                r#"
                DELETE FROM templates.sessions
                WHERE id = $1
                "#,
                session_id
            )
            .execute(&pool)
            .await
            .with_context(|| format!("Cannot delete template session with ID: {}", session_id))?;
        }

        Ok(StatusCode::OK.into_response())
    }
}
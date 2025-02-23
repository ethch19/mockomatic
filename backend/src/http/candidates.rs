use anyhow::{Context, anyhow};
use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::AccessClaims, AppState, SomethingID};
use crate::error::AppError;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get", get(Candidate::get))
        .route("/get-session-all", get(Candidate::get_session_all))
        .route("/get-slot-all", get(Candidate::get_slot_all))
        .route("/create", post(create))
        .route("/update", post(Candidate::update))
        .route("/delete", post(Candidate::delete))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Candidate {
    pub id: Uuid,
    pub session_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub shortcode: String,
    pub female_only: bool,
    pub partner_pref: Option<String>,
    pub checked_in: bool, 
    pub slot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CandidatePayload {
    pub session_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub shortcode: String,
    pub female_only: bool,
    pub partner_pref: Option<String>,
    pub checked_in: bool, 
    pub slot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CandidateChange {
    pub id: Uuid,
    pub session_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub shortcode: Option<String>,
    pub female_only: Option<bool>,
    pub partner_pref: Option<String>,
    pub checked_in: Option<bool>, 
    pub slot: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CandidatesBySlot {
    pub slot: String,
    pub candidates: Vec<Candidate>,
}

async fn create(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(candidate): Json<CandidatePayload>,
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let result = Candidate::create(&pool, candidate).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

pub async fn create_fill(session_id: Uuid, pool: &sqlx::PgPool) -> Result<Candidate, AppError> {
    let candidate = CandidatePayload {
        session_id,
        first_name: "Fill".to_string(),
        last_name: "Candidate".to_string(),
        shortcode: Uuid::new_v4().to_string(),
        female_only: false,
        partner_pref: None,
        slot: None,
        checked_in: false,
    };
    Candidate::create(pool, candidate).await
}

impl Candidate {
    pub async fn get(
        State(pool): State<sqlx::PgPool>,
        Json(candidate): Json<SomethingID>,
    ) -> Result<impl IntoResponse, AppError> {
        let result = sqlx::query_as!(
            Candidate,
            r#"
            SELECT * FROM people.candidates WHERE id = $1
            "#,
            candidate.id
        )
        .fetch_one(&pool)
        .await
        .with_context(|| format!("Cannot get candidate with specific id"))?;

        Ok((StatusCode::OK, Json(result)).into_response())
    }

    pub async fn get_session_all(
        State(pool): State<sqlx::PgPool>,
        Json(session): Json<SomethingID>,
    ) -> Result<impl IntoResponse, AppError> {
        let result = sqlx::query_as!(
            Candidate,
            r#"
            SELECT * FROM people.candidates WHERE session_id = $1
            "#,
            session.id
        )
        .fetch_all(&pool)
        .await
        .with_context(|| format!("Cannot get all candidates with specific session_id"))?;

        Ok((StatusCode::OK, Json(result)).into_response())
    }

    pub async fn get_slot_all(
    ) -> Result<impl IntoResponse, AppError> {
        // This is based on allocation 
        Ok(StatusCode::OK.into_response())
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        candidate: CandidatePayload,
    ) -> Result<Candidate, AppError> {
        sqlx::query_as!(
            Candidate,
            r#"
            INSERT INTO people.candidates (session_id, first_name, last_name, shortcode, female_only, partner_pref, checked_in, slot)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            candidate.session_id,
            candidate.first_name,
            candidate.last_name,
            candidate.shortcode,
            candidate.female_only,
            candidate.partner_pref,
            candidate.checked_in,
            candidate.slot
        )
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot create new examiner")))
    }

    pub async fn update(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(candidate): Json<CandidateChange>,
    ) -> Result<impl IntoResponse, AppError> {
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        let _ = sqlx::query!(
            r#"
            UPDATE people.candidates
            SET
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                shortcode = COALESCE($5, shortcode),
                female_only = COALESCE($6, female_only),
                partner_pref = COALESCE($7, partner_pref),
                checked_in = COALESCE($8, checked_in),
                slot = COALESCE($9, slot)
            WHERE id = $1 AND session_id = $2
            "#,
            candidate.id,
            candidate.session_id,
            candidate.first_name,
            candidate.last_name,
            candidate.shortcode,
            candidate.female_only,
            candidate.partner_pref,
            candidate.checked_in,
            candidate.slot
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot update candidate"))?;

        Ok(StatusCode::OK.into_response())
    }

    pub async fn delete(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(candidate): Json<SomethingID>,
    ) -> Result<impl IntoResponse, AppError> {
        if !claim.admin {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }
        let _ = sqlx::query!(
            r#"
            DELETE FROM people.candidates
            WHERE id = $1
            "#,
            candidate.id
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot delete candidate"))?;

        Ok(StatusCode::OK.into_response())
    }
}
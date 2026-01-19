use anyhow::{Context, anyhow};
use axum::{extract::{Json, State, Query}, http::StatusCode, response::IntoResponse, routing::{get, post}, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::AccessClaims, AppState, SomethingID, allocations::Availability, runs::RunTime};
use crate::{error::AppError, http::users::User};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get", get(Candidate::get))
        .route("/get-session-all", get(get_session_all))
        .route("/get-slot-all", get(Candidate::get_slot_all))
        .route("/create", post(create))
        .route("/update", post(Candidate::update))
        .route("/delete", post(delete))
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
    pub am: Option<bool>,
    pub pm: Option<bool>,
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
    pub am: Option<bool>,
    pub pm: Option<bool>,
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
    pub am: Option<bool>,
    pub pm: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CandidateExcel {
    pub first_name: String,
    pub last_name: String,
    pub shortcode: String,
    pub female_only: bool,
    pub partner_pref: Option<String>,
    pub am: Option<bool>,
    pub pm: Option<bool>
}

#[derive(Debug, Serialize)]
pub struct CandidatesByTime {
    pub time: Availability,
    pub candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
pub struct DeleteCandidatePayload {
    pub session_id: Uuid,
    pub ids: Vec<Uuid>
}

async fn create(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(candidate): Json<CandidatePayload>,
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let result = Candidate::create(&pool, claim.organisation_id, candidate).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn get_session_all(
    State(pool): State<sqlx::PgPool>,
    session_id: Query<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Candidate::get_all_by_session(&pool, &session_id.0.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn delete(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(candidates): Json<DeleteCandidatePayload>,
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    Candidate::delete(pool, claim.organisation_id, candidates).await?;
    Ok((StatusCode::OK).into_response())
}

pub async fn create_fill(session_id: Uuid, organsation_id: Uuid, pool: &sqlx::PgPool, time: Option<Availability>, female_only: bool) -> Result<Candidate, AppError> {
    if let Some(can_ava) = time {
        let candidate = CandidatePayload {
            session_id,
            first_name: "fill".to_string(),
            last_name: "candidate".to_string(),
            shortcode: Uuid::new_v4().to_string(),
            female_only,
            partner_pref: None,
            am: Some(can_ava.am),
            pm: Some(can_ava.pm),
            checked_in: false,
        };
        Candidate::create(pool, organsation_id, candidate).await
    } else {
        let candidate = CandidatePayload {
            session_id,
            first_name: "fill".to_string(),
            last_name: "candidate".to_string(),
            shortcode: Uuid::new_v4().to_string(),
            female_only,
            partner_pref: None,
            am: Some(true),
            pm: Some(true),
            checked_in: false,
        };
        Candidate::create(pool, organsation_id, candidate).await
    }
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

    pub async fn get_all_by_session(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
    ) -> Result<Vec<Candidate>, AppError> {
        sqlx::query_as!(
            Candidate,
            r#"
            SELECT * FROM people.candidates WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all candidates with specific session_id")))
    }

    pub async fn get_ava_all(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        ava: Availability,
    ) -> Result<Vec<Candidate>, AppError> {
        sqlx::query_as!(
            Candidate,
            r#"
            SELECT * FROM people.candidates WHERE session_id = $1 AND am = $2 AND pm = $3
            "#,
            session_id,
            ava.am,
            ava.pm
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all candidates with specific avability")))
    }

    pub async fn get_female_ava_all(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        ava: Availability,
    ) -> Result<Vec<Candidate>, AppError> {
        sqlx::query_as!(
            Candidate,
            r#"
            SELECT * FROM people.candidates WHERE session_id = $1 AND female_only = TRUE AND am = $2 AND pm = $3
            "#,
            session_id,
            ava.am,
            ava.pm
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all female_only candidates with specific avability")))
    }

    pub async fn get_all_by_time(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        run_time: RunTime,
    ) -> Result<Vec<Candidate>, AppError> {
        match run_time {
            RunTime::AM => {
                sqlx::query_as!(
                    Candidate,
                    r#"
                    SELECT * FROM people.candidates WHERE session_id = $1 AND am = TRUE
                    "#,
                    session_id
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get all AM candidates")))
            },
            RunTime::PM => {
                sqlx::query_as!(
                    Candidate,
                    r#"
                    SELECT * FROM people.candidates WHERE session_id = $1 AND pm = TRUE
                    "#,
                    session_id
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get all PM candidates")))
            }
        }
        
    }

    pub async fn get_all_female_by_time(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        run_time: RunTime,
    ) -> Result<Vec<Candidate>, AppError> { // includes BOTH AM/PM candidates and ones that are available full day
        match run_time{
            RunTime::AM => {
                sqlx::query_as!(
                    Candidate,
                    r#"
                    SELECT * FROM people.candidates WHERE session_id = $1 AND am = TRUE AND female_only = TRUE
                    "#,
                    session_id,
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get all AM female_only candidates")))
            },
            RunTime::PM => {
                sqlx::query_as!(
                    Candidate,
                    r#"
                    SELECT * FROM people.candidates WHERE session_id = $1 AND pm = TRUE  AND female_only = TRUE
                    "#,
                    session_id,
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot to get all PM female_only candidates")))
            }
        }
    }

    pub async fn get_female_all(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
    ) -> Result<Vec<Candidate>, AppError> {
        sqlx::query_as!(
            Candidate,
            r#"
            SELECT * FROM people.candidates WHERE session_id = $1 AND female_only = TRUE
            "#,
            session_id,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all female_only candidates")))
    }


    pub async fn get_slot_all(
    ) -> Result<impl IntoResponse, AppError> {
        // This is based on allocation 
        // Get all candidates allocated to a particular slot_id
        Ok(StatusCode::OK.into_response())
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        organisation_id: Uuid,
        candidate: CandidatePayload,
    ) -> Result<Candidate, AppError> {
        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        sqlx::query!(
            r#"
            SELECT organisation_id FROM records.sessions
            WHERE id = $1 AND organisation_id = $2
            "#,
            &candidate.session_id, // given by client
            &organisation_id // from claims
        )
        .fetch_one(&mut *transaction)
        .await
        .with_context(|| format!("Session not found or you do not have permission for this operation"))?;

        let candidate = sqlx::query_as!(
            Candidate,
            r#"
            INSERT INTO people.candidates (session_id, first_name, last_name, shortcode, female_only, partner_pref, checked_in, am, pm)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            candidate.session_id,
            candidate.first_name,
            candidate.last_name,
            candidate.shortcode,
            candidate.female_only,
            candidate.partner_pref,
            candidate.checked_in,
            candidate.am,
            candidate.pm,
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot create new candidate")));

        transaction.commit().await.with_context(|| format!("Transaction failed to commit"))?;
        return candidate
    }

    pub async fn update(
        State(pool): State<sqlx::PgPool>,
        Extension(claim): Extension<AccessClaims>,
        Json(candidate): Json<CandidateChange>,
    ) -> Result<impl IntoResponse, AppError> {
        if !User::is_admin(&pool, &claim.id).await? {
            return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
        }

        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        sqlx::query!(
            r#"
            SELECT organisation_id FROM records.sessions
            WHERE id = $1 AND organisation_id = $2
            "#,
            &candidate.session_id, // given by client
            &claim.organisation_id // from claims
        )
        .fetch_one(&mut *transaction)
        .await
        .with_context(|| format!("Session not found or you do not have permission for this operation"))?;

        sqlx::query!(
            r#"
            UPDATE people.candidates
            SET
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                shortcode = COALESCE($5, shortcode),
                female_only = COALESCE($6, female_only),
                partner_pref = COALESCE($7, partner_pref),
                checked_in = COALESCE($8, checked_in),
                am = COALESCE($9, am),
                pm = COALESCE($10, pm)
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
            candidate.am,
            candidate.pm,
        )
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("Cannot update candidate"))?;

        transaction.commit().await.with_context(|| format!("Transaction failed to commit"))?;

        Ok(StatusCode::OK.into_response())
    }

    pub async fn delete(
        pool: sqlx::PgPool,
        organisation_id: Uuid,
        candidates: DeleteCandidatePayload,
    ) -> Result<(), AppError> {
        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        sqlx::query!(
            r#"
            SELECT organisation_id FROM records.sessions
            WHERE id = $1 AND organisation_id = $2
            "#,
            &candidates.session_id, // given by client
            &organisation_id // from claims
        )
        .fetch_one(&mut *transaction)
        .await
        .with_context(|| format!("Session not found or you do not have permission for this operation"))?;

        sqlx::query!(
            r#"
            DELETE FROM people.candidates
            WHERE id = ANY($1) AND session_id = $2
            "#,
            &candidates.ids,
            &candidates.session_id
        )
        .execute(&mut *transaction)
        .await
        .with_context(|| format!("Failed to delete candidates"))?;

        transaction.commit().await.with_context(|| format!("Transaction failed to commit"))?;
        Ok(())
    }
}
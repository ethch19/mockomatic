use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode, response::IntoResponse,
    routing::{get, post}
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use anyhow::{anyhow, Context};

use crate::error::AppError;

use super::{runs::RunTime, users::{AccessClaims, User}, AppState, SomethingID, SomethingMultipleID, allocations::Availability};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get", get(get_by_id))
        .route("/get-session-all", get(get_all_by_session))
        .route("/create", post(create))
        .route("/update", post(update))
        .route("/delete", post(delete))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Examiner {
    pub id: Uuid,
    pub session_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub shortcode: String,
    pub female: bool,
    pub am: bool,
    pub pm: bool,
    pub checked_in: bool, 
}

#[derive(Debug, Deserialize)]
pub struct ExaminerPayload {
    pub session_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub shortcode: String,
    pub female: bool,
    pub am: bool,
    pub pm: bool,
    pub checked_in: bool, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExaminerExcel {
    pub first_name: String,
    pub last_name: String,
    pub shortcode: String,
    pub female: bool,
    pub am: bool,
    pub pm: bool
}

#[derive(Debug, Deserialize)]
pub struct ExaminerChange {
    pub id: Uuid,
    pub session_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub shortcode: Option<String>,
    pub female: Option<bool>,
    pub am: Option<bool>,
    pub pm: Option<bool>,
    pub checked_in: Option<bool>, 
}

async fn get_by_id(
    State(pool): State<sqlx::PgPool>,
    examiner: Query<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Examiner::get(&pool, &examiner.0.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn get_all_by_session(
    State(pool): State<sqlx::PgPool>,
    session: Query<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Examiner::get_all_by_session(&pool, &session.0.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn create(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(examiner): Json<ExaminerPayload>,
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let result = Examiner::create(&pool, examiner).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn update(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(examiner): Json<ExaminerChange>,
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let result = Examiner::update(pool, examiner).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

pub async fn create_fill(session_id: Uuid, pool: &sqlx::PgPool, time: Option<Availability>, female: bool) -> Result<Examiner, AppError> {
    if let Some(exam_ava) = time {
        let examiner = ExaminerPayload {
            session_id,
            first_name: "fill".to_string(),
            last_name: "candidate".to_string(),
            shortcode: Uuid::new_v4().to_string(),
            female,
            am: exam_ava.am,
            pm: exam_ava.pm,
            checked_in: false,
        };
        Examiner::create(pool, examiner).await
    } else {
        let examiner = ExaminerPayload {
            session_id,
            first_name: "fill".to_string(),
            last_name: "candidate".to_string(),
            shortcode: Uuid::new_v4().to_string(),
            female,
            am: true,
            pm: true,
            checked_in: false,
        };
        Examiner::create(pool, examiner).await
    }
}

async fn delete(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(examiner): Json<SomethingMultipleID>,
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    Examiner::delete(pool, examiner.ids).await?;
    Ok((StatusCode::OK).into_response())
}

impl Examiner {
    pub async fn get(
        pool: &sqlx::PgPool,
        examiner_id: &Uuid,
    ) -> Result<Examiner, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE id = $1
            "#,
            examiner_id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get examiner with id: {}", examiner_id)))
    }

    pub async fn get_all_by_session(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
    ) -> Result<Vec<Examiner>, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all examiners with session_id: {}", session_id)))
    }

    pub async fn get_all_by_time(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        run_time: RunTime,
    ) -> Result<Vec<Examiner>, AppError> {
        match run_time {
            RunTime::AM => {
                sqlx::query_as!(
                    Examiner,
                    r#"
                    SELECT * FROM people.examiners WHERE session_id = $1 AND am = TRUE
                    "#,
                    session_id
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Unable to get all AM examiners")))
            },
            RunTime::PM => {
                sqlx::query_as!(
                    Examiner,
                    r#"
                    SELECT * FROM people.examiners WHERE session_id = $1 AND pm = TRUE
                    "#,
                    session_id
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Unable to get all PM examiners")))
            }
        }
    }

    pub async fn get_all_female_by_time(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        run_time: RunTime,
    ) -> Result<Vec<Examiner>, AppError> { // includes BOTH AM/PM examiner and FULL day ones
        match run_time{
            RunTime::AM => {
                sqlx::query_as!( 
                    Examiner,
                    r#"
                    SELECT * FROM people.examiners WHERE session_id = $1 AND am = TRUE AND female = TRUE
                    "#,
                    session_id,
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Unable to get all AM female examiners")))
            },
            RunTime::PM => {
                sqlx::query_as!(
                    Examiner,
                    r#"
                    SELECT * FROM people.examiners WHERE session_id = $1 AND pm = TRUE  AND female = TRUE
                    "#,
                    session_id,
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Unable to get all PM female examiners")))
            }
        }
    }

    pub async fn get_ava_all(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        ava: Availability,
    ) -> Result<Vec<Examiner>, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE session_id = $1 AND am = $2 AND pm = $3
            "#,
            session_id,
            ava.am,
            ava.pm
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all examiners with specific availability")))
    }

    pub async fn get_female_ava_all(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
        ava: Availability,
    ) -> Result<Vec<Examiner>, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE session_id = $1 AND am = $2 AND pm = $3 AND female = TRUE
            "#,
            session_id,
            ava.am,
            ava.pm
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all female examiners with specific availability")))
    }

    pub async fn get_female_all(
        pool: &sqlx::PgPool,
        session_id: &Uuid,
    ) -> Result<Vec<Examiner>, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE session_id = $1 AND female = TRUE
            "#,
            session_id,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all female_only examiners")))
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        examiner: ExaminerPayload,
    ) -> Result<Examiner, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            INSERT INTO people.examiners (session_id, first_name, last_name, shortcode, female, am, pm, checked_in)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            examiner.session_id,
            examiner.first_name,
            examiner.last_name,
            examiner.shortcode,
            examiner.female,
            examiner.am,
            examiner.pm,
            examiner.checked_in
        )
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot create new examiner")))
    }

    pub async fn update(
        pool: sqlx::PgPool,
        examiner: ExaminerChange,
    ) -> Result<Examiner, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            UPDATE people.examiners
            SET
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                shortcode = COALESCE($5, shortcode),
                female = COALESCE($6, female),
                am = COALESCE($7, am),
                pm = COALESCE($8, pm),
                checked_in = COALESCE($9, checked_in)
            WHERE id = $1 AND session_id = $2
            RETURNING *
            "#,
            examiner.id,
            examiner.session_id,
            examiner.first_name,
            examiner.last_name,
            examiner.shortcode,
            examiner.female,
            examiner.am,
            examiner.pm,
            examiner.checked_in
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot update examiner")))
    }

    pub async fn delete(
        pool: sqlx::PgPool,
        examiner_ids: Vec<Uuid>,
    ) -> Result<(), AppError> {
        let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

        for examiner_id in &examiner_ids {
            sqlx::query!(
                r#"
                DELETE FROM people.examiners
                WHERE id = $1
                "#,
                examiner_id
            )
            .execute(&mut *transaction)
            .await
            .with_context(|| format!("Cannot delete examiner with id: {}", examiner_id))?;
        }

        transaction.commit().await.with_context(|| format!("Rolled back successful. Transaction failed to commit"))?;
        Ok(())
    }
}
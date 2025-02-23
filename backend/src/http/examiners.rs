use axum::{
    extract::{Extension, Json, Multipart, Query, State},
    http::StatusCode, response::IntoResponse,
    routing::{get, post}
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use anyhow::{anyhow, Context};
use calamine::{Reader, Xlsx, open_workbook_from_rs, Data, DataType};
use std::collections::HashSet;
use std::io::Cursor;

use crate::error::AppError;

use super::{slots::SlotTime, users::AccessClaims, AppState, SomethingID};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/get", get(get_by_id))
        .route("/get-all", get(get_all_by_session))
        .route("/create", post(create))
        .route("/update", post(update))
        .route("/delete", post(delete))
        .route("/upload-xlsx", post(upload_examiners))
}

#[derive(Debug, Serialize, Deserialize)]
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


const REQUIRED_EXAMINER_HEADERS: &[&str] = &[
    "first_name",
    "last_name",
    "shortcode",
    "female",
    "am",
    "pm",
];

async fn upload_examiners(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    session_data: Query<SomethingID>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }

    let mut file_data = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| anyhow!("Error reading multipart field: {}", e))? {
        if field.name() == Some("file") {
            file_data = Some(field.bytes().await.map_err(|e| anyhow!("Error reading file bytes: {}", e))?);
            break;
        }
    }
    let file_data = file_data.ok_or(anyhow!("No file uploaded"))?;
    let session_id: SomethingID = session_data.0;

    let cursor = Cursor::new(file_data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| anyhow!("Failed to read XLSX file: {}", e))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let sheet = workbook.worksheet_range(&sheet_names[0])
        .map_err(|e| anyhow!("No sheets found in workbook: {}", e))?;

    let headers: Vec<String> = sheet.rows()
        .next()
        .ok_or(anyhow!("Empty spreadsheet"))?
        .iter()
        .map(|cell| cell.get_string().unwrap_or("").to_lowercase())
        .collect();
    let required_headers: HashSet<&str> = REQUIRED_EXAMINER_HEADERS.iter().copied().collect();
    let header_set: HashSet<&str> = headers.iter().map(|s| s.as_str()).collect();
    
    if !required_headers.is_subset(&header_set) {
        return Err(anyhow!("Missing required headers"))?;
    }

    let mut header_indices = std::collections::HashMap::new();
    for (i, header) in headers.iter().enumerate() {
        header_indices.insert(header.as_str(), i);
    }

    let mut examiners: Vec<ExaminerExcel> = Vec::new();
    for (row_idx, row) in sheet.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.is_empty()) {
            continue;
        }
        let get_bool = |value: &Data, row_idx: usize, header: &str| -> Result<bool, AppError> {
            match value {
                Data::String(s) => s.parse::<bool>().map_err(|_| AppError::Anyhow(anyhow!(
                        "Invalid boolean value at row {}, column {}",
                        row_idx + 2,
                        header
                    ))),
                Data::Bool(b) => Ok(*b),
                _ => Err(anyhow!(
                    "Invalid data type at row {}, column {}",
                    row_idx + 2,
                    header
                ))?,
            }
        };

        let examiner = ExaminerExcel {
            first_name: row[header_indices["first_name"]]
                .get_string()
                .ok_or_else(|| {
                    anyhow!(
                        "Missing first_name at row {}",
                        row_idx + 2
                    )
                })?
                .to_string(),
            last_name: row[header_indices["last_name"]]
                .get_string()
                .ok_or_else(|| {
                    anyhow!(
                        "Missing last_name at row {}",
                        row_idx + 2
                    )
                })?
                .to_string(),
            shortcode: row[header_indices["shortcode"]]
                .get_string()
                .ok_or_else(|| {
                    anyhow!(
                        "Missing shortcode at row {}",
                        row_idx + 2
                    )
                })?
                .to_string(),
            female: get_bool(&row[header_indices["female"]], row_idx, "female")?,
            am: get_bool(&row[header_indices["am"]], row_idx, "am")?,
            pm: get_bool(&row[header_indices["pm"]], row_idx, "pm")?
        };

        examiners.push(examiner);
    }

    let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;

    for examiner in &examiners {
        let result = sqlx::query!(
            r#"
            INSERT INTO people.examiners (
                session_id,
                first_name,
                last_name,
                shortcode,
                female,
                am,
                pm,
                checked_in
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            &session_id.id,
            examiner.first_name,
            examiner.last_name,
            examiner.shortcode,
            examiner.female,
            examiner.am,
            examiner.pm,
            false
        )
        .execute(&mut *transaction)
        .await;

        if let Err(e) = result {
            transaction.rollback().await.with_context(|| format!("Failed rollback whilst adding examiners. Failed transaction: {}", e))?;
            return Err(AppError::from(anyhow!("Rolled back successful. Transaction failed whilst adding examiners: {}", e)));
        }
    }

    transaction.commit().await.with_context(|| format!("Rolled back successful. Transaction failed to commit"))?;

    Ok(StatusCode::CREATED.into_response())
}

async fn get_by_id(
    State(pool): State<sqlx::PgPool>,
    Json(examiner): Json<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Examiner::get(pool, examiner.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn get_all_by_session(
    State(pool): State<sqlx::PgPool>,
    Json(session): Json<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    let result = Examiner::get_all_by_session(pool, session.id).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn create(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(examiner): Json<ExaminerPayload>,
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let result = Examiner::create(pool, examiner).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn update(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(examiner): Json<ExaminerChange>,
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    let result = Examiner::update(pool, examiner).await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

async fn delete(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    Json(examiner): Json<SomethingID>,
) -> Result<impl IntoResponse, AppError> {
    if !claim.admin {
        return Ok((StatusCode::FORBIDDEN, "You do not have access to perform this operation").into_response())
    }
    Examiner::delete(pool, examiner.id).await?;
    Ok((StatusCode::OK).into_response())
}

impl Examiner {
    pub async fn get(
        pool: sqlx::PgPool,
        examiner_id: Uuid,
    ) -> Result<Examiner, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE id = $1
            "#,
            examiner_id
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get examiner with id: {}", examiner_id)))
    }

    pub async fn get_all_by_session(
        pool: sqlx::PgPool,
        session_id: Uuid,
    ) -> Result<Vec<Examiner>, AppError> {
        sqlx::query_as!(
            Examiner,
            r#"
            SELECT * FROM people.examiners WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_all(&pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot get all examiners with session_id: {}", session_id)))
    }

    pub async fn get_all_by_session_slot(
        pool: &sqlx::PgPool,
        session_id: Uuid,
        slot_time: SlotTime,
    ) -> Result<Vec<Examiner>, AppError> {
        match slot_time {
            SlotTime::AM => {
                sqlx::query_as!(
                    Examiner,
                    r#"
                    SELECT * FROM people.examiners WHERE session_id = $1 AND am = $2
                    "#,
                    session_id,
                    true
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get all AM examiners with session_id: {}", session_id)))
            }
            SlotTime::PM => {
                sqlx::query_as!(
                    Examiner,
                    r#"
                    SELECT * FROM people.examiners WHERE session_id = $1 AND pm = $2
                    "#,
                    session_id,
                    true
                )
                .fetch_all(pool)
                .await
                .map_err(|_| AppError::from(anyhow!("Cannot get all PM examiners with session_id: {}", session_id)))
            }
        }
        
    }

    pub async fn create(
        pool: sqlx::PgPool,
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
        .fetch_one(&pool)
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
        examiner_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM people.examiners
            WHERE id = $1
            "#,
            examiner_id
        )
        .execute(&pool)
        .await
        .map_err(|_| AppError::from(anyhow!("Cannot delete examiner with id: {}", examiner_id)));
        Ok(())
    }
}
use axum::{
    extract::{Extension, Multipart, Query, State},
    http::StatusCode, response::IntoResponse,
    routing::post
};
use anyhow::{anyhow, Context};
use calamine::{Reader, Xlsx, open_workbook_from_rs, Data, DataType};
use std::collections::HashSet;
use std::io::Cursor;

use crate::error::AppError;

use super::{users::{AccessClaims, User}, AppState, SomethingID, examiners::ExaminerExcel, candidates::CandidateExcel};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/upload-xlsx", post(upload_xlsx))
}

const REQUIRED_EXAMINER_HEADERS: &[&str] = &[
    "first_name",
    "last_name",
    "shortcode",
    "female",
    "am",
    "pm",
];

const REQUIRED_CANDIDATE_HEADERS: &[&str] = &[
    "first_name",
    "last_name",
    "shortcode",
    "female_only",
    "partner_pref",
];

fn get_bool(value: &Data, row_index: usize, header: &str) -> Result<bool, AppError> {
    match value {
        Data::Empty => Ok(false),
        Data::String(s) => s.parse::<bool>().map_err(|_| AppError::Anyhow(anyhow!(
                "Invalid boolean value at row {}, column {}",
                row_index + 2,
                header
            ))),
        Data::Bool(b) => Ok(*b),
        _ => Err(anyhow!(
            "Invalid data type at row {}, column {}",
            row_index + 2,
            header
        ))?,
    }
}

fn get_option_bool(value: &Data, row_index: usize, header: &str) -> Result<Option<bool>, AppError> {
    match value {
        Data::Empty => Ok(Some(false)),
        Data::String(s) => {
            let bool_value = s.parse::<bool>().map_err(|_| AppError::Anyhow(anyhow!(
                "Invalid boolean value at row {}, column {}",
                row_index + 2,
                header
            )))?;
            Ok(Some(bool_value))
        },
        Data::Bool(b) => Ok(Some(*b)),
        _ => Err(anyhow!(
            "Invalid data type at row {}, column {}",
            row_index + 2,
            header
        ))?,
    }
}

fn get_pref(value: &Data, row_index: usize, header: &str) -> Result<Option<String>, AppError> {
    match value {
        Data::Empty => Ok(None),
        Data::String(s) => { Ok(Some(s.to_lowercase()))
        },
        _ => Err(anyhow!(
            "Invalid data type at row {}, column {}",
            row_index + 2,
            header
        ))?,
    }
}

async fn upload_xlsx(
    State(pool): State<sqlx::PgPool>,
    Extension(claim): Extension<AccessClaims>,
    session_data: Query<SomethingID>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    if !User::is_admin(&pool, &claim.id).await? {
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
    let session_id = session_data.0.id;

    let cursor = Cursor::new(file_data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| anyhow!("Failed to read XLSX file: {}", e))?;

    let sheet_names: Vec<String> = workbook.sheet_names().iter().map(|s| s.to_lowercase()).collect();
    if !sheet_names.contains(&String::from("examiners")) && !sheet_names.contains(&String::from("candidates")) {
        return Err(AppError::from(anyhow!("Cannot find both examiners and candidates sheets")));
    }

    let mut can_time = false;
    let mut new_examiners: Vec<ExaminerExcel> = vec![];
    let mut new_candidates: Vec<CandidateExcel> = vec![];
    for (index, (sheet_name, sheet_data)) in workbook.worksheets().iter().enumerate() {
        println!("Index: {:?}\nSheet Name: {:?}\nSheet Data: {:?}", index, sheet_name, sheet_data);

        let headers: Vec<String> = sheet_data.rows()
            .next()
            .ok_or(anyhow!("Empty spreadsheet"))?
            .iter()
            .map(|cell| cell.get_string().unwrap_or("").to_lowercase())
            .collect();

        let mut header_indices = std::collections::HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            header_indices.insert(header.as_str(), i);
        }

        match sheet_name.to_lowercase().as_str() {
            "examiners" => {
                let required_headers: HashSet<&str> = REQUIRED_EXAMINER_HEADERS.iter().copied().collect();
                let header_set: HashSet<&str> = headers.iter().map(|s| s.as_str()).collect();
                if !required_headers.is_subset(&header_set) {
                    return Err(anyhow!("Missing required headers"))?;
                }

                let mut header_indices = std::collections::HashMap::new();
                for (i, header) in headers.iter().enumerate() {
                    header_indices.insert(header.as_str(), i);
                }

                for (row_index, row) in sheet_data.rows().skip(1).enumerate() {
                    let examiner = ExaminerExcel {
                        first_name: row[header_indices["first_name"]].get_string().ok_or_else(|| { anyhow!( "Missing first_name at row {}", row_index + 2 ) })?.to_string(),
                        last_name: row[header_indices["last_name"]].get_string().ok_or_else(|| { anyhow!( "Missing last_name at row {}", row_index + 2 ) })?.to_string(),
                        shortcode: row[header_indices["shortcode"]].get_string().ok_or_else(|| { anyhow!( "Missing shortcode at row {}", row_index + 2 ) })?.to_string().to_lowercase(),
                        female: get_bool(&row[header_indices["female"]], row_index, "female")?,
                        am: get_bool(&row[header_indices["am"]], row_index, "am")?,
                        pm: get_bool(&row[header_indices["pm"]], row_index, "pm")?
                    };
                    new_examiners.push(examiner);
                }
            },
            "candidates" => {
                let required_headers: HashSet<&str> = REQUIRED_CANDIDATE_HEADERS.iter().copied().collect();
                let header_set: HashSet<&str> = headers.iter().map(|s| s.as_str()).collect();
                if !required_headers.is_subset(&header_set) {
                    return Err(anyhow!("Missing required headers"))?;
                }

                if header_indices.contains_key("am") && header_indices.contains_key("pm") {
                    can_time = true;
                    for (row_index, row) in sheet_data.rows().skip(1).enumerate() {
                        let candidate = CandidateExcel {
                            first_name: row[header_indices["first_name"]].get_string().ok_or_else(|| { anyhow!( "Missing first_name at row {}", row_index + 2 ) })?.to_string(),
                            last_name: row[header_indices["last_name"]].get_string().ok_or_else(|| { anyhow!( "Missing last_name at row {}", row_index + 2 ) })?.to_string(),
                            shortcode: row[header_indices["shortcode"]].get_string().ok_or_else(|| { anyhow!( "Missing shortcode at row {}", row_index + 2 ) })?.to_string().to_lowercase(),
                            female_only: get_bool(&row[header_indices["female_only"]], row_index, "female_only")?,
                            partner_pref: get_pref(&row[header_indices["partner_pref"]], row_index, "partner_pref")?,
                            am: get_option_bool(&row[header_indices["am"]], row_index, "am")?,
                            pm: get_option_bool(&row[header_indices["am"]], row_index, "am")?
                        };
                        new_candidates.push(candidate);
                    }
                } else {
                    for (row_index, row) in sheet_data.rows().skip(1).enumerate() {
                        let candidate = CandidateExcel {
                            first_name: row[header_indices["first_name"]].get_string().ok_or_else(|| { anyhow!( "Missing first_name at row {}", row_index + 2 ) })?.to_string(),
                            last_name: row[header_indices["last_name"]].get_string().ok_or_else(|| { anyhow!( "Missing last_name at row {}", row_index + 2 ) })?.to_string(),
                            shortcode: row[header_indices["shortcode"]].get_string().ok_or_else(|| { anyhow!( "Missing shortcode at row {}", row_index + 2 ) })?.to_string().to_lowercase(),
                            female_only: get_bool(&row[header_indices["female_only"]], row_index, "female_only")?,
                            partner_pref: get_pref(&row[header_indices["partner_pref"]], row_index, "partner_pref")?,
                            am: None,
                            pm: None,
                        };
                        new_candidates.push(candidate);
                    }
                }
            },
            _ => return Err(AppError::from(anyhow!("Cannot match sheet name with 'candidates' or 'examiners'")))
        }
    }

    println!("Session_ID: {:?}", session_id);
    let mut transaction = pool.begin().await.with_context(|| "Unable to create a transaction in database")?;
    for c in new_candidates.into_iter() {
        if can_time {
            let _ = sqlx::query!(
                r#"
                INSERT INTO people.candidates (
                    session_id,
                    first_name,
                    last_name,
                    shortcode,
                    female_only,
                    partner_pref,
                    am,
                    pm,
                    checked_in
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                &session_id,
                c.first_name,
                c.last_name,
                c.shortcode,
                c.female_only,
                c.partner_pref,
                c.am,
                c.pm,
                false
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| anyhow!("Failed to insert candidate from excel: {}", err))?;
        } else {
            let _ = sqlx::query!(
                r#"
                INSERT INTO people.candidates (
                    session_id,
                    first_name,
                    last_name,
                    shortcode,
                    female_only,
                    partner_pref,
                    checked_in
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                &session_id,
                c.first_name,
                c.last_name,
                c.shortcode,
                c.female_only,
                c.partner_pref,
                false
            )
            .execute(&mut *transaction)
            .await
            .map_err(|err| anyhow!("Failed to insert candidate from excel: {}", err))?;
        }
    }
    for e in new_examiners.into_iter() {
        let _ = sqlx::query!(
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
            &session_id,
            e.first_name,
            e.last_name,
            e.shortcode,
            e.female,
            e.am,
            e.pm,
            false
        )
        .execute(&mut *transaction)
        .await
        .map_err(|err| anyhow!("Failed to insert candidate from excel: {}", err))?;
    }

    sqlx::query!("UPDATE records.sessions SET status = 'prep' WHERE id = $1", &session_id).execute(&mut *transaction).await.map_err(|err| anyhow!("Failed to change session upload status: {}", err))?;
    transaction.commit().await.with_context(|| format!("Rolled back successful. Transaction failed to commit"))?;

    Ok(StatusCode::CREATED.into_response())
}
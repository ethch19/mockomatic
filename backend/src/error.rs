use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.to_string()),
        )
        .into_response()
    }
}


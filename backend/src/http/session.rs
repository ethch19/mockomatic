use axum::{response::IntoResponse, routing::get, Extension};
use super::{users::AccessClaims, AppState};
use tracing::trace;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/test", get(test_function))
}

async fn test_function(
    Extension(user): Extension<AccessClaims>
) -> impl IntoResponse {
    trace!("Successful test function, user: {:?}", user);
    "This worked".into_response()
}
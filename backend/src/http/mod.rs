use axum::{middleware::from_fn_with_state, Router, extract::FromRef};
use anyhow::Result;
use axum_extra::extract::cookie::Key;

mod users;
mod default;
mod sessions;
mod pg_interval;

use crate::http::users::mid_jwt_auth;

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

pub fn router_app(db: sqlx::PgPool) -> Router {
    let secret = dotenvy::var("cookie_secret");
    let app_state = match secret {
        Ok(sec) => AppState { key: Key::from(sec.as_bytes()), db },
        Err(_) => AppState { key: Key::generate(), db },
    };
    let v1_routes = Router::new()
        .nest("/users", users::router())
        .nest("/sessions", sessions::router())
        .layer(from_fn_with_state(app_state.clone(), mid_jwt_auth))
        .nest("/users", users::login_router());
    Router::new()
        .nest("/api/v1", v1_routes)
        .with_state(app_state)
}

pub async fn serve(db: sqlx::PgPool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, router_app(db)).await;
    Ok(())
}
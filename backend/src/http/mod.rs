use axum::{extract::FromRef, http, middleware::from_fn_with_state, Router};
use anyhow::Result;
use axum_extra::extract::cookie::Key;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer
};
use serde::Deserialize;

mod users;
mod default;
mod sessions;
mod pg_interval;
mod option_pg_interval;
mod people;

use crate::http::users::mid_jwt_auth;

#[derive(Debug, Deserialize)]
pub struct SomethingID {
    pub id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SomethingMultipleID {
    pub ids: Vec<uuid::Uuid>,
}

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
        .nest("/people", people::router())
        .layer(from_fn_with_state(app_state.clone(), mid_jwt_auth))
        .nest("/users", users::login_router());
    Router::new()
        .nest("/api/v1", v1_routes)
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_methods([http::Method::GET, http::Method::POST])
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
                .allow_private_network(true)
                .allow_credentials(true)
                .allow_origin(["http://localhost:3000".parse::<http::HeaderValue>().unwrap(),
                "http://127.0.0.1:8080".parse::<http::HeaderValue>().unwrap()])
        )
        .layer(
            TraceLayer::new_for_http()
        )
}

pub async fn serve(db: sqlx::PgPool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    
    axum::serve(listener, router_app(db)).await;
    Ok(())
}
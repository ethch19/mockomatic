use axum::{extract::FromRef, http, middleware::from_fn_with_state, Router};
use anyhow::Result;
use axum_extra::extract::cookie::Key;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer
};
use serde::Deserialize;
use tokio::sync::broadcast;

mod users;
pub mod sessions;
pub mod slots;
pub mod stations;
pub mod circuits;
pub mod runs;
pub mod candidates;
pub mod examiners;
mod allocations;
mod templates;
mod pg_interval;
mod option_pg_interval;
mod default;
mod websocket;

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
pub struct AppState {
    pub db: sqlx::PgPool,
    pub key: Key,
    pub tx: broadcast::Sender<String>,
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
    let (tx, _) = broadcast::channel(100);
    let app_state = match secret {
        Ok(sec) => AppState { key: Key::from(sec.as_bytes()), db, tx },
        Err(_) => AppState { key: Key::generate(), db, tx },
    };
    let v1_routes = Router::new()
        .nest("/users", users::router())
        .nest("/sessions", sessions::router())
        .nest("/examiners", examiners::router())
        .nest("/candidates", candidates::router())
        .nest("/allocations", allocations::router())
        .nest("/templates", templates::router())
        .layer(from_fn_with_state(app_state.clone(), mid_jwt_auth))
        .nest("/users", users::login_router());
    Router::new()
        .nest("/api/v1", v1_routes) //remove /api when deploying
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_methods([http::Method::GET, http::Method::POST])
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
                .allow_private_network(true)
                .allow_credentials(true)
                .allow_origin(["http://localhost:3000".parse::<http::HeaderValue>().unwrap(),
                "http://0.0.0.0:8080".parse::<http::HeaderValue>().unwrap(),
                "http://0.0.0.0:3000".parse::<http::HeaderValue>().unwrap(),
                "http://localhost:8080".parse::<http::HeaderValue>().unwrap()])
        )
        .layer(
            TraceLayer::new_for_http()
        )
}

pub async fn serve(db: sqlx::PgPool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    axum::serve(listener, router_app(db)).await;
    Ok(())
}
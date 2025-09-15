use axum::{
    extract::FromRef,
    http::{
        Method, 
        header::{HeaderValue, HeaderName, ACCEPT, AUTHORIZATION, CONTENT_TYPE}, 
    },
    middleware::from_fn_with_state,
    Router
};
use anyhow::Result;
use axum_extra::extract::cookie::Key;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer
};
use serde::Deserialize;
use tokio::sync::broadcast;

pub mod users;
pub mod sessions;
pub mod slots;
pub mod stations;
pub mod circuits;
pub mod runs;
pub mod candidates;
pub mod examiners;
mod upload;
mod allocations;
mod templates;
mod pg_interval;
mod option_pg_interval;
mod default;
mod websocket;
mod csrf;

use crate::http::{users::jwt_auth_middleware, csrf::csrf_auth_middleware};

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
    let secret = dotenvy::var("cookie_secret"); // needs to be stored in secret manager offsite

    let (tx, _) = broadcast::channel(100);
    let app_state = match secret {
        Ok(sec) => AppState { key: Key::from(sec.as_bytes()), db, tx },
        Err(_) => AppState { key: Key::generate(), db, tx },
    };

    let v1_routes = Router::new()
        .nest("/sessions", sessions::router())
        .nest("/stations", stations::router())
        .nest("/slots", slots::router())
        .nest("/runs", runs::router())
        .nest("/circuits", circuits::router())
        .nest("/examiners", examiners::router())
        .nest("/candidates", candidates::router())
        .nest("/allocations", allocations::router())
        .nest("/templates", templates::router())
        .nest("/files", upload::router())
        .nest("/users", users::router()) //l they can login without jwt tokens, perhaps i should implement pre-session auth
        .layer(from_fn_with_state(app_state.clone(), csrf_auth_middleware))
        .layer(from_fn_with_state(app_state.clone(), jwt_auth_middleware))
        .nest("/users", users::login_router());

    Router::new()
        .nest("/api/v1", v1_routes) //remove /api when deploying
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([ACCEPT, CONTENT_TYPE, AUTHORIZATION, HeaderName::from_static("x-csrf-token")])
                .allow_private_network(true)
                .allow_credentials(true)
                .allow_origin(["http://localhost:3000".parse::<HeaderValue>().unwrap(),
                "http://0.0.0.0:8080".parse::<HeaderValue>().unwrap(),
                "http://0.0.0.0:3000".parse::<HeaderValue>().unwrap(),
                "http://localhost:8080".parse::<HeaderValue>().unwrap()])
        )
        .layer(
            TraceLayer::new_for_http()
        )
}
        

pub async fn serve(db: sqlx::PgPool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    let _ = axum::serve(listener, router_app(db)).await;
    Ok(())
}
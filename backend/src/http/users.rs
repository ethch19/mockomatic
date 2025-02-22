use std::collections::HashMap;
use axum::{
    extract::{Json, Request, State},
    http::StatusCode, middleware::Next, response::{IntoResponse, Response}, routing::{get, post},
    debug_middleware
};
use axum_extra::{
    extract::{cookie::Cookie, PrivateCookieJar}, typed_header::{TypedHeader, TypedHeaderRejection}
};
use cookie::Key;
use headers::{Authorization, authorization::Bearer};
use anyhow::{Context, anyhow};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use tracing::{warn, instrument, trace};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use time::{Duration, OffsetDateTime};
use rand::Rng;

use crate::error::AppError;

use super::AppState;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/create", post(User::create))
}

pub fn login_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/login", post(User::login))
        .route("/refresh", get(User::refresh_token))
        .route("/validate", get(User::validate_token))
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub admin: bool,
    pub created_at: OffsetDateTime,
    pub last_login: Option<OffsetDateTime>,
    pub jti: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String, // subject (username)
    pub exp: i64, // expiration
    pub id: uuid::Uuid,
    pub admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String, // subject (username)
    pub exp: i64, // expiration
    pub id: uuid::Uuid,
    pub jti: uuid::Uuid,
}

#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer ".to_string(),
        }
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

const ACCESS_KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = dotenvy::var("ACCESS_JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

const REFRESH_KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = dotenvy::var("REFRESH_JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

async fn generate_token(
    selected_user: &User,
    pool: &sqlx::PgPool
) -> Result<HashMap<String, String>, AppError> {
    let mut tokens: HashMap<String, String> = HashMap::new();
    let expiration = OffsetDateTime::now_utc()
        .checked_add(Duration::minutes(1))
        .expect("Overflow occurred")
        .unix_timestamp();

    let claims = AccessClaims {
        sub: selected_user.username.clone(),
        exp: expiration,
        id: selected_user.id,
        admin: selected_user.admin,
    };

    let access_token = encode(&Header::default(), &claims, &ACCESS_KEYS.encoding)
        .with_context(|| format!("Failed to encode access JWT"))?;
    tokens.insert("access_token".to_string(), access_token);

    let expiration = OffsetDateTime::now_utc()
    .checked_add(Duration::weeks(4))
    .expect("Overflow occurred")
    .unix_timestamp();

    let jti = crate::http::default::default_uuid();
    let _ = sqlx::query!(
        "UPDATE auth.users SET jti = $1 WHERE id = $2",
        &jti,
        &selected_user.id,
    )
    .execute(&*pool)
    .await
    .with_context(|| format!("Cannot add jti into database"))?;

    let refresh_claims = RefreshClaims {
        sub: selected_user.username.clone(),
        exp: expiration,
        id: selected_user.id.clone(),
        jti,
    };

    let refresh_token = encode(&Header::default(), &refresh_claims, &REFRESH_KEYS.encoding)
        .with_context(|| format!("Cannot make refresh JWT"))?;
    tokens.insert("refresh_token".to_string(), refresh_token);
    Ok(tokens)
}

impl User {
    #[instrument(name = "create_user", level = "TRACE")]
    pub async fn create(
        State(pool): State<sqlx::PgPool>,
        Json(req): Json<UserAuth>,
    ) -> Result<StatusCode, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .with_context(|| format!("Failed to hash password when creating user: {}", req.username))?
            .to_string();
        
        sqlx::query!(
            r#"INSERT INTO auth.users (username, password, admin) VALUES ($1, $2, $3)"#,
            req.username,
            password_hash,
            false)
            .execute(&pool)
            .await
            .with_context(|| format!("Failed to insert new user into database"))?;

        Ok(StatusCode::CREATED)
    }

    pub async fn login(
        State(pool): State<sqlx::PgPool>,
        jar: PrivateCookieJar,
        Json(req): Json<UserAuth>,
    ) -> Result<impl IntoResponse, AppError> {
        if req.username.is_empty() || req.password.is_empty() {
            warn!("Missing credentials when logging in");
            return Err(AppError::from(anyhow!("Missing Credentials when logging in")));
        }
        let selected_user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                username,
                password,
                admin,
                jti,
                created_at AS "created_at: time::OffsetDateTime",
                last_login AS "last_login?: time::OffsetDateTime"
            FROM auth.users WHERE username = $1
            "#,
            &req.username,
        )
        .fetch_one(&pool)
        .await
        .with_context(|| format!("Cannot get user from database when logging in"))?;

        let parsed_hash = PasswordHash::new(&selected_user.password)
            .with_context(|| format!("Cannot create PasswordHash from user's password when logging in"))?;

        if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_ok()
        {
            let tokens = generate_token(&selected_user, &pool).await?;
            let cookie = Cookie::build(("refresh_token", tokens["refresh_token"].clone()))
            //.domain("www.ethanchang.dev")
            .path("/")
            //.secure(true)
            .http_only(true)
            .same_site(axum_csrf::SameSite::None) // change back to STRICT
            .max_age(cookie::time::Duration::weeks(4));

            let jar = jar.add(cookie);

            return Ok((
                StatusCode::OK,
                jar,
                Json(AuthBody::new(tokens["access_token"].clone()))
            ).into_response())
        }
        let rand_sleep = rand::rng()
            .random_range(std::time::Duration::from_millis(100)..=std::time::Duration::from_millis(500));
        tokio::time::sleep(rand_sleep).await;
        Err(AppError::from(anyhow!("Wrong Credentials when logging in")))
    }

    #[instrument(level = "trace", skip_all, fields(token))]
    async fn refresh_token(
        State(pool): State<sqlx::PgPool>,
        jar: PrivateCookieJar,
    ) -> Result<impl IntoResponse, AppError> {
        if let Some(token) = jar.get("refresh_token") {
            let token_data = decode::<RefreshClaims>(
                token.value(),
                &REFRESH_KEYS.decoding,
                &Validation::default(),
            )
            .with_context(|| {
                warn!(name: "token_decoding_error", "Cannot decode token into claims");
                format!("Cannot decode token into claims")
            })?;

            let selected_user = sqlx::query_as!(
                User,
                r#"
                SELECT
                    id,
                    username,
                    password,
                    admin,
                    jti,
                    created_at AS "created_at: time::OffsetDateTime",
                    last_login AS "last_login?: time::OffsetDateTime"
                FROM auth.users WHERE id = $1
                "#,
                &token_data.claims.id
            )
            .fetch_one(&pool)
            .await
            .with_context(|| {
                warn!(name: "db_error", "Cannot fetch corresponding jti from db");
                format!("Cannot fetch corresponding jti from db")
            })?;

            if let Some(jwt_id) = selected_user.jti {
                if jwt_id == token_data.claims.jti {
                    let tokens = generate_token(&selected_user, &pool).await?;
                    let jar = jar.remove(Cookie::from("refresh_token"));

                    let cookie = Cookie::build(("refresh_token", tokens["refresh_token"].clone()))
                    //.domain("www.ethanchang.dev")
                    .path("/")
                    //.secure(true)
                    .http_only(true)
                    .same_site(axum_csrf::SameSite::None) // change back to STRICT
                    .max_age(cookie::time::Duration::weeks(4));

                    let jar = jar.add(cookie);

                    return Ok((
                        StatusCode::OK,
                        jar,
                        Json(AuthBody::new(tokens["access_token"].clone()))
                    ).into_response())
                } else {
                    warn!(name: "exception_error", "Refresh token do not match jti");
                    return Err(AppError::from(anyhow!("Error during token creation")));
                }
            } else {
                warn!(name: "exception_error", "User does not have jti stored in db");
                return Err(AppError::from(anyhow!("Error during token creation")));
            }
        } else {
            warn!(name: "exception_error", "Refresh token not found in cookie");
            Err(AppError::from(anyhow!("Error during token creation")))
        } 
    }

    async fn validate_token(
        header : Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>
    ) -> Result<StatusCode, AppError> {
        if let Ok(TypedHeader(Authorization(bearer))) = header {
            let token_data = decode::<AccessClaims>(
                bearer.token(),
                &ACCESS_KEYS.decoding,
                &Validation::default(),
            );
    
            match token_data {
                Ok(_) => {
                    return Ok(StatusCode::OK);
                }
                Err(e) => {
                    trace!(name: "decoding_error", "Cannot decode access token: {}", e);
                }
            } 
        } else {
            trace!(name: "exception_error", "Authorization header not found");
        }
        Ok(StatusCode::FORBIDDEN)
    }
}

#[debug_middleware]
#[instrument(name = "jwt_auth_middleware", level = "TRACE", skip(header, jar, state, req, next))]
pub async fn mid_jwt_auth(
        header : Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>,
        State(state): State<AppState>,
        jar: PrivateCookieJar<Key>,
        mut req: Request,
        next: Next,
    ) -> Result<Response, AppError> {
    let pool = &state.db;
    if let Ok(TypedHeader(Authorization(bearer))) = header {
        let token_data = decode::<AccessClaims>(
            bearer.token(),
            &ACCESS_KEYS.decoding,
            &Validation::default(),
        );

        match token_data {
            Ok(token) => {
                req.extensions_mut().insert(token.claims);
                return Ok(next.run(req).await);
            }
            Err(e) => {
                trace!(name: "decoding_error", "Cannot decode access token: {}", e);
            }
        } 
    } else {
        trace!(name: "exception_error", "Authorization header not found");
    }

    if let Some(token) = jar.get("refresh_token") {
        trace!("Refresh_token found in cookie");
        let token_data = decode::<RefreshClaims>(
            token.value(),
            &REFRESH_KEYS.decoding,
            &Validation::default(),
        )
        .with_context(|| {
            warn!(name: "token_decoding_error", "Cannot decode token into claims");
            format!("Cannot decode token into claims")
        })?;

        let selected_user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                username,
                password,
                admin,
                jti,
                created_at AS "created_at: time::OffsetDateTime",
                last_login AS "last_login?: time::OffsetDateTime"
            FROM auth.users WHERE id = $1
            "#,
            &token_data.claims.id
        )
        .fetch_one(pool)
        .await
        .with_context(|| {
            warn!(name: "db_error", "Cannot fetch corresponding jti from db");
            format!("Cannot fetch corresponding jti from db")
        })?;

        if let Some(jwt_id) = selected_user.jti {
            if jwt_id == token_data.claims.jti {
                let tokens = generate_token(&selected_user, &pool).await?;
                let jar = jar.remove(Cookie::from("refresh_token"));

                let cookie = Cookie::build(("refresh_token", tokens["refresh_token"].clone()))
                //.domain("www.ethanchang.dev")
                .path("/")
                //.secure(true)
                .http_only(true)
                .same_site(axum_csrf::SameSite::None) // change back to STRICT
                .max_age(cookie::time::Duration::weeks(4));

                let jar = jar.add(cookie);

                let expiration = OffsetDateTime::now_utc()
                    .checked_add(Duration::minutes(1))
                    .expect("Overflow occurred")
                    .unix_timestamp();
                let temp_claim = AccessClaims {
                    sub: selected_user.username, // subject (username)
                    exp: expiration, // expiration
                    id: selected_user.id,
                    admin: selected_user.admin,
                };
                req.extensions_mut().insert(temp_claim);

                let mut response = next.run(req).await;
                response.headers_mut().insert(
                    "Authorization",
                    format!("Bearer {}", tokens["access_token"]).parse().unwrap(),
                );

                return Ok((jar, response).into_response());
            }
        }
    }
    trace!("No refresh_token in cookie");
    Ok(StatusCode::UNAUTHORIZED.into_response())
}
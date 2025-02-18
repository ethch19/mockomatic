use axum::{
    debug_middleware,
    extract::{Json, Request, State},
    http::StatusCode, middleware::Next, response::{IntoResponse, Redirect, Response}, routing::{get, post}
};
use axum_extra::{
    extract::{cookie::Cookie, PrivateCookieJar}, typed_header::{TypedHeader, TypedHeaderRejection}
};
use headers::{Authorization, authorization::Bearer};
use anyhow::{Context, anyhow};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use tracing::{warn, instrument};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use chrono::{DateTime, Duration, Utc};
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
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub admin: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
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
    pub exp: usize, // expiration
    pub id: uuid::Uuid,
    pub admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String, // subject (username)
    pub exp: usize, // expiration
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
                created_at AS "created_at: DateTime<Utc>",
                last_login AS "last_login?: DateTime<Utc>"
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
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(1))
                .expect("valid timestamp")
                .timestamp();

            let claims = AccessClaims {
                sub: selected_user.username.clone(),
                exp: expiration as usize,
                id: selected_user.id,
                admin: selected_user.admin,
            };

            let access_token = encode(&Header::default(), &claims, &ACCESS_KEYS.encoding)
                .with_context(|| format!("Failed to encode access JWT"))?;

            let expiration = Utc::now()
                .checked_add_signed(Duration::weeks(4))
                .expect("valid timestamp")
                .timestamp();
            
            let jti = crate::http::default::default_uuid();
            let _ = sqlx::query!(
                "UPDATE auth.users SET jti = $1 WHERE id = $2",
                &jti,
                &selected_user.id,
            )
            .execute(&pool)
            .await
            .with_context(|| format!("Cannot add jti into database"))?;

            let refresh_claims = RefreshClaims {
                sub: selected_user.username,
                exp: expiration as usize,
                id: selected_user.id,
                jti,
            };

            let refresh_token = encode(&Header::default(), &refresh_claims, &REFRESH_KEYS.encoding)
                .with_context(|| format!("Cannot make refresh JWT"))?;

            let cookie = Cookie::build(("refresh_token", refresh_token))
            //.domain("www.ethanchang.dev")
            .path("/")
            //.secure(true)
            .http_only(true)
            .same_site(axum_csrf::SameSite::Strict)
            .max_age(cookie::time::Duration::weeks(4));

            let jar = jar.add(cookie);

            return Ok((
                StatusCode::OK,
                jar,
                Json(AuthBody::new(access_token))
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
                    created_at AS "created_at: DateTime<Utc>",
                    last_login AS "last_login?: DateTime<Utc>"
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
                    let expiration = Utc::now()
                        .checked_add_signed(Duration::hours(1))
                        .expect("valid timestamp")
                        .timestamp();

                    let claims = AccessClaims {
                        sub: selected_user.username.clone(),
                        exp: expiration as usize,
                        id: selected_user.id,
                        admin: selected_user.admin,
                    };

                    let access_token =
                        encode(&Header::default(), &claims, &ACCESS_KEYS.encoding)
                        .with_context(|| {
                            warn!(name: "token_encoding_error", "Problem creating new access token");
                            format!("Problem creating new acces token") 
                        })?;

                    let expiration = Utc::now()
                        .checked_add_signed(Duration::weeks(4))
                        .expect("valid timestamp")
                        .timestamp();

                    let jti = crate::http::default::default_uuid();
                    let _ = sqlx::query!(
                        "UPDATE auth.users SET jti = $1 WHERE id = $2",
                        jti,
                        selected_user.id
                    )
                    .execute(&pool)
                    .await
                    .with_context(|| {
                        warn!(name: "db_error", "Error when updating jti in db");
                        format!("Error when updating jti in db")
                    })?;

                    let refresh_claims = RefreshClaims {
                        sub: selected_user.username,
                        exp: expiration as usize,
                        id: selected_user.id,
                        jti,
                    };

                    let refresh_token =
                        encode(&Header::default(), &refresh_claims, &REFRESH_KEYS.encoding)
                        .with_context(|| {
                            warn!(name: "token_encoding_error", "Problem when encoding new refresh token");
                            format!("Problem when encdoing new refresh token")
                        })?;

                    let jar = jar.remove(Cookie::from("refresh_token"));

                    let cookie = Cookie::build(("refresh_token", refresh_token))
                    //.domain("www.ethanchang.dev")
                    .path("/")
                    //.secure(true)
                    .http_only(true)
                    .same_site(axum_csrf::SameSite::Strict)
                    .max_age(cookie::time::Duration::weeks(4));

                    let jar = jar.add(cookie);

                    return Ok((
                        StatusCode::OK,
                        jar,
                        Json(AuthBody::new(access_token))
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
}

#[instrument(level = "trace", skip(header, req, next))]
#[axum::debug_middleware]
pub async fn mid_jwt_auth(
        header : Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>,
        mut req: Request,
        next: Next,
    ) -> Result<Response, AppError> {
    match header {
        Ok(TypedHeader(Authorization(bearer))) => {
            let token_data = decode::<AccessClaims>(
                bearer.token(),
                &ACCESS_KEYS.decoding,
                &Validation::default(),
            )
            .map_err(|e| {
                warn!("Failed to decode access token: {}", e);
                AppError::from(anyhow!("Invalid access token"))
            })?;

            req.extensions_mut().insert(token_data.claims);
            return Ok(next.run(req).await);
        }
        Err(_) => {
            warn!(name: "exception_error", "Authorization header not found");
            Err(AppError::from(anyhow!("Missing authorization header")))
        }
    }
}

use std::collections::HashMap;
use axum::{
    extract::{Json, Request, State}, http::StatusCode, middleware::Next, response::{IntoResponse, Response}, routing::{get, post}, Extension
};
use axum_extra::{
    extract::{cookie::Cookie, PrivateCookieJar, CookieJar}, typed_header::{TypedHeader, TypedHeaderRejection}
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
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, TokenData};
use once_cell::sync::Lazy;
use time::{Duration, OffsetDateTime};
use rand::Rng;

use crate::error::AppError;
use super::{AppState, csrf::generate_token, default::default_uuid_v4};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/validate", get(validate_token)) // testing purposes, disable in production
        .route("/logout", get(logout))
}

pub fn login_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/login", post(login))
        .route("/refresh", get(refresh))
}

// == HANDLERS ==
async fn login(
    State(pool): State<sqlx::PgPool>,
    private_jar: PrivateCookieJar,
    cookie_jar: CookieJar,
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
            organisation_id,
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
        let tokens = selected_user.new_tokens(&pool).await?;

        let private_jar = private_jar.add(tokens.0);
        let cookie_jar = cookie_jar.add(tokens.1);

        sqlx::query!(
            r#"UPDATE auth.users SET last_login = $1 WHERE id = $2"#,
            OffsetDateTime::now_utc(),
            &selected_user.id
        )
        .execute(&pool)
        .await
        .with_context(|| format!("Cannot update last login time for user {}", &selected_user.username))?;

        return Ok((
            StatusCode::OK, cookie_jar, private_jar,
            Json(tokens.2)
        ).into_response());
    }
    let rand_sleep = rand::rng()
        .random_range(std::time::Duration::from_millis(100)..=std::time::Duration::from_millis(500));
    tokio::time::sleep(rand_sleep).await;
    Err(AppError::from(anyhow!("Wrong Credentials when logging in")))
}

async fn logout(
    State(pool) : State<sqlx::PgPool>,
    Extension(claims): Extension<AccessClaims>,
    private_jar: PrivateCookieJar,
    cookie_jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    User::remove_jti(&pool, &claims.id).await?;
    
    let private_jar = private_jar.remove(Cookie::from("refresh_token"));
    let cookie_jar = cookie_jar.remove(Cookie::from("csrf_token"));
    let (refresh_cookie, csrf_cookie) = removal_cookies();
    let private_jar = private_jar.add(refresh_cookie);
    let cookie_jar = cookie_jar.add(csrf_cookie);

    Ok((StatusCode::OK, cookie_jar, private_jar).into_response())
}

async fn refresh(
    State(pool): State<sqlx::PgPool>,
    private_jar: PrivateCookieJar,
    cookie_jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    if let Some(token) = private_jar.get("refresh_token") {
        let token_data = decode_token::<RefreshClaims>(token.value(), &REFRESH_KEYS.decoding)?;
        let selected_user = User::check_jti(&pool, &token_data.claims).await?;
        let new_tokens = selected_user.new_tokens(&pool).await?;
        let private_jar = private_jar.remove(Cookie::from("refresh_token"));
        let cookie_jar = cookie_jar.remove(Cookie::from("csrf_token"));
        let private_jar = private_jar.add(new_tokens.0);
        let cookie_jar = cookie_jar.add(new_tokens.1);
        return Ok((StatusCode::OK, cookie_jar, private_jar, Json(new_tokens.2)).into_response())
    } else {
        warn!(name: "exception_error", "Refresh token not found in cookies");
        return Err(AppError::from(anyhow!("Missing Credentials")));
    }
}


async fn validate_token( // testing purposes
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

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub organisation_id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub admin: bool,
    pub created_at: OffsetDateTime,
    pub last_login: Option<OffsetDateTime>,
    pub jti: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserPayLoad {
    pub id: uuid::Uuid,
    pub organisation_id: uuid::Uuid,
    pub username: String,
    pub admin: bool,
    pub created_at: OffsetDateTime,
    pub last_login: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreation {
    pub username: String,
    pub password: String,
    pub organisation_id: uuid::Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Organisation {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessClaims {
    pub sub: String, // subject (username)
    pub exp: i64, // expiration
    pub id: uuid::Uuid,
    pub organisation_id: uuid::Uuid,
    pub jti: uuid::Uuid, // different to refresh jti, used for CSRF, meant for shortlived access
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RefreshClaims {
    pub sub: String, // subject (username)
    pub exp: i64, // expiration
    pub id: uuid::Uuid,
    pub jti: uuid::Uuid, // uuid v4 jwt id that lasts as long as the refresh token (ie session), used in CSRF as well
}

#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
    username: String,
    role: String,
    organisation: String,
}

impl AuthBody { // Passed to client via JSON, for access token ONLY
    fn new(access_token: String, username: String, role: String, organisation: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer ".to_string(), // token in Authorization header
            username,
            role,
            organisation,
        }
    }
}

// -- REFACTOR --
// secrets/keys should be stored in a secure vault in production
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

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| {
            warn!(name: "password_hash_error", "Cannot hash password");
            AppError::from(anyhow!("Failed to hash password"))
        })?
        .to_string();
    Ok(password_hash)
}

// wrapper function with my own error handling
fn decode_token<T>(
    token: &str,
    key: &DecodingKey,
) -> Result<TokenData<T>, AppError>
where
    T: serde::de::DeserializeOwned,
{
    decode::<T>(token, key, &Validation::default())
        .map_err(|e| {
            warn!(name: "token_decoding_error", "Cannot decode token into claims");
            trace!(name: "token_decoding_error_details", "Error details: {}", e);
            AppError::from(anyhow!("Token error"))
        })
}

fn encode_token<T>(
    claims: &T,
    key: &EncodingKey,
) -> Result<String, AppError> 
where
    T: serde::ser::Serialize,
{
    encode(&Header::default(), claims, key)
        .map_err(|_| {
            warn!(name: "token_encoding_error", "Cannot encode token into claims");
            AppError::from(anyhow!("Token error"))
        })
}

/// HashMap<AccessToken, RefreshToken>
fn create_tokens(
    user_id: uuid::Uuid,
    organisation_id: uuid::Uuid,
    username: String,
    access_jti: uuid::Uuid,
    refresh_jti: uuid::Uuid,
) -> Result<(HashMap<String, String>, AccessClaims, RefreshClaims), AppError> {
    let mut tokens: HashMap<String, String> = HashMap::new();

    let expiration = OffsetDateTime::now_utc()
        .checked_add(Duration::minutes(15)) // in reality, maybe 5 minutes in production?
        .expect("Overflow occurred")
        .unix_timestamp();
    let access_claims = AccessClaims {
        sub: username.clone(),
        exp: expiration,
        id: user_id,
        organisation_id,
        jti: access_jti,
    };
    tokens.insert(
        String::from("access_token"),
        encode_token(&access_claims, &ACCESS_KEYS.encoding)?
    );

    let expiration = OffsetDateTime::now_utc()
        .checked_add(Duration::weeks(4))
        .expect("Overflow occurred")
        .unix_timestamp();
    let refresh_claims = RefreshClaims {
        sub: username.clone(),
        exp: expiration,
        id: user_id,
        jti: refresh_jti,
    };
    tokens.insert(
        String::from("refresh_token"),
        encode_token(&refresh_claims, &REFRESH_KEYS.encoding)?
    );
    Ok((tokens, access_claims, refresh_claims))
}

/// refresh_token, csrf_token
fn build_cookies(
    refresh_token: String,
    csrf_token: String,
) -> (Cookie<'static>, Cookie<'static>) {
    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
    // -- PRODUCTION --
    // change domain to amsa.co.uk
    // add secure flags
        //.domain("www.ethanchang.dev")
        .path("/")
        //.secure(true)
        .http_only(true)
        .same_site(cookie::SameSite::Lax) // ?Lax to allow links for redirects
        .max_age(cookie::time::Duration::weeks(4))
        .build();

    let csrf_cookie = Cookie::build(("csrf_token", csrf_token))
        //.domain("www.ethanchang.dev")
        .path("/")
        //.secure(true)
        .http_only(false) // CSRF token needs to be accessible via JavaScript
        .same_site(cookie::SameSite::Lax)
        .build();

    (refresh_cookie, csrf_cookie)
}

/// refresh_cookie, csrf_cookie
fn removal_cookies() -> (Cookie<'static>, Cookie<'static>) {
    let refresh_cookie = Cookie::build(("refresh_token", ""))
    // -- PRODUCTION --
    // change domain to amsa.co.uk
    // add secure flags
        //.domain("www.ethanchang.dev")
        .path("/")
        //.secure(true)
        .http_only(true)
        .same_site(cookie::SameSite::Lax) // ?Lax to allow links for redirects
        .removal()
        .build();

    let csrf_cookie = Cookie::build(("csrf_token", ""))
        //.domain("www.ethanchang.dev")
        .path("/")
        //.secure(true)
        .http_only(false) // CSRF token needs to be accessible via JavaScript
        .same_site(cookie::SameSite::Lax)
        .removal()
        .build();

    (refresh_cookie, csrf_cookie)
}

impl Organisation {
    pub async fn create(
        pool: &sqlx::PgPool,
        name: &String,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO auth.organisations (name) VALUES ($1)"#,
            &name)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to insert new organisation into database"))?;

        Ok(())
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM auth.organisations WHERE id = $1"#,
            &id
        )
        .execute(pool)
        .await
        .with_context(|| format!("Failed to delete organisation {} from database", id))?;
        
        Ok(())
    }

    pub async fn get_id(
        pool: &sqlx::PgPool,
        name: &String,
    ) -> Result<uuid::Uuid, AppError> {
        let organisation = sqlx::query!(
            r#"SELECT id FROM auth.organisations WHERE name = $1"#,
            &name
        )
        .fetch_one(pool)
        .await
        .with_context(|| format!("Failed to fetch organisation {} from database", name))?;

        Ok(organisation.id)
    }

    pub async fn get_name(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<String, AppError> {
        let organisation = sqlx::query!(
            r#"SELECT name FROM auth.organisations WHERE id = $1"#,
            &id
        )
        .fetch_one(pool)
        .await
        .with_context(|| format!("Failed to fetch organisation {} from database", id))?;

        Ok(organisation.name)
    }

    pub async fn get_all(
        pool: &sqlx::PgPool
    ) -> Result<Vec<Organisation>, AppError> {
        let organisations = sqlx::query_as!(
            Organisation,
            r#"
            SELECT id, name FROM auth.organisations
            "#
        )
        .fetch_all(pool)
        .await
        .with_context(|| format!("Failed to fetch all organisations from database"))?;

        Ok(organisations)
    }
}

impl User {
    pub async fn create(
        pool: &sqlx::PgPool,
        req: &UserCreation,
        admin: &bool,
    ) -> Result<(), AppError> {
        // -- REFACTOR --
        // check that password meets requirements
        if req.username.is_empty() || req.password.is_empty() || req.organisation_id.is_nil() {
            return Err(AppError::from(anyhow!("Username or password cannot be empty")));
        }
        let password_hash = hash_password(&req.password)?;

        sqlx::query!(
            r#"INSERT INTO auth.users (username, password, admin, organisation_id) VALUES ($1, $2, $3, $4)"#,
            &req.username,
            &password_hash,
            &admin,
            &req.organisation_id
        )
        .execute(pool)
        .await
            .with_context(|| format!("Failed to insert new user into database"))?;

        Ok(())
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        username: &String,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM auth.users WHERE username = $1"#,
            &username
        )
        .execute(pool)
        .await
        .with_context(|| format!("Failed to delete user {} from database", username))?;
        
        Ok(())
    }

    pub async fn toggle_admin(
        pool: &sqlx::PgPool,
        username: &String,
    ) -> Result<bool, AppError> {
        let new_status = sqlx::query!(
            r#"UPDATE auth.users SET admin = NOT admin WHERE username = $1 RETURNING admin"#,
            &username
        )
        .fetch_one(pool)
        .await
        .with_context(|| format!("Failed to toggle admin status for user {}", username))?;
        
        Ok(new_status.admin)
    }

    pub async fn get_all(pool: &sqlx::PgPool) -> Result<Vec<UserPayLoad>, AppError> {
        let users = sqlx::query_as!(
            UserPayLoad,
            r#"
            SELECT id, organisation_id, username, admin, created_at, last_login FROM auth.users
            "#
        )
        .fetch_all(pool)
        .await
        .with_context(|| format!("Failed to fetch all users from database"))?;

        Ok(users)
    }

    pub async fn is_admin(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<bool, AppError> {
        let is_admin = sqlx::query!(
            r#"SELECT admin FROM auth.users WHERE id = $1"#,
            id
        )
        .fetch_one(pool)
        .await
        .with_context(|| format!("Failed to check if user {} is admin", id))?;

        Ok(is_admin.admin)
    }

    async fn check_jti(
        pool: &sqlx::PgPool,
        token_data: &RefreshClaims,
    ) -> Result<User, AppError> {
        let selected_user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                organisation_id,
                username,
                password,
                admin,
                jti,
                created_at AS "created_at: time::OffsetDateTime",
                last_login AS "last_login?: time::OffsetDateTime"
            FROM auth.users WHERE id = $1
            "#,
            &token_data.id
        )
        .fetch_one(pool)
        .await
        .with_context(|| {
            warn!(name: "db_error", "Cannot fetch corresponding jti from db");
            format!("Cannot fetch corresponding jti from db")
        })?;

        if let Some(jwt_id) = selected_user.jti {
            if jwt_id == token_data.jti {
                return Ok(selected_user);
            }
        }
        Err(AppError::from(anyhow!("Refresh token does not match jti")))
    }

    async fn remove_jti(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query!(
            "UPDATE auth.users SET jti = NULL WHERE id = $1",
            id,
        )
        .execute(&*pool)
        .await
        .with_context(|| format!("Cannot remove jti from database"))?;

        Ok(())
    }

    /// refresh cookie, csrf cookie, auth body, access claims
    #[instrument(level = "trace", skip_all)]
    async fn new_tokens(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<(Cookie<'static>, Cookie<'static>, AuthBody, AccessClaims), AppError> {
        let tokens =  self.generate_jwt_tokens(&pool).await?;
        let csrf_token = generate_token(tokens.1.jti)?;

        println!("csrf_token: {}", csrf_token);

        let (refresh_cookie, csrf_cookie) = build_cookies(
            tokens.0["refresh_token"].clone(),
            csrf_token,
        );

        let role = if self.admin { "admin".to_string() } else { "user".to_string() };
        let organisation = Organisation::get_name(&pool, &self.organisation_id).await?;
        Ok((refresh_cookie, csrf_cookie, AuthBody::new(tokens.0["access_token"].clone(), self.username.clone(), role, organisation), tokens.1))
    }

    /// HashMap<AccessToken, RefreshToken>
    async fn generate_jwt_tokens(
        &self,
        pool: &sqlx::PgPool
    ) -> Result<(HashMap<String, String>, AccessClaims), AppError> {
        let access_jti = default_uuid_v4();
        let refresh_jti = default_uuid_v4();
        let _ = sqlx::query!(
            "UPDATE auth.users SET jti = $1 WHERE id = $2",
            refresh_jti,
            self.id,
            )
            .execute(&*pool)
            .await
            .with_context(|| format!("Cannot add jti into database"))?;

        let tokens = create_tokens(
            self.id,
            self.organisation_id,
            self.username.clone(),
            access_jti,
            refresh_jti,
        )?;

        Ok((tokens.0, tokens.1))
    }
}

#[instrument(name = "jwt_auth_middleware", level = "TRACE", skip_all)]
pub async fn jwt_auth_middleware(
        header : Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>,
        State(state): State<AppState>,
        jar: PrivateCookieJar<Key>,
        mut req: Request,
        next: Next,
    ) -> Result<Response, AppError> {
    let pool = &state.db;
    if let Ok(TypedHeader(Authorization(bearer))) = header {
        let token_data = decode_token::<AccessClaims>(bearer.token(), &ACCESS_KEYS.decoding);

        match token_data {
            Ok(token) => {
                // extract access token claims and insert them into the request extensions

                // -- REFACTOR --
                // Redis to create blacklist of withdrawn non-expired tokens +  cross check
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

    // if no access token is found, check for refresh token
    if let Some(token) = jar.get("refresh_token") {
        let token_data = decode_token::<RefreshClaims>(token.value(), &REFRESH_KEYS.decoding)?;
        let selected_user = User::check_jti(&pool, &token_data.claims).await?;
        let new_tokens = selected_user.new_tokens(&pool).await?;
        let jar = jar.remove(Cookie::from("refresh_token"));
        let jar = jar.remove(Cookie::from("csrf_token"));
        let jar = jar.add(new_tokens.0);
        let jar = jar.add(new_tokens.1);
        req.extensions_mut().insert(new_tokens.3);
        let mut response = next.run(req).await; // allows the request to continue first
        response.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", new_tokens.2.access_token).parse().unwrap()
        );
        return Ok((jar, response).into_response());
    } else {
        warn!(name: "exception_error", "Refresh token not found in cookies");
        return Err(AppError::from(anyhow!("Missing Credentials")));
    }
}

// == TESTS ==
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();
        assert!(!hashed.is_empty(), "Hashed password should not be empty");

        let parsed_hash = PasswordHash::new(&hashed).unwrap();
        assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok());
    }

    #[test]
    fn test_jwt_tokens() {
        let user_id = crate::http::default::default_uuid_v4();
        let organisation_id = crate::http::default::default_uuid_v4();
        let access_jti = default_uuid_v4();
        let refresh_jti = default_uuid_v4();
        let tokens = create_tokens(
            user_id,
            organisation_id,
            String::from("test_user"),
            access_jti,
            refresh_jti,
        ).unwrap();

        let decoded_access = decode_token::<AccessClaims>(&tokens.0["access_token"], &ACCESS_KEYS.decoding).unwrap();
        let decoded_refresh = decode_token::<RefreshClaims>(&tokens.0["refresh_token"], &REFRESH_KEYS.decoding).unwrap();

        assert_eq!(tokens.1, decoded_access.claims);
        assert_eq!(tokens.2, decoded_refresh.claims);
    }
}
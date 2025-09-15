use hmac::{Mac, Hmac, digest::generic_array::GenericArray};
use sha2::Sha256;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use once_cell::sync::Lazy;
use anyhow::anyhow;
use axum::{debug_middleware, extract::{Extension, Request}, http::HeaderMap, middleware::Next, response::Response};
use axum_extra::extract::CookieJar;
use tracing::{instrument, trace};

use crate::{error::AppError, http::{users::AccessClaims}};

type HmacSha256 = Hmac<Sha256>;

const CSRF_KEY: Lazy<String> = Lazy::new(|| { // should be replaced with external secret management
    return dotenvy::var("CSRF_SECRET").expect("CSRF_SECRET must be set");
});

// -- REFACTOR --
// make all session-based or JWT id into v4, database use v7
pub fn generate_token(session_id: uuid::Uuid) -> Result<String, AppError> { 
    let mut rng = ChaCha20Rng::from_os_rng();
    let mut key_bytes = [0u8; 32]; // 256 bits, random value
    rng.fill(&mut key_bytes);

    let message: String = format!("{}!{}!{}!{}", session_id.to_u128_le(), session_id, key_bytes.len(), hex::encode(key_bytes));
    let mut mac = HmacSha256::new_from_slice(CSRF_KEY.as_bytes()).map_err(|_| AppError::from(anyhow!("Failed token generation")))?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    let token = format!("{}.{}", hex::encode(code_bytes), hex::encode(key_bytes));

    Ok(token)
}

/// session_id from access claims
fn verify_token(request_token: &str, cookie_token: &str, session_id: uuid::Uuid) -> Result<(), AppError> {
    println!("Verifying CSRF token: request_token={}, cookie_token={})", request_token, cookie_token);
    let request_parts: Vec<&str> = request_token.split('.').collect();
    let cookie_parts: Vec<&str> = cookie_token.split('.').collect();
    if request_parts.len() != 2 || cookie_parts.len() != 2 {
        trace!("Request or cookie token format is invalid");
        return Err(AppError::from(anyhow!("Auth Error")));
    }

    let key_bytes = hex::decode(request_parts[1]).map_err(|_| AppError::from(anyhow!("Auth Error")))?;
    let request_message = format!("{}!{}!{}!{}", session_id.to_u128_le(), session_id, key_bytes.len(), request_parts[1]);
    let mut mac = HmacSha256::new_from_slice(CSRF_KEY.as_bytes()).map_err(|_| AppError::from(anyhow!("Auth Error")))?;
    mac.update(request_message.as_bytes());
    trace!("Mac is fine");
    let cookie_result = hex::decode(cookie_parts[0]).map_err(|_| AppError::from(anyhow!("Auth Error")))?;
    trace!("cookie result can be decoded");
    mac.verify(GenericArray::from_slice(&cookie_result)).map_err(|_| AppError::from(anyhow!("Auth Error")))
}

#[debug_middleware]
#[instrument(name = "csrf_auth_middleware", level = "TRACE", skip_all)]
pub async fn csrf_auth_middleware(
    headers: HeaderMap,
    Extension(claims): Extension<AccessClaims>,
    jar: CookieJar,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // -- REFACTOR --
    // currently, csrf cookie is not encrypted or signed, so it is readable by the client
    // as PrivateCookieJar's encryption is handled by 'cookie' crate, which uses AES-GCM with a 256-bit key and 96-bit nonce as of now
    // this gives two ways (given cookie's encrpytion doesn't change for a while):
    // (1) implement this decryption explicitly
    // (2) store CSRF token unencrypted (CURRENTLY USED)
    // it is probably computationally useless to encrypt the CSRF token again, since it is already signed by HMAC
    // we don't care if it is readable cause it is just a random UUID that refreshes every 15 minutes (same time as access JWT)
    if let Some(token) = headers.get("x-csrf-token") {
        let request_token = token.to_str().map_err(|_| AppError::from(anyhow!("Auth Error")))?.to_owned();
        if let Some(cookie) = jar.get("csrf_token") {
            let cookie_token = cookie.value();
            println!("jti: {}", claims.jti);
            if verify_token(&request_token, cookie_token, claims.jti).is_ok() {
                return Ok(next.run(req).await);
            } else {
                trace!("CSRF token verification failed");
            }
        } else {
            trace!("CSRF token not found in cookies");
        }
    } else {
        trace!("X-CSRF-Token not found in headers");
    }
    Err(AppError::from(anyhow!("Auth Error")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token() {
        let session_id = uuid::Uuid::new_v4();

        let token = generate_token(session_id).expect("Token generation failed");
        assert!(verify_token(&token, &token.clone(), session_id).is_ok());
    }
}
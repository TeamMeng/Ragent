//! JWT token generation/validation and Argon2 password hashing.

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::error::{AppError, AppResult};

// ─── Claims ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

// ─── Password hashing ─────────────────────────────────

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash).map_err(|e| AppError::Internal(e.into()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// ─── Token operations ─────────────────────────────────

pub fn generate_tokens(user_id: Uuid, username: &str, config: &JwtConfig) -> AppResult<TokenPair> {
    let now = Utc::now();
    let access_exp = (now + Duration::seconds(config.access_token_expiry_secs)).timestamp();
    let refresh_exp = (now + Duration::seconds(config.refresh_token_expiry_secs)).timestamp();

    let access_claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: access_exp,
        iat: now.timestamp(),
        token_type: "access".into(),
    };

    let refresh_claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: refresh_exp,
        iat: now.timestamp(),
        token_type: "refresh".into(),
    };

    let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());

    let access_token = encode(&Header::default(), &access_claims, &encoding_key)?;
    let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_key)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: config.access_token_expiry_secs,
    })
}

pub fn validate_access_token(token: &str, config: &JwtConfig) -> AppResult<Claims> {
    let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
    let token_data = decode::<Claims>(token, &decoding_key, &Validation::default())?;

    if token_data.claims.token_type != "access" {
        return Err(AppError::Unauthorized("Not an access token".into()));
    }
    Ok(token_data.claims)
}

pub fn validate_refresh_token(token: &str, config: &JwtConfig) -> AppResult<Claims> {
    let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
    let token_data = decode::<Claims>(token, &decoding_key, &Validation::default())?;

    if token_data.claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("Not a refresh token".into()));
    }
    Ok(token_data.claims)
}

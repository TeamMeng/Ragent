//! Authentication routes: register, login, refresh.

use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use app_core::{
    auth::{generate_tokens, hash_password, validate_refresh_token, verify_password},
    error::{AppError, AppResult},
    models::{CreateUser, User, UserPublic},
};

/// POST /api/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateUser>,
) -> AppResult<(StatusCode, Json<UserPublic>)> {
    // Validate input
    if input.username.len() < 3 {
        return Err(AppError::Validation(
            "Username must be at least 3 characters".into(),
        ));
    }
    if input.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }

    // Hash password
    let password_hash = hash_password(&input.password)?;

    // Insert user
    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (username, email, password_hash, display_name)
           VALUES ($1, $2, $3, $4)
           RETURNING *"#,
    )
    .bind(&input.username)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(&input.display_name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint().is_some() => {
            let msg = if db_err.constraint().unwrap().contains("username") {
                "Username already exists"
            } else {
                "Email already exists"
            };
            AppError::Validation(msg.into())
        }
        _ => AppError::Database(e),
    })?;

    tracing::info!(user_id = %user.id, "User registered");
    Ok((StatusCode::CREATED, Json(user.into())))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let username = body["username"].as_str().unwrap_or("");
    let password = body["password"].as_str().unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Err(AppError::Validation(
            "Username and password required".into(),
        ));
    }

    // Look up user
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = $1 AND is_active = TRUE")
        .bind(username)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

    // Verify password
    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    // Generate tokens
    let tokens = generate_tokens(user.id, &user.username, &state.config.jwt)?;

    tracing::info!(user_id = %user.id, "User logged in");
    Ok(Json(serde_json::json!({
        "user": UserPublic::from(user),
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "expires_in": tokens.expires_in,
    })))
}

/// POST /api/auth/refresh
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let refresh_token = body["refresh_token"].as_str().unwrap_or("");
    if refresh_token.is_empty() {
        return Err(AppError::Unauthorized("Refresh token required".into()));
    }

    let claims = validate_refresh_token(refresh_token, &state.config.jwt)?;

    // Verify user still exists and is active
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1 AND is_active = TRUE")
        .bind(claims.sub)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::Unauthorized("User not found or inactive".into()))?;

    // Issue new token pair
    let tokens = generate_tokens(user.id, &user.username, &state.config.jwt)?;

    Ok(Json(serde_json::json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "expires_in": tokens.expires_in,
    })))
}

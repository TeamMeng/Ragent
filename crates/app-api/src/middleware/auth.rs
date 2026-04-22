//! JWT authentication middleware.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::AppState;

/// Extract user_id from JWT in Authorization header.
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = app_core::auth::validate_access_token(token, &state.config.jwt)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Store user_id in request extensions for handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

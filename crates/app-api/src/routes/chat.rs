//! Chat session and message routes.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use app_core::error::{AppError, AppResult};
use app_core::models::*;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub before: Option<Uuid>,
    pub limit: Option<u32>,
}

/// POST /api/sessions — create a new chat session.
///
/// TODO: Wire auth middleware to extract Claims and set created_by.
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateSession>,
) -> AppResult<(StatusCode, Json<Session>)> {
    let session = sqlx::query_as::<_, Session>(
        r#"INSERT INTO sessions (name, session_type, created_by)
           VALUES ($1, $2, '00000000-0000-0000-0000-000000000000'::uuid)
           RETURNING *"#,
    )
    .bind(input.name.unwrap_or_else(|| "New Chat".into()))
    .bind(input.session_type.unwrap_or(1))
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    tracing::info!(session_id = %session.id, "Session created");
    Ok((StatusCode::CREATED, Json(session)))
}

/// GET /api/sessions — list user's sessions.
///
/// TODO: Filter by current user from auth Claims.
pub async fn list_sessions(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Session>>> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE is_archived = FALSE ORDER BY updated_at DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(sessions))
}

/// POST /api/sessions/:session_id/messages — send a message.
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Json(input): Json<SendMessage>,
) -> AppResult<(StatusCode, Json<MessageOut>)> {
    // Verify session exists
    let _session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Session not found".into()))?;

    let content_type = input.content_type.clone().unwrap_or_else(|| "text".into());

    // Insert message
    let message = sqlx::query_as::<_, Message>(
        r#"INSERT INTO messages (session_id, sender_type, sender_id, content, content_type, token_count)
           VALUES ($1, 0, NULL, $2, $3, 0)
           RETURNING *"#,
    )
    .bind(session_id)
    .bind(&input.content)
    .bind(&content_type)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    // TODO: Trigger agent response via async task queue

    tracing::info!(msg_id = %message.id, session_id = %session_id, "Message sent");
    Ok((StatusCode::CREATED, Json(message.into())))
}

/// GET /api/sessions/:session_id/messages — list messages.
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> AppResult<Json<Vec<MessageOut>>> {
    let limit = pagination.limit.unwrap_or(50).min(100) as i64;

    let messages = sqlx::query_as::<_, Message>(
        r#"SELECT * FROM messages
           WHERE session_id = $1
           ORDER BY created_at DESC
           LIMIT $2"#,
    )
    .bind(session_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let mut out: Vec<MessageOut> = messages.into_iter().map(Into::into).collect();
    out.reverse(); // Chronological order
    Ok(Json(out))
}

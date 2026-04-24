//! Route definitions.

pub mod auth;
pub mod chat;
pub mod ws;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;

/// Create the application router with all routes.
pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Auth
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/refresh", post(auth::refresh))
        // Chat sessions
        .route("/api/sessions", post(chat::create_session))
        .route("/api/sessions", get(chat::list_sessions))
        .route(
            "/api/sessions/{session_id}/messages",
            post(chat::send_message),
        )
        .route(
            "/api/sessions/{session_id}/messages",
            get(chat::list_messages),
        )
        // WebSocket
        .route("/ws", get(ws::ws_handler));

    Router::new()
        .merge(api_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

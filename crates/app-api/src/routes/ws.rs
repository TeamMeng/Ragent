//! WebSocket handler for real-time chat.

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::AppState;
use app_proto::WsEvent;

/// GET /ws — WebSocket upgrade endpoint
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection.
async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket connection established");

    // Wait for auth message first
    let mut user_id: Option<uuid::Uuid> = None;

    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                let text_str = &*text;
                // Try to parse as WsEvent
                match serde_json::from_str::<WsEvent>(text_str) {
                    Ok(event) => {
                        match event {
                            WsEvent::Auth { token } => {
                                match app_core::auth::validate_access_token(
                                    &token,
                                    &state.config.jwt,
                                ) {
                                    Ok(claims) => {
                                        user_id = Some(claims.sub);
                                        let auth_ok = WsEvent::AuthResult {
                                            ok: true,
                                            user_id: Some(claims.sub),
                                            error: None,
                                        };
                                        let _ = socket
                                            .send(Message::Text(
                                                serde_json::to_string(&auth_ok).unwrap().into(),
                                            ))
                                            .await;
                                        info!(user_id = %claims.sub, "WebSocket authenticated");
                                    }
                                    Err(e) => {
                                        let auth_fail = WsEvent::AuthResult {
                                            ok: false,
                                            user_id: None,
                                            error: Some(e.to_string()),
                                        };
                                        let _ = socket
                                            .send(Message::Text(
                                                serde_json::to_string(&auth_fail).unwrap().into(),
                                            ))
                                            .await;
                                        warn!("WebSocket auth failed");
                                    }
                                }
                            }
                            WsEvent::Ping { ts } => {
                                let pong = WsEvent::Pong { ts };
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&pong).unwrap().into(),
                                    ))
                                    .await;
                            }
                            WsEvent::ChatMessage {
                                session_id,
                                content,
                                content_type,
                            } => {
                                if user_id.is_none() {
                                    let err = WsEvent::Error {
                                        code: "UNAUTHORIZED".into(),
                                        message: "Authenticate first".into(),
                                    };
                                    let _ = socket
                                        .send(Message::Text(
                                            serde_json::to_string(&err).unwrap().into(),
                                        ))
                                        .await;
                                    continue;
                                }

                                info!(session_id = %session_id, "Chat message via WebSocket");

                                // TODO: Save message to DB, trigger agent response, stream back
                                // For now, echo back as a system message
                                let reply = WsEvent::Message {
                                    id: uuid::Uuid::new_v4(),
                                    session_id,
                                    sender_type: 2, // system
                                    sender_id: None,
                                    content: format!("[Echo] {}", content),
                                    content_type: content_type.unwrap_or_else(|| "text".into()),
                                    created_at: chrono::Utc::now().to_rfc3339(),
                                };
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&reply).unwrap().into(),
                                    ))
                                    .await;
                            }
                            _ => {
                                warn!("Unhandled WebSocket event type");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                        let err = WsEvent::Error {
                            code: "PARSE_ERROR".into(),
                            message: format!("Invalid message: {}", e),
                        };
                        let _ = socket
                            .send(Message::Text(serde_json::to_string(&err).unwrap().into()))
                            .await;
                    }
                }
            }
            Message::Close(_) => {
                info!("WebSocket connection closed");
                break;
            }
            _ => {}
        }
    }

    info!("WebSocket handler ended");
}

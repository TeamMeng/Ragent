//! WebSocket event definitions shared between client and server.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All events sent over the WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    // ─── Client → Server ─────────────────────────────
    /// Authenticate the WebSocket connection.
    Auth { token: String },

    /// Send a chat message.
    ChatMessage {
        session_id: Uuid,
        content: String,
        content_type: Option<String>,
    },

    /// Request message history.
    History {
        session_id: Uuid,
        before: Option<Uuid>,
        limit: Option<u32>,
    },

    /// Start a new chat session.
    NewSession {
        name: Option<String>,
        session_type: Option<i16>,
    },

    /// Stop the current agent generation.
    StopGeneration,

    /// Client heartbeat.
    Ping { ts: i64 },

    // ─── Server → Client ─────────────────────────────
    /// Auth result.
    AuthResult {
        ok: bool,
        user_id: Option<Uuid>,
        error: Option<String>,
    },

    /// A new message was created.
    Message {
        id: Uuid,
        session_id: Uuid,
        sender_type: i16,
        sender_id: Option<Uuid>,
        content: String,
        content_type: String,
        created_at: String,
    },

    /// Agent is currently generating (streaming).
    StreamStart { session_id: Uuid, agent_id: Uuid },

    /// A chunk of streamed content.
    StreamChunk { session_id: Uuid, chunk: String },

    /// Agent finished generating.
    StreamEnd { session_id: Uuid },

    /// Task status updated.
    TaskUpdate { task_id: Uuid, status: String },

    /// Heartbeat response.
    Pong { ts: i64 },

    /// Generic error.
    Error { code: String, message: String },
}

impl WsEvent {
    /// Create a chat message event.
    pub fn chat(session_id: Uuid, content: impl Into<String>) -> Self {
        Self::ChatMessage {
            session_id,
            content: content.into(),
            content_type: None,
        }
    }
}

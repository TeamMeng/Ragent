//! Database models using sqlx::FromRow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── User ─────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            is_active: u.is_active,
            created_at: u.created_at,
        }
    }
}

// ─── Session ──────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub session_type: i16,
    pub created_by: Uuid,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSession {
    pub name: Option<String>,
    pub session_type: Option<i16>,
    pub agent_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct SessionWithMembers {
    #[serde(flatten)]
    pub session: Session,
    pub members: Vec<MemberInfo>,
}

// ─── Member ───────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MemberInfo {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

// ─── Agent ────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub model: String,
    pub tools: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Message ──────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_type: i16,
    pub sender_id: Option<Uuid>,
    pub content: String,
    pub content_type: String,
    pub token_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessage {
    pub content: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageOut {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_type: i16,
    pub sender_id: Option<Uuid>,
    pub content: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageOut {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            session_id: m.session_id,
            sender_type: m.sender_type,
            sender_id: m.sender_id,
            content: m.content,
            content_type: m.content_type,
            created_at: m.created_at,
        }
    }
}

// ─── Task ─────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub session_id: Uuid,
    pub assigned_to: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i16,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub priority: Option<i16>,
    pub due_at: Option<DateTime<Utc>>,
}

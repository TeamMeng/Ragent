//! # app-core
//!
//! Shared business logic: config, database, error types, models, auth.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod models;

pub use config::AppConfig;
pub use db::create_pool;
pub use error::{AppError, AppResult};

use std::sync::Arc;

use sqlx::sqlite::SqlitePool;

use crate::config::Config;

/// Shared state across all the web controllers
#[derive(Clone)]
pub struct AppState {
    /// Initial web app configuration
    pub config: Arc<Config>,

    /// Connection to the sqlite database
    pub db_pool: SqlitePool
}
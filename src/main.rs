use sqlx::sqlite::SqlitePool;
use clap::Parser;
use tracing_subscriber::prelude::*;

mod config;
mod app_state;
mod http;
mod api;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenv::dotenv()?;

    // Initialize console tracing with its level controlled by environment
    // variable RUST_LOG
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::parse();

    // Connect to sqlite database
    let db_pool = SqlitePool::connect(config.database_url.as_ref()).await?;

    // Automatically apply all the migrations to the database
    sqlx::migrate!()
        .run(&db_pool).await?;

    http::serve(config, db_pool).await
}
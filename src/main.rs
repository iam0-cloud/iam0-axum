use std::sync::Arc;

use app_state::AppState;
use sqlx::sqlite::SqlitePool;
use clap::Parser;
use tracing_subscriber::prelude::*;

mod config;
mod app_state;
mod http;
mod api;

use crate::config::Config;

async fn create_server(is_unit_test: bool) -> anyhow::Result<AppState> {
    // Load .env
    dotenv::dotenv()?;

    // TODO: Make the tracing export to id identified files and maybe do testing over traces
    if !is_unit_test {
        // Initialize console tracing with its level controlled by environment
        // variable RUST_LOG
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Load configuration
    let config = if !is_unit_test { 
        Config::parse() 
    } else {
        // This is needed for testing, since otherwise clap will override the arguments of
        // test binaries (bad)
        Config::parse_from([""])
    };

    // Connect to sqlite database
    let db_pool = SqlitePool::connect(config.database_url.as_ref()).await?;

    // Automatically apply all the migrations to the database
    sqlx::migrate!()
        .run(&db_pool).await?;

    Ok(AppState {
        config: Arc::new(config),
        db_pool
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = create_server(false).await?;

    http::serve(app).await
}
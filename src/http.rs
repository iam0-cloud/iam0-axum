use std::sync::Arc;
use std::net::SocketAddr;

use sqlx::sqlite::SqlitePool;
use axum::routing::*;
use axum::extract::*;

use crate::config::Config;
use crate::app_state::AppState;

pub async fn serve(
    config: Config,
    db_pool: SqlitePool,
) -> anyhow::Result<()> {
    let config = Arc::new(config);
    let state = AppState {
        config: config.clone(),
        db_pool,
    };

    let app = axum::Router::new()
        .merge(crate::api::handlers(state))
        .fallback_service(tower_http::services::ServeDir::new(&config.www_dir));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
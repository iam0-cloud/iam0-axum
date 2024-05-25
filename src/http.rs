use std::net::SocketAddr;

use crate::app_state::AppState;

pub async fn serve(
    app_state: AppState,
) -> anyhow::Result<()> {
    let app = axum::Router::new()
        .merge(crate::api::handlers(app_state.clone()))
        .fallback_service(tower_http::services::ServeDir::new(&app_state.config.www_dir));

    let addr = SocketAddr::from(([127, 0, 0, 1], app_state.config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
use std::sync::Arc;

use axum::response::*;
use axum::routing::*;
use axum::extract::*;

use crate::app_state::AppState;

pub fn handlers(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/api/login", post(zkp_login))
        .route("/oauth/authorization", post(authorization))
        .with_state(state)
}

async fn zkp_login() -> impl IntoResponse {
    tracing::info!("hello");
    todo!()
}

async fn authorization() -> impl IntoResponse {
    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_login() {
    }
}
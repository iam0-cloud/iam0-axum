use axum::extract::*;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Common error type used throughout the http API compatible with anyhow
/// and axum
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("authentication required")]
    Unauthorized,

    #[error("user may not perform that action")]
    Forbidden,

    #[error("resource not found")]
    NotFound,

    /// Automatically return '500 Internal Server Error' on a 'sqlx::Error'
    /// without context for security reasons
    /// 
    /// Note that this are unexpected error, a query error should be handled
    /// gracefully with the other error categories
    #[error("error on database query")]
    Sqlx(#[from] sqlx::Error),

    /// Automatically return '500 Internal Server Error' on a 'anyhow::Error'
    ///
    /// This errors are usually not fatal, this is used for tracing purposes to
    /// get on the traces all the backtrace, like sqlx errors doesn't return context
    /// to the user for security reasons
    #[error("an internal server error occured")]
    Anyhow(#[from] anyhow::Error)
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR 
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Handle tracing before sending response
        match &self {
            Error::Sqlx(ref e) => tracing::error!("sqlx error: {:?}", e),
            Error::Anyhow(ref e) => tracing::error!("generic error: {:?}", e),
            _ => {}
        }

        (
            self.status_code(),
            Json(serde_json::json!({
                "error": self.to_string()
            }))
        ).into_response()
    }
}
// A single enum for all application errors.
//
// Key Rust concepts used here:
//  - `thiserror::Error` derive macro: generates `std::error::Error` impl for free.
//  - `IntoResponse` trait: Axum calls this to turn our error into an HTTP response.
//  - Pattern matching on error variants to choose the right HTTP status code.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;
use thiserror::Error;

/// Every error the application can produce.
#[derive(Debug, Error)]
pub enum AppError {
    /// 404 — resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// 400 — client sent bad data
    #[error("Validation error: {0}")]
    Validation(String),

    /// 409 — e.g. duplicate email
    #[error("Conflict: {0}")]
    Conflict(String),

    /// 500 — database or other infra failure
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// 500 — anything else
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Convert AppError → HTTP Response.
///
/// This is the Rust equivalent of a FastAPI exception handler.
/// Axum will automatically call this when a handler returns `Err(AppError)`.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred".to_string(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": {
                "status":  status.as_u16(),
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

/// Convenience type alias — every handler returns this.
pub type AppResult<T> = Result<T, AppError>;

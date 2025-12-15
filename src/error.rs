use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Ollama error: {0}")]
    Ollama(String),

    #[error("Brave search error: {0}")]
    BraveSearch(String),

    #[error("Page fetch error: {0}")]
    PageFetch(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::SessionNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

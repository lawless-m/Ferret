use axum::{extract::State, Json};
use serde::Serialize;

use crate::session::manager;
use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    ollama: String,
    sessions: usize,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let ollama_status = match state.ollama.check_health().await {
        Ok(true) => "connected",
        Ok(false) => "unhealthy",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        ollama: ollama_status.to_string(),
        sessions: manager::session_count(&state.sessions),
    })
}

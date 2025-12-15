use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::session::manager;
use crate::AppState;

pub async fn clear(
    cookies: CookieJar,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(cookie) = cookies.get("session_id") {
        if let Ok(session_id) = Uuid::parse_str(cookie.value()) {
            manager::clear_session(&state.sessions, session_id);
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert("HX-Trigger", HeaderValue::from_static("chat-cleared"));

    (headers, "OK")
}

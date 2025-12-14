use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use uuid::Uuid;

use crate::session::{manager, Session};
use crate::AppState;

const INDEX_HTML: &str = include_str!("../../templates/index.html");

pub async fn index(cookies: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
    let session_id = cookies
        .get("session_id")
        .and_then(|c| Uuid::parse_str(c.value()).ok())
        .unwrap_or_else(|| {
            let id = Uuid::new_v4();
            state.sessions.insert(id, Session::new(id));
            id
        });

    // Ensure session exists
    manager::get_or_create_session(&state.sessions, session_id);

    let cookie = Cookie::build(("session_id", session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict);

    let cookies = cookies.add(cookie);

    (cookies, Html(INDEX_HTML))
}

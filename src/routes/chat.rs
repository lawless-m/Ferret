use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Form,
};
use axum_extra::extract::CookieJar;
use futures::stream::Stream;
use serde::Deserialize;
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::chat::{handle_chat, StreamEvent};
use crate::error::AppError;
use crate::session::manager;
use crate::AppState;

#[derive(Deserialize)]
pub struct ChatInput {
    pub message: String,
}

pub async fn chat(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(input): Form<ChatInput>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let message = input.message.trim().to_string();

    if message.is_empty() {
        return Err(AppError::InvalidRequest("Message cannot be empty".to_string()));
    }

    let session_id = cookies
        .get("session_id")
        .and_then(|c| Uuid::parse_str(c.value()).ok())
        .ok_or(AppError::SessionNotFound)?;

    let mut session = manager::get_or_create_session(&state.sessions, session_id);

    let (tx, rx) = mpsc::channel::<StreamEvent>(100);

    // Spawn chat handler
    let ollama = state.ollama.clone();
    let tools = state.tools.clone();

    tokio::spawn(async move {
        handle_chat(&ollama, &tools, &mut session, message, tx).await;

        // Update session after handling
        manager::update_session(&state.sessions, session);
    });

    // Convert to SSE stream
    let stream = ReceiverStream::new(rx).map(|event| {
        Ok(Event::default().data(serde_json::to_string(&event).unwrap()))
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

use futures::StreamExt;

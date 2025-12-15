use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod chat;
mod config;
mod error;
mod ollama;
mod routes;
mod session;
mod tools;

use config::AppConfig;
use ollama::OllamaClient;
use session::{create_session_manager, SessionManager};
use tools::ToolExecutor;

#[derive(Clone)]
pub struct AppState {
    pub sessions: SessionManager,
    pub ollama: OllamaClient,
    pub tools: ToolExecutor,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ferret=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env file if present
    if let Err(e) = dotenvy::dotenv() {
        info!("No .env file found or error loading it: {}", e);
    }

    // Load configuration
    let config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            error!("Please ensure BRAVE_API_KEY environment variable is set");
            std::process::exit(1);
        }
    };

    info!("Starting Ferret with configuration:");
    info!("  Ollama URL: {}", config.ollama_url);
    info!("  Ollama Model: {}", config.ollama_model);
    info!("  Bind Address: {}", config.bind_address);

    // Create shared state
    let state = AppState {
        sessions: create_session_manager(),
        ollama: OllamaClient::new(&config.ollama_url, &config.ollama_model),
        tools: ToolExecutor::new(&config.brave_api_key),
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/", get(routes::index))
        .route("/chat", post(routes::chat))
        .route("/clear", post(routes::clear))
        .route("/health", get(routes::health))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    // Parse bind address
    let addr: SocketAddr = config
        .bind_address
        .parse()
        .expect("Invalid bind address");

    info!("Ferret is ready and listening on http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub ollama_url: String,
    pub ollama_model: String,
    pub brave_api_key: String,
    pub bind_address: String,
    pub session_timeout_mins: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen2.5:7b".to_string()),
            brave_api_key: env::var("BRAVE_API_KEY")?,
            bind_address: env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            session_timeout_mins: env::var("SESSION_TIMEOUT_MINS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        })
    }
}

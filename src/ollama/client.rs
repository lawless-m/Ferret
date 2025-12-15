use futures::StreamExt;
use reqwest::Client;
use tokio::sync::mpsc;
use tracing::{debug, error};

use crate::error::AppError;
use crate::session::ChatMessage;

use super::types::{OllamaChatChunk, OllamaChatRequest};

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    pub async fn check_health(&self) -> Result<bool, AppError> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                error!("Ollama health check failed: {}", e);
                Err(AppError::Ollama(e.to_string()))
            }
        }
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, AppError> {
        let url = format!("{}/api/chat", self.base_url);

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: None,
        };

        debug!("Sending chat request to Ollama");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ollama(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ollama(format!("Status {}: {}", status, body)));
        }

        let chunk: OllamaChatChunk = response
            .json()
            .await
            .map_err(|e| AppError::Ollama(e.to_string()))?;

        Ok(chunk.message.content)
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<mpsc::Receiver<Result<String, AppError>>, AppError> {
        let url = format!("{}/api/chat", self.base_url);

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            options: None,
        };

        debug!("Starting streaming chat request to Ollama");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ollama(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ollama(format!("Status {}: {}", status, body)));
        }

        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if line.is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<OllamaChatChunk>(line) {
                                Ok(chunk) => {
                                    if !chunk.message.content.is_empty() {
                                        if tx.send(Ok(chunk.message.content)).await.is_err() {
                                            return;
                                        }
                                    }
                                    if chunk.done {
                                        return;
                                    }
                                }
                                Err(e) => {
                                    debug!("Failed to parse chunk: {} - line: {}", e, line);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(AppError::Ollama(e.to_string()))).await;
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }
}

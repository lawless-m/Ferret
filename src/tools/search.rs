use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, error};

use crate::error::AppError;

#[derive(Clone)]
pub struct BraveClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct BraveSearchResponse {
    pub web: Option<WebResults>,
}

#[derive(Debug, Deserialize)]
pub struct WebResults {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    #[serde(default)]
    pub age: Option<String>,
}

impl SearchResult {
    pub fn format_for_context(&self) -> String {
        format!(
            "Title: {}\nURL: {}\nSnippet: {}",
            self.title, self.url, self.description
        )
    }
}

impl BraveClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn search(&self, query: &str, count: u8) -> Result<Vec<SearchResult>, AppError> {
        let url = "https://api.search.brave.com/res/v1/web/search";

        debug!("Searching Brave for: {}", query);

        let response = self
            .client
            .get(url)
            .header("X-Subscription-Token", &self.api_key)
            .query(&[("q", query), ("count", &count.to_string())])
            .send()
            .await
            .map_err(|e| AppError::BraveSearch(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Brave search failed: {} - {}", status, body);
            return Err(AppError::BraveSearch(format!("Status {}: {}", status, body)));
        }

        let search_response: BraveSearchResponse = response
            .json()
            .await
            .map_err(|e| AppError::BraveSearch(e.to_string()))?;

        Ok(search_response
            .web
            .map(|w| w.results)
            .unwrap_or_default())
    }

    pub fn format_results(query: &str, results: &[SearchResult]) -> String {
        let mut output = format!("[Tool Result: search]\nQuery: \"{}\"\n\n", query);

        if results.is_empty() {
            output.push_str("No results found.\n");
        } else {
            for (i, result) in results.iter().enumerate() {
                output.push_str(&format!(
                    "{}. {}\n",
                    i + 1,
                    result.format_for_context()
                ));
                output.push('\n');
            }
        }

        output.push_str("[End Tool Result]");
        output
    }

    pub fn format_error(error: &str) -> String {
        format!("[Tool Result: search]\nError: {}\n[End Tool Result]", error)
    }
}

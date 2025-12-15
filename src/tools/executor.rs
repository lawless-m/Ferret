use tracing::{debug, error};

use super::fetch::PageFetcher;
use super::parser::ToolCall;
use super::search::BraveClient;

#[derive(Clone)]
pub struct ToolExecutor {
    brave: BraveClient,
    fetcher: PageFetcher,
}

pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub content: String,
}

impl ToolExecutor {
    pub fn new(brave_api_key: &str) -> Self {
        Self {
            brave: BraveClient::new(brave_api_key),
            fetcher: PageFetcher::new(),
        }
    }

    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        match call {
            ToolCall::Search { query } => self.execute_search(query).await,
            ToolCall::Fetch { url } => self.execute_fetch(url).await,
        }
    }

    async fn execute_search(&self, query: &str) -> ToolResult {
        debug!("Executing search: {}", query);

        match self.brave.search(query, 10).await {
            Ok(results) => {
                let content = BraveClient::format_results(query, &results);
                ToolResult {
                    tool: "search".to_string(),
                    success: true,
                    content,
                }
            }
            Err(e) => {
                error!("Search failed: {}", e);
                ToolResult {
                    tool: "search".to_string(),
                    success: false,
                    content: BraveClient::format_error(&e.to_string()),
                }
            }
        }
    }

    async fn execute_fetch(&self, url: &str) -> ToolResult {
        debug!("Executing fetch: {}", url);

        match self.fetcher.fetch(url).await {
            Ok(content) => ToolResult {
                tool: "fetch".to_string(),
                success: true,
                content,
            },
            Err(e) => {
                error!("Fetch failed: {}", e);
                ToolResult {
                    tool: "fetch".to_string(),
                    success: false,
                    content: PageFetcher::format_error(url, &e.to_string()),
                }
            }
        }
    }
}

use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tracing::debug;

use crate::error::AppError;

const FETCH_TIMEOUT_SECS: u64 = 10;
const MAX_CONTENT_SIZE: usize = 1_000_000; // 1MB
const MAX_OUTPUT_CHARS: usize = 4000;

#[derive(Clone)]
pub struct PageFetcher {
    client: Client,
}

impl PageFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .user_agent("Ferret/0.1 (Web research assistant)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn fetch(&self, url: &str) -> Result<String, AppError> {
        debug!("Fetching page: {}", url);

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::PageFetch("Invalid URL: must start with http:// or https://".to_string()));
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::PageFetch(format!("Connection timeout after {} seconds", FETCH_TIMEOUT_SECS))
                } else {
                    AppError::PageFetch(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            return Err(AppError::PageFetch(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        // Check content length if available
        if let Some(len) = response.content_length() {
            if len as usize > MAX_CONTENT_SIZE {
                return Err(AppError::PageFetch(format!(
                    "Content too large: {} bytes (max {})",
                    len, MAX_CONTENT_SIZE
                )));
            }
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::PageFetch(e.to_string()))?;

        if bytes.len() > MAX_CONTENT_SIZE {
            return Err(AppError::PageFetch(format!(
                "Content too large: {} bytes (max {})",
                bytes.len(),
                MAX_CONTENT_SIZE
            )));
        }

        let html = String::from_utf8_lossy(&bytes).to_string();
        let text = extract_text(&html);

        Ok(Self::format_result(url, &content_type, &text))
    }

    fn format_result(url: &str, content_type: &str, text: &str) -> String {
        let truncated = if text.len() > MAX_OUTPUT_CHARS {
            format!(
                "{}\n\n[Content truncated at {} characters]",
                &text[..MAX_OUTPUT_CHARS],
                MAX_OUTPUT_CHARS
            )
        } else {
            text.to_string()
        };

        format!(
            "[Tool Result: fetch]\nURL: {}\nContent-Type: {}\nLength: {} characters\n\n{}\n[End Tool Result]",
            url,
            content_type,
            text.len(),
            truncated
        )
    }

    pub fn format_error(url: &str, error: &str) -> String {
        format!(
            "[Tool Result: fetch]\nURL: {}\nError: {}\n[End Tool Result]",
            url, error
        )
    }
}

impl Default for PageFetcher {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_text(html: &str) -> String {
    let document = Html::parse_document(html);

    // Remove script and style elements
    let _script_selector = Selector::parse("script, style, noscript, nav, footer, header").ok();

    let mut text_parts = Vec::new();

    // Try to find main content areas first
    let content_selectors = [
        "article",
        "main",
        "[role=\"main\"]",
        ".content",
        "#content",
        ".post-content",
        ".article-content",
    ];

    let mut found_main_content = false;

    for selector_str in content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let cleaned = clean_text(&text);
                if !cleaned.is_empty() && cleaned.len() > 100 {
                    text_parts.push(cleaned);
                    found_main_content = true;
                }
            }
        }
        if found_main_content {
            break;
        }
    }

    // Fallback to body if no main content found
    if !found_main_content {
        if let Ok(body_selector) = Selector::parse("body") {
            for element in document.select(&body_selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                text_parts.push(clean_text(&text));
            }
        }
    }

    // Also try to get the title
    if let Ok(title_selector) = Selector::parse("title") {
        if let Some(title_el) = document.select(&title_selector).next() {
            let title = title_el.text().collect::<String>();
            if !title.is_empty() {
                text_parts.insert(0, format!("Title: {}\n", title.trim()));
            }
        }
    }

    text_parts.join("\n\n")
}

fn clean_text(text: &str) -> String {
    // Replace multiple whitespace with single space
    let mut result = String::new();
    let mut prev_whitespace = false;

    for c in text.chars() {
        if c.is_whitespace() {
            if !prev_whitespace {
                result.push(' ');
            }
            prev_whitespace = true;
        } else {
            result.push(c);
            prev_whitespace = false;
        }
    }

    result.trim().to_string()
}

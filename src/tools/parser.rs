use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SEARCH_PATTERN: Regex =
        Regex::new(r"<search>(.*?)</search>").unwrap();
    static ref FETCH_PATTERN: Regex =
        Regex::new(r"<fetch>(.*?)</fetch>").unwrap();
}

#[derive(Debug, Clone)]
pub enum ToolCall {
    Search { query: String },
    Fetch { url: String },
}

impl ToolCall {
    pub fn name(&self) -> &'static str {
        match self {
            ToolCall::Search { .. } => "search",
            ToolCall::Fetch { .. } => "fetch",
        }
    }

    pub fn query(&self) -> &str {
        match self {
            ToolCall::Search { query } => query,
            ToolCall::Fetch { url } => url,
        }
    }
}

pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    for cap in SEARCH_PATTERN.captures_iter(text) {
        if let Some(query) = cap.get(1) {
            let query_str = query.as_str().trim();
            if !query_str.is_empty() {
                calls.push(ToolCall::Search {
                    query: query_str.to_string(),
                });
            }
        }
    }

    for cap in FETCH_PATTERN.captures_iter(text) {
        if let Some(url) = cap.get(1) {
            let url_str = url.as_str().trim();
            if !url_str.is_empty() {
                calls.push(ToolCall::Fetch {
                    url: url_str.to_string(),
                });
            }
        }
    }

    calls
}

pub fn has_tool_calls(text: &str) -> bool {
    SEARCH_PATTERN.is_match(text) || FETCH_PATTERN.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search() {
        let text = "Let me search for that. <search>rust async streams</search>";
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            ToolCall::Search { query } => assert_eq!(query, "rust async streams"),
            _ => panic!("Expected search call"),
        }
    }

    #[test]
    fn test_parse_fetch() {
        let text = "<fetch>https://example.com/page</fetch>";
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            ToolCall::Fetch { url } => assert_eq!(url, "https://example.com/page"),
            _ => panic!("Expected fetch call"),
        }
    }

    #[test]
    fn test_parse_multiple() {
        let text = "Let me search and fetch.\n<search>query</search>\n<fetch>https://example.com</fetch>";
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_empty_query() {
        let text = "<search></search>";
        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 0);
    }
}

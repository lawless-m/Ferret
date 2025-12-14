use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Chunk { content: String },
    ToolStart { tool: String, query: String },
    ToolEnd { tool: String, success: bool },
    Error { message: String },
    Done,
}

impl StreamEvent {
    pub fn to_sse(&self) -> String {
        format!("data: {}\n\n", serde_json::to_string(self).unwrap())
    }

    pub fn chunk(content: impl Into<String>) -> Self {
        StreamEvent::Chunk {
            content: content.into(),
        }
    }

    pub fn tool_start(tool: impl Into<String>, query: impl Into<String>) -> Self {
        StreamEvent::ToolStart {
            tool: tool.into(),
            query: query.into(),
        }
    }

    pub fn tool_end(tool: impl Into<String>, success: bool) -> Self {
        StreamEvent::ToolEnd {
            tool: tool.into(),
            success,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        StreamEvent::Error {
            message: message.into(),
        }
    }

    pub fn done() -> Self {
        StreamEvent::Done
    }
}

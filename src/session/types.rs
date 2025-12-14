use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

impl Session {
    pub fn new(id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            messages: Vec::new(),
            created_at: now,
            last_activity: now,
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.last_activity = Utc::now();
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.last_activity = Utc::now();
    }
}

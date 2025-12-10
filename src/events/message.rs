//! Message event type

use super::traits::{Event, EventType};
use crate::InternalMessage;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a simple UUID-like ID
fn generate_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("evt_{:x}", now)
}

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Information about the model that generated a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name/identifier
    pub model_name: String,
    /// Provider name (e.g., "openai", "anthropic")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// A message event in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    /// Unique event ID
    pub event_id: String,

    /// Session this event belongs to
    pub session_id: String,

    /// Project hash (for storage routing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_hash: Option<String>,

    /// Event timestamp (Unix milliseconds)
    pub timestamp_ms: u64,

    /// Sequence number for ordering
    pub sequence: u32,

    /// The message content
    pub message: InternalMessage,

    /// Cached token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<usize>,

    /// Model information (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_info: Option<ModelInfo>,
}

impl MessageEvent {
    /// Create a new message event
    pub fn new(session_id: impl Into<String>, sequence: u32, message: InternalMessage) -> Self {
        Self {
            event_id: generate_id(),
            session_id: session_id.into(),
            project_hash: None,
            timestamp_ms: now_ms(),
            sequence,
            message,
            token_count: None,
            model_info: None,
        }
    }

    /// Create a user message event
    pub fn user(session_id: impl Into<String>, sequence: u32, content: impl Into<String>) -> Self {
        Self::new(session_id, sequence, InternalMessage::user(content))
    }

    /// Create an assistant message event
    pub fn assistant(
        session_id: impl Into<String>,
        sequence: u32,
        content: impl Into<String>,
    ) -> Self {
        Self::new(session_id, sequence, InternalMessage::assistant(content))
    }

    /// Create a system message event
    pub fn system(
        session_id: impl Into<String>,
        sequence: u32,
        content: impl Into<String>,
    ) -> Self {
        Self::new(session_id, sequence, InternalMessage::system(content))
    }

    /// Set project hash
    pub fn with_project(mut self, project_hash: impl Into<String>) -> Self {
        self.project_hash = Some(project_hash.into());
        self
    }

    /// Set token count
    pub fn with_token_count(mut self, count: usize) -> Self {
        self.token_count = Some(count);
        self
    }

    /// Set model info
    pub fn with_model_info(mut self, model: impl Into<String>, provider: Option<String>) -> Self {
        self.model_info = Some(ModelInfo {
            model_name: model.into(),
            provider,
        });
        self
    }

    /// Set a specific event ID (useful for testing or migration)
    pub fn with_event_id(mut self, event_id: impl Into<String>) -> Self {
        self.event_id = event_id.into();
        self
    }
}

impl Event for MessageEvent {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn event_type(&self) -> EventType {
        EventType::Message
    }

    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    fn sequence(&self) -> u32 {
        self.sequence
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

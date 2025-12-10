//! Tool result event type

use super::traits::{Event, EventType};
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

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this is a result for
    pub tool_call_id: String,

    /// Result content (text or JSON)
    pub content: serde_json::Value,

    /// Whether this is an error result
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful result with text content
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: serde_json::Value::String(content.into()),
            is_error: false,
        }
    }

    /// Create a successful result with JSON content
    pub fn success_json(tool_call_id: impl Into<String>, content: serde_json::Value) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content,
            is_error: false,
        }
    }

    /// Create an error result
    pub fn error(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: serde_json::Value::String(error.into()),
            is_error: true,
        }
    }
}

/// A tool result event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultEvent {
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

    /// Reference to the tool call event
    pub tool_call_event_id: String,

    /// The result
    pub result: ToolResult,

    /// Execution duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResultEvent {
    /// Create a successful result event
    pub fn success(
        session_id: impl Into<String>,
        sequence: u32,
        tool_call_event_id: impl Into<String>,
        tool_call_id: impl Into<String>,
        content: serde_json::Value,
    ) -> Self {
        Self {
            event_id: generate_id(),
            session_id: session_id.into(),
            project_hash: None,
            timestamp_ms: now_ms(),
            sequence,
            tool_call_event_id: tool_call_event_id.into(),
            result: ToolResult {
                tool_call_id: tool_call_id.into(),
                content,
                is_error: false,
            },
            duration_ms: None,
            error: None,
        }
    }

    /// Create an error result event
    pub fn error(
        session_id: impl Into<String>,
        sequence: u32,
        tool_call_event_id: impl Into<String>,
        tool_call_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        let error_str = error.into();
        Self {
            event_id: generate_id(),
            session_id: session_id.into(),
            project_hash: None,
            timestamp_ms: now_ms(),
            sequence,
            tool_call_event_id: tool_call_event_id.into(),
            result: ToolResult {
                tool_call_id: tool_call_id.into(),
                content: serde_json::Value::String(error_str.clone()),
                is_error: true,
            },
            duration_ms: None,
            error: Some(error_str),
        }
    }

    /// Set project hash
    pub fn with_project(mut self, project_hash: impl Into<String>) -> Self {
        self.project_hash = Some(project_hash.into());
        self
    }

    /// Set execution duration
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Set a specific event ID (useful for testing or migration)
    pub fn with_event_id(mut self, event_id: impl Into<String>) -> Self {
        self.event_id = event_id.into();
        self
    }
}

impl Event for ToolResultEvent {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn event_type(&self) -> EventType {
        EventType::ToolResult
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

//! Event trait definitions

use serde::{Deserialize, Serialize};

/// Event type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// A message in the conversation
    Message,
    /// A tool call request
    ToolCall,
    /// A tool execution result
    ToolResult,
    /// System signal (e.g., session start/end)
    SystemSignal,
    /// An error event
    Error,
}

impl EventType {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::Message => "message",
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::SystemSignal => "system_signal",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Base event trait for all conversation events
pub trait Event: Send + Sync {
    /// Get the unique event ID
    fn event_id(&self) -> &str;

    /// Get the event type
    fn event_type(&self) -> EventType;

    /// Get the session ID this event belongs to
    fn session_id(&self) -> &str;

    /// Get the event timestamp (Unix milliseconds)
    fn timestamp_ms(&self) -> u64;

    /// Get the sequence number (for ordering within session)
    fn sequence(&self) -> u32;

    /// Serialize to JSON value
    fn to_json(&self) -> serde_json::Value;
}

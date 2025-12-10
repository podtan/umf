//! Event envelope for type-erased event storage

use super::traits::EventType;
use super::{MessageEvent, ToolCallEvent, ToolResultEvent};
use serde::{Deserialize, Serialize};

/// Event envelope for storage and serialization
///
/// This provides a uniform wrapper for any event type, suitable for
/// JSONL storage where each line is a single envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event ID
    pub event_id: String,

    /// Event type discriminator
    pub event_type: EventType,

    /// Session this event belongs to
    pub session_id: String,

    /// Project hash (for storage routing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_hash: Option<String>,

    /// Event timestamp (Unix milliseconds)
    pub timestamp_ms: u64,

    /// Sequence number for ordering within session
    pub sequence: u32,

    /// Type-specific payload
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Create a message event envelope
    pub fn message(event: MessageEvent) -> Self {
        Self {
            event_id: event.event_id.clone(),
            event_type: EventType::Message,
            session_id: event.session_id.clone(),
            project_hash: event.project_hash.clone(),
            timestamp_ms: event.timestamp_ms,
            sequence: event.sequence,
            payload: serde_json::to_value(&event).unwrap(),
        }
    }

    /// Create a tool call event envelope
    pub fn tool_call(event: ToolCallEvent) -> Self {
        Self {
            event_id: event.event_id.clone(),
            event_type: EventType::ToolCall,
            session_id: event.session_id.clone(),
            project_hash: event.project_hash.clone(),
            timestamp_ms: event.timestamp_ms,
            sequence: event.sequence,
            payload: serde_json::to_value(&event).unwrap(),
        }
    }

    /// Create a tool result event envelope
    pub fn tool_result(event: ToolResultEvent) -> Self {
        Self {
            event_id: event.event_id.clone(),
            event_type: EventType::ToolResult,
            session_id: event.session_id.clone(),
            project_hash: event.project_hash.clone(),
            timestamp_ms: event.timestamp_ms,
            sequence: event.sequence,
            payload: serde_json::to_value(&event).unwrap(),
        }
    }

    /// Extract as message event
    pub fn as_message_event(&self) -> Option<MessageEvent> {
        if self.event_type == EventType::Message {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Extract as tool call event
    pub fn as_tool_call_event(&self) -> Option<ToolCallEvent> {
        if self.event_type == EventType::ToolCall {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Extract as tool result event
    pub fn as_tool_result_event(&self) -> Option<ToolResultEvent> {
        if self.event_type == EventType::ToolResult {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Serialize to JSON string (for JSONL storage)
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Parse from JSON string (for JSONL reading)
    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

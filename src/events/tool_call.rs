//! Tool call event type

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

/// Simple tool call representation for events
///
/// This is a simplified version that stores the essential tool call info
/// without provider-specific formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Name of the tool to call
    pub name: String,
    /// Input arguments (JSON value)
    pub arguments: serde_json::Value,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }
}

/// Tool call execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    /// Tool call created, not yet executing
    Pending,
    /// Tool is currently executing
    Executing,
    /// Tool completed successfully
    Completed,
    /// Tool execution failed
    Failed,
    /// Tool execution was cancelled
    Cancelled,
}

impl Default for ToolCallStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// MCP (Model Context Protocol) server context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    /// MCP server name
    pub server_name: String,
    /// MCP server URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    /// Transport type (e.g., "stdio", "http")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
}

/// A tool call event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
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

    /// Reference to the assistant message that requested this call
    pub message_event_id: String,

    /// The tool call details
    pub tool_call: ToolCall,

    /// Current execution status
    #[serde(default)]
    pub status: ToolCallStatus,

    /// MCP context (if this is an MCP tool)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_context: Option<McpContext>,
}

impl ToolCallEvent {
    /// Create a new tool call event
    pub fn new(
        session_id: impl Into<String>,
        sequence: u32,
        message_event_id: impl Into<String>,
        tool_call: ToolCall,
    ) -> Self {
        Self {
            event_id: generate_id(),
            session_id: session_id.into(),
            project_hash: None,
            timestamp_ms: now_ms(),
            sequence,
            message_event_id: message_event_id.into(),
            tool_call,
            status: ToolCallStatus::Pending,
            mcp_context: None,
        }
    }

    /// Set project hash
    pub fn with_project(mut self, project_hash: impl Into<String>) -> Self {
        self.project_hash = Some(project_hash.into());
        self
    }

    /// Set MCP context
    pub fn with_mcp_context(mut self, ctx: McpContext) -> Self {
        self.mcp_context = Some(ctx);
        self
    }

    /// Update status
    pub fn with_status(mut self, status: ToolCallStatus) -> Self {
        self.status = status;
        self
    }

    /// Set a specific event ID (useful for testing or migration)
    pub fn with_event_id(mut self, event_id: impl Into<String>) -> Self {
        self.event_id = event_id.into();
        self
    }
}

impl Event for ToolCallEvent {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn event_type(&self) -> EventType {
        EventType::ToolCall
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

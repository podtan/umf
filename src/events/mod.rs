//! Event types for conversation tracking and storage
//!
//! This module provides event types that wrap messages, tool calls, and tool results
//! for storage and analytics purposes. Each event includes metadata like timestamps,
//! sequence numbers, and session information.
//!
//! ## Event Types
//!
//! - [`MessageEvent`]: A message in a conversation (user, assistant, system)
//! - [`ToolCallEvent`]: A tool call requested by the assistant
//! - [`ToolResultEvent`]: The result of executing a tool
//! - [`EventEnvelope`]: A wrapper that can hold any event type
//!
//! ## Usage
//!
//! ```rust
//! use umf::events::{MessageEvent, ToolCallEvent, ToolResultEvent, EventEnvelope};
//! use umf::InternalMessage;
//!
//! // Create a message event
//! let msg_event = MessageEvent::user("session_123", 1, "Hello!");
//!
//! // Wrap in envelope for storage
//! let envelope = EventEnvelope::message(msg_event);
//! let json_line = serde_json::to_string(&envelope).unwrap();
//! ```

mod envelope;
mod message;
mod tool_call;
mod tool_result;
mod traits;

pub use envelope::EventEnvelope;
pub use message::{MessageEvent, ModelInfo};
pub use tool_call::{McpContext, ToolCall, ToolCallEvent, ToolCallStatus};
pub use tool_result::{ToolResult, ToolResultEvent};
pub use traits::{Event, EventType};

#[cfg(test)]
mod tests;

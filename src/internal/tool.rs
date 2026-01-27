//! Internal tool types for the Universal Message Format hub model.
//!
//! This module provides protocol-agnostic tool representations that serve
//! as the canonical format for tool definitions, calls, and results.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Internal tool representation - the hub format for all protocols.
///
/// This type serves as the canonical representation for tools from any source:
/// - Native tools (e.g., from CATS)
/// - MCP tools (from Model Context Protocol servers)
/// - A2A tools (from Agent-to-Agent protocol, future)
///
/// Protocol-specific metadata is preserved in the `metadata` field to enable
/// round-trip conversion without data loss.
///
/// # Example
///
/// ```rust
/// use umf::internal::InternalTool;
/// use serde_json::json;
///
/// let tool = InternalTool::new(
///     "get_weather",
///     "Get the current weather for a location",
///     json!({
///         "type": "object",
///         "properties": {
///             "location": { "type": "string", "description": "City name" }
///         },
///         "required": ["location"]
///     }),
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InternalTool {
    /// Tool name (unique identifier within a registry).
    pub name: String,

    /// Tool description for LLM consumption.
    pub description: String,

    /// JSON Schema for tool parameters.
    ///
    /// This should follow the JSON Schema specification and typically
    /// has the structure:
    /// ```json
    /// {
    ///     "type": "object",
    ///     "properties": { ... },
    ///     "required": [ ... ]
    /// }
    /// ```
    pub parameters: Value,

    /// Protocol-specific metadata preserved through conversion.
    ///
    /// Used to store protocol-specific fields that don't map directly
    /// to the internal format, enabling lossless round-trip conversion.
    /// Keys are prefixed with the protocol name (e.g., "mcp_annotations").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

impl InternalTool {
    /// Create a new internal tool.
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the tool.
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Check if the tool has a specific metadata key.
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }
}

/// Internal tool call representation.
///
/// Represents a request to invoke a tool with specific arguments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InternalToolCall {
    /// Unique identifier for this tool call.
    ///
    /// Used to correlate tool calls with their results.
    pub id: String,

    /// Name of the tool to invoke.
    pub name: String,

    /// Arguments to pass to the tool.
    ///
    /// This should match the tool's parameter schema.
    pub arguments: Value,
}

impl InternalToolCall {
    /// Create a new tool call.
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }
}

/// Internal tool result representation.
///
/// Represents the result of a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InternalToolResult {
    /// ID of the tool call this is a result for.
    pub tool_call_id: String,

    /// Result content (typically stringified).
    pub content: String,

    /// Whether this result represents an error.
    #[serde(default)]
    pub is_error: bool,
}

impl InternalToolResult {
    /// Create a successful tool result.
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Create an error tool result.
    pub fn error(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: error.into(),
            is_error: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_internal_tool_creation() {
        let tool = InternalTool::new(
            "get_weather",
            "Get weather for a location",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            }),
        );

        assert_eq!(tool.name, "get_weather");
        assert_eq!(tool.description, "Get weather for a location");
        assert!(tool.metadata.is_empty());
    }

    #[test]
    fn test_internal_tool_with_metadata() {
        let tool = InternalTool::new("test", "Test tool", json!({}))
            .with_metadata("mcp_annotations", json!({"readOnlyHint": true}));

        assert!(tool.has_metadata("mcp_annotations"));
        assert_eq!(
            tool.get_metadata("mcp_annotations"),
            Some(&json!({"readOnlyHint": true}))
        );
    }

    #[test]
    fn test_internal_tool_call() {
        let call = InternalToolCall::new("call_123", "get_weather", json!({"location": "SF"}));

        assert_eq!(call.id, "call_123");
        assert_eq!(call.name, "get_weather");
        assert_eq!(call.arguments["location"], "SF");
    }

    #[test]
    fn test_internal_tool_result() {
        let success = InternalToolResult::success("call_123", "72°F, sunny");
        assert!(!success.is_error);
        assert_eq!(success.content, "72°F, sunny");

        let error = InternalToolResult::error("call_456", "Location not found");
        assert!(error.is_error);
        assert_eq!(error.content, "Location not found");
    }

    #[test]
    fn test_internal_tool_serialization() {
        let tool = InternalTool::new("test", "Test", json!({"type": "object"}))
            .with_metadata("source", json!("mcp"));

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: InternalTool = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, tool);
    }
}

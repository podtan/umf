//! Universal Message Format (UMF)
//!
//! This crate provides a provider-agnostic message representation for LLM interactions.
//! It follows the OpenAI-compatible JSON structure as the foundation while supporting
//! conversion to any LLM provider format (Anthropic, Google Gemini, Cohere, etc.).
//!
//! ## UDML/URP Interface (v0.2.0+)
//!
//! **Recommended Usage**: Enable the `udml` feature and use the standard UDML interface:
//!
//! ```rust,ignore
//! use umf::{UmfHandler, create_message_urp};
//!
//! // Create a URP request for a user message
//! let urp = create_message_urp("create-user-message", "Hello, world!", "my-component")?;
//!
//! // Handle the request through UDML interface
//! let handler = UmfHandler::new();
//! let response = handler.handle(urp)?;
//!
//! // Extract the message from the response
//! let message: InternalMessage = serde_json::from_value(
//!     response.information.data.expect("Should have data")
//! )?;
//! ```
//!
//! ## Core Principles
//!
//! 1. **UDML-First Design**: All operations exposed through uniform URP interface (v0.2.0+)
//! 2. **OpenAI-Compatible Base**: The format follows OpenAI's JSON structure
//! 3. **Provider-Agnostic**: Can be converted to any LLM provider format
//! 4. **Metadata Support**: Includes optional metadata for internal tracking
//! 5. **Tool Calling Support**: Full support for function/tool calling
//!
//! ## Legacy Usage (Deprecated)
//!
//! Direct struct access is deprecated when using the `udml` feature:
//!
//! ```rust,ignore
//! // ⚠️ DEPRECATED: Direct usage without UDML
//! use umf::{InternalMessage, MessageRole, ContentBlock};
//!
//! let msg = InternalMessage::user("Hello, world!");
//! ```
//!
//! Use the URP interface instead for UDML compliance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// UDML Specification and URP Support
// ============================================================================

pub mod udml_spec;
pub use udml_spec::{UDML_SPEC_YAML, COMPONENT_ID, schema_ref};

#[cfg(feature = "udml")]
pub use udml_spec::load_specification;

#[cfg(feature = "udml")]
pub mod urp_handler;

#[cfg(feature = "udml")]
pub use urp_handler::{UmfHandler, create_message_urp};

// ============================================================================
// ChatML Support
// ============================================================================

pub mod chatml;
pub use chatml::{ChatMLFormatter, ChatMLMessage, MessageRole as ChatMLMessageRole};

// ============================================================================
// Streaming Support (optional feature)
// ============================================================================

#[cfg(feature = "streaming")]
pub mod streaming;
#[cfg(feature = "streaming")]
pub use streaming::{StreamingAccumulator, StreamChunk};

// ============================================================================
// Core Message Types
// ============================================================================

/// A message in the internal format
///
/// This represents a single message in a conversation, with role, content,
/// and optional metadata for provider-specific information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalMessage {
    /// Message role (system, user, assistant, tool)
    pub role: MessageRole,
    /// Message content (text or structured blocks)
    pub content: MessageContent,
    /// Optional metadata for provider-specific data
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    /// Tool call ID for tool messages (required when role is "tool")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool name for tool messages (required when role is "tool")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl InternalMessage {
    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::Text(text.into()),
            metadata: HashMap::new(),
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a user message
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(text.into()),
            metadata: HashMap::new(),
            tool_call_id: None,
            name: None,
        }
    }

    /// Create an assistant message
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(text.into()),
            metadata: HashMap::new(),
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a tool result message (legacy - use tool_result instead)
    pub fn tool(content: MessageContent) -> Self {
        Self {
            role: MessageRole::Tool,
            content,
            metadata: HashMap::new(),
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a properly structured tool result message
    pub fn tool_result(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            role: MessageRole::Tool,
            content: MessageContent::Text(content.into()),
            metadata: HashMap::new(),
            tool_call_id: Some(tool_call_id.into()),
            name: Some(name.into()),
        }
    }

    /// Create an assistant message with tool calls
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ContentBlock>) -> Self {
        let mut blocks = vec![ContentBlock::text(content.into())];
        blocks.extend(tool_calls);

        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Blocks(blocks),
            metadata: HashMap::new(),
            tool_call_id: None,
            name: None,
        }
    }

    /// Get text content if this is a text message
    pub fn text(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get blocks if this is a block-based message
    pub fn blocks(&self) -> Option<&[ContentBlock]> {
        match &self.content {
            MessageContent::Blocks(blocks) => Some(blocks),
            _ => None,
        }
    }

    /// Extract all text content from the message
    ///
    /// For text messages, returns the text directly.
    /// For block messages, extracts and concatenates text from all text blocks.
    pub fn to_text(&self) -> String {
        match &self.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Blocks(blocks) => {
                blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(text.as_str()),
                        ContentBlock::ToolResult { content, .. } => Some(content.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

/// Message role in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System-level instructions
    System,
    /// User input
    User,
    /// Assistant response
    Assistant,
    /// Tool execution result
    Tool,
}

impl MessageRole {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Message content (text or structured blocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Structured content blocks (for images, tool use, etc.)
    Blocks(Vec<ContentBlock>),
}

impl MessageContent {
    /// Create text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Create blocks content
    pub fn blocks(blocks: Vec<ContentBlock>) -> Self {
        Self::Blocks(blocks)
    }

    /// Check if this is text content
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Check if this is blocks content
    pub fn is_blocks(&self) -> bool {
        matches!(self, Self::Blocks(_))
    }
}

// ============================================================================
// Content Block Types
// ============================================================================

/// Image source for image blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image data
    Base64 {
        /// MIME type of the image (e.g., "image/png")
        media_type: String,
        /// Base64-encoded image data
        data: String,
    },
    /// URL to an image
    Url {
        /// URL of the image
        url: String,
    },
}

/// A content block within a message
///
/// This follows the Universal Message Format specification exactly.
/// Each variant serializes to JSON with a "type" field and flattened fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content
    Text {
        /// The text content
        text: String,
    },
    /// Image content
    Image {
        /// The image source
        source: ImageSource,
    },
    /// Tool use (function call)
    ToolUse {
        /// Unique identifier for this tool call
        id: String,
        /// Name of the tool to call
        name: String,
        /// Input arguments for the tool
        input: serde_json::Value,
    },
    /// Tool result (function response)
    ToolResult {
        /// ID of the tool call this is a result for
        tool_use_id: String,
        /// The result content
        content: String,
    },
}

impl ContentBlock {
    /// Create a text block
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create an image block from a source
    pub fn image(source: ImageSource) -> Self {
        Self::Image { source }
    }

    /// Create a tool use block
    pub fn tool_use(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Create a tool result block
    pub fn tool_result(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
        }
    }

    /// Get the text from a text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Get tool use information (id, name, input)
    pub fn as_tool_use(&self) -> Option<(&str, &str, &serde_json::Value)> {
        match self {
            Self::ToolUse { id, name, input } => Some((id, name, input)),
            _ => None,
        }
    }

    /// Get tool result information (tool_use_id, content)
    pub fn as_tool_result(&self) -> Option<(&str, &str)> {
        match self {
            Self::ToolResult { tool_use_id, content } => Some((tool_use_id, content)),
            _ => None,
        }
    }

    /// Get image source
    pub fn as_image(&self) -> Option<&ImageSource> {
        match self {
            Self::Image { source } => Some(source),
            _ => None,
        }
    }
}

// ============================================================================
// OpenAI-Compatible Tool Types (Internal)
// ============================================================================
//
// These types are internal to UMF and used for ChatML formatting and streaming.
// External access should go through the UDML/URP interface.
//
// They are kept as pub(crate) for internal modules but not exposed in the public API.

/// Function call structure for tool invocations (internal)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Tool call structure for function calling (internal)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: FunctionCall,
}

/// Function definition for tools (internal)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Function {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Tool definition for OpenAI-compatible tools (internal)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Tool {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: Function,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = InternalMessage::system("You are a helpful assistant");
        assert_eq!(msg.role, MessageRole::System);
        assert_eq!(msg.text(), Some("You are a helpful assistant"));

        let msg = InternalMessage::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.text(), Some("Hello"));

        let msg = InternalMessage::assistant("Hi there!");
        assert_eq!(msg.role, MessageRole::Assistant);
        assert_eq!(msg.text(), Some("Hi there!"));
    }

    #[test]
    fn test_content_blocks() {
        let block = ContentBlock::text("Hello world");
        assert_eq!(block.as_text(), Some("Hello world"));

        let block = ContentBlock::tool_use(
            "tool_123",
            "get_weather",
            serde_json::json!({"location": "SF"}),
        );
        let (id, name, input) = block.as_tool_use().unwrap();
        assert_eq!(id, "tool_123");
        assert_eq!(name, "get_weather");
        assert_eq!(input["location"], "SF");

        let block = ContentBlock::tool_result("tool_123", "72°F, sunny");
        let (tool_use_id, content) = block.as_tool_result().unwrap();
        assert_eq!(tool_use_id, "tool_123");
        assert_eq!(content, "72°F, sunny");
    }

    #[test]
    fn test_message_serialization() {
        let msg = InternalMessage::user("Test message");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: InternalMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, MessageRole::User);
        assert_eq!(deserialized.text(), Some("Test message"));
    }

    #[test]
    fn test_role_string_conversion() {
        assert_eq!(MessageRole::System.as_str(), "system");
        assert_eq!(MessageRole::User.as_str(), "user");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(MessageRole::Tool.as_str(), "tool");
    }

    #[test]
    fn test_text_block_matches_spec() {
        let block = ContentBlock::text("Hello world");
        let json = serde_json::to_value(&block).unwrap();

        // Verify exact structure: {"type":"text","text":"Hello world"}
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello world");

        // Verify exactly 2 fields
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 2);
    }

    #[test]
    fn test_tool_use_block_matches_spec() {
        let block = ContentBlock::tool_use(
            "call_123",
            "search",
            serde_json::json!({"query": "weather"}),
        );
        let json = serde_json::to_value(&block).unwrap();

        // Verify exact structure
        assert_eq!(json["type"], "tool_use");
        assert_eq!(json["id"], "call_123");
        assert_eq!(json["name"], "search");
        assert_eq!(json["input"]["query"], "weather");

        // Verify exactly 4 fields
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn test_tool_result_block_matches_spec() {
        let block = ContentBlock::tool_result("call_123", "Result text");
        let json = serde_json::to_value(&block).unwrap();

        // Verify exact structure
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["tool_use_id"], "call_123");
        assert_eq!(json["content"], "Result text");

        // Verify exactly 3 fields
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 3);
    }

    #[test]
    fn test_message_with_tool_call_id() {
        let msg = InternalMessage::tool_result("call_123", "search", "Weather is sunny");
        let json = serde_json::to_value(&msg).unwrap();

        // Verify tool_call_id and name are at top level
        assert_eq!(json["role"], "tool");
        assert_eq!(json["tool_call_id"], "call_123");
        assert_eq!(json["name"], "search");
        assert_eq!(json["content"], "Weather is sunny");
    }

    #[test]
    fn test_full_message_roundtrip() {
        let blocks = vec![
            ContentBlock::text("I'll search for you"),
            ContentBlock::tool_use("call_123", "search", serde_json::json!({"q": "test"})),
        ];

        let msg = InternalMessage {
            role: MessageRole::Assistant,
            content: MessageContent::Blocks(blocks),
            metadata: std::collections::HashMap::new(),
            tool_call_id: None,
            name: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: InternalMessage = serde_json::from_str(&json).unwrap();

        // Verify structure is preserved
        assert_eq!(deserialized.role, MessageRole::Assistant);
        if let MessageContent::Blocks(blocks) = deserialized.content {
            assert_eq!(blocks.len(), 2);
            assert!(matches!(blocks[0], ContentBlock::Text { .. }));
            assert!(matches!(blocks[1], ContentBlock::ToolUse { .. }));
        } else {
            panic!("Expected blocks content");
        }
    }

    #[test]
    fn test_spec_compliance_full_example() {
        // Recreate Example 4 from universal_message_format.md
        let blocks = vec![
            ContentBlock::text("I'll help you search"),
            ContentBlock::tool_use(
                "call_abc123",
                "search",
                serde_json::json!({"query": "weather"}),
            ),
        ];

        let msg = InternalMessage {
            role: MessageRole::Assistant,
            content: MessageContent::Blocks(blocks),
            metadata: std::collections::HashMap::new(),
            tool_call_id: None,
            name: None,
        };

        let json = serde_json::to_value(&msg).unwrap();

        // Verify structure matches spec
        assert_eq!(json["role"], "assistant");

        let content = json["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);

        // First block: text
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "I'll help you search");

        // Second block: tool_use
        assert_eq!(content[1]["type"], "tool_use");
        assert_eq!(content[1]["id"], "call_abc123");
        assert_eq!(content[1]["name"], "search");
        assert_eq!(content[1]["input"]["query"], "weather");
    }

    #[test]
    fn test_wasm_provider_can_parse() {
        // Verify that serialized messages can be parsed as raw JSON with expected structure
        let msg = InternalMessage::tool_result("call_123", "search", "Result");
        let json_str = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // WASM provider expects these fields at top level
        assert_eq!(parsed["role"].as_str(), Some("tool"));
        assert_eq!(parsed["tool_call_id"].as_str(), Some("call_123"));
        assert_eq!(parsed["name"].as_str(), Some("search"));
        assert_eq!(parsed["content"].as_str(), Some("Result"));
    }
}

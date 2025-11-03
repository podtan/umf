//! Type definitions for streaming responses.

use serde::{Deserialize, Serialize};

/// Streaming response chunk from LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamChunk {
    /// Text content delta
    Text(String),
    /// Tool call delta (index-based like OpenAI SSE format)
    /// Contains partial updates to tool call at given index
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: Option<String>,
    },
    /// Stream completed
    Done,
}

/// Accumulated response from streaming
#[derive(Debug, Clone)]
pub struct AccumulatedResponse {
    /// Accumulated text content
    pub text: String,
    /// Accumulated tool calls (in index order)
    pub tool_calls: Vec<crate::ToolCall>,
}

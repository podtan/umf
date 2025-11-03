//! Streaming response accumulator.

use super::types::{StreamChunk, AccumulatedResponse};
use std::collections::HashMap;

/// Accumulates streaming chunks into a complete response.
///
/// Handles both text deltas and tool call deltas with sparse index support.
/// Anthropic may send tool_use at index 1 if index 0 is a text block, so we
/// use HashMap-based accumulation to handle non-sequential indices.
#[derive(Debug, Default)]
pub struct StreamingAccumulator {
    text: String,
    tool_calls: HashMap<usize, crate::ToolCall>,
}

impl StreamingAccumulator {
    /// Create a new accumulator
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a single chunk and accumulate it
    pub fn process_chunk(&mut self, chunk: StreamChunk) -> bool {
        match chunk {
            StreamChunk::Text(text) => {
                self.text.push_str(&text);
                false // Not done
            }
            StreamChunk::ToolCallDelta { index, id, name, arguments_delta } => {
                // Create tool call entry if it doesn't exist
                let tool_call = self.tool_calls.entry(index).or_insert_with(|| {
                    crate::ToolCall {
                        id: String::new(),
                        r#type: "function".to_string(),
                        function: crate::FunctionCall {
                            name: String::new(),
                            arguments: String::new(),
                        },
                    }
                });

                // Update the tool call (accumulative)
                if let Some(id_value) = id {
                    tool_call.id = id_value;
                }
                if let Some(name_value) = name {
                    tool_call.function.name = name_value;
                }
                if let Some(args_delta) = arguments_delta {
                    // Accumulate arguments by appending
                    tool_call.function.arguments.push_str(&args_delta);
                }
                false // Not done
            }
            StreamChunk::Done => true, // Done
        }
    }

    /// Get the accumulated response
    pub fn finish(self) -> AccumulatedResponse {
        // Convert HashMap to Vec, filtering out empty tool calls
        let tool_calls: Vec<crate::ToolCall> = self.tool_calls
            .into_iter()
            .map(|(_, tool_call)| tool_call)
            .filter(|tc| !tc.function.name.is_empty())
            .collect();

        AccumulatedResponse {
            text: self.text,
            tool_calls,
        }
    }

    /// Accumulate an entire stream into a response
    ///
    /// This is a convenience method that processes all chunks from a stream
    /// and returns the final accumulated response.
    pub async fn accumulate_stream<S, E>(mut stream: S) -> Result<AccumulatedResponse, E>
    where
        S: futures_util::Stream<Item = Result<StreamChunk, E>> + Unpin,
    {
        use futures_util::StreamExt;
        
        let mut accumulator = Self::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            if accumulator.process_chunk(chunk) {
                break; // Done
            }
        }
        
        Ok(accumulator.finish())
    }
}

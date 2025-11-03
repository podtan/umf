//! Tests for streaming accumulator

use super::*;
use crate::{ToolCall, FunctionCall};

#[test]
fn test_text_accumulation() {
    let mut acc = StreamingAccumulator::new();
    
    acc.process_chunk(StreamChunk::Text("Hello ".to_string()));
    acc.process_chunk(StreamChunk::Text("world".to_string()));
    acc.process_chunk(StreamChunk::Text("!".to_string()));
    
    let response = acc.finish();
    assert_eq!(response.text, "Hello world!");
    assert_eq!(response.tool_calls.len(), 0);
}

#[test]
fn test_tool_call_accumulation() {
    let mut acc = StreamingAccumulator::new();
    
    // Tool call at index 0
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: Some("call_123".to_string()),
        name: None,
        arguments_delta: None,
    });
    
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: None,
        name: Some("search_file".to_string()),
        arguments_delta: None,
    });
    
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: None,
        name: None,
        arguments_delta: Some("{\"pat".to_string()),
    });
    
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: None,
        name: None,
        arguments_delta: Some("tern\": \"test\"}".to_string()),
    });
    
    let response = acc.finish();
    assert_eq!(response.text, "");
    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.tool_calls[0].id, "call_123");
    assert_eq!(response.tool_calls[0].function.name, "search_file");
    assert_eq!(response.tool_calls[0].function.arguments, "{\"pattern\": \"test\"}");
}

#[test]
fn test_sparse_indices() {
    let mut acc = StreamingAccumulator::new();
    
    // Anthropic case: text block at index 0, tool_use at index 1
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 1,
        id: Some("call_456".to_string()),
        name: Some("classify_task".to_string()),
        arguments_delta: Some("{\"task_type\": \"feature\"}".to_string()),
    });
    
    let response = acc.finish();
    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.tool_calls[0].id, "call_456");
}

#[test]
fn test_multiple_tool_calls() {
    let mut acc = StreamingAccumulator::new();
    
    // Two tool calls at different indices
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: Some("call_1".to_string()),
        name: Some("tool_a".to_string()),
        arguments_delta: Some("{}".to_string()),
    });
    
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 1,
        id: Some("call_2".to_string()),
        name: Some("tool_b".to_string()),
        arguments_delta: Some("{}".to_string()),
    });
    
    let response = acc.finish();
    assert_eq!(response.tool_calls.len(), 2);
}

#[test]
fn test_mixed_content() {
    let mut acc = StreamingAccumulator::new();
    
    acc.process_chunk(StreamChunk::Text("Thinking...".to_string()));
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: Some("call_789".to_string()),
        name: Some("open".to_string()),
        arguments_delta: Some("{\"path\": \"test.rs\"}".to_string()),
    });
    
    let response = acc.finish();
    assert_eq!(response.text, "Thinking...");
    assert_eq!(response.tool_calls.len(), 1);
}

#[test]
fn test_empty_tool_calls_filtered() {
    let mut acc = StreamingAccumulator::new();
    
    // Tool call with no name should be filtered out
    acc.process_chunk(StreamChunk::ToolCallDelta {
        index: 0,
        id: Some("call_empty".to_string()),
        name: None,
        arguments_delta: None,
    });
    
    let response = acc.finish();
    assert_eq!(response.tool_calls.len(), 0);
}

#[test]
fn test_done_chunk() {
    let mut acc = StreamingAccumulator::new();
    
    let done = acc.process_chunk(StreamChunk::Done);
    assert!(done);
}

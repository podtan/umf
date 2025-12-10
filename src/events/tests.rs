//! Tests for the events module

use super::*;
use super::tool_call::ToolCall;

#[test]
fn test_message_event_user() {
    let event = MessageEvent::user("session_1", 1, "Hello, world!");

    assert!(event.event_id.starts_with("evt_"));
    assert_eq!(event.session_id, "session_1");
    assert_eq!(event.sequence, 1);
    assert!(event.timestamp_ms > 0);
    assert!(event.project_hash.is_none());
    assert!(event.token_count.is_none());
    assert!(event.model_info.is_none());

    // Check the message content
    assert_eq!(event.message.text(), Some("Hello, world!"));
}

#[test]
fn test_message_event_with_model_info() {
    let event = MessageEvent::assistant("session_1", 2, "I can help with that!")
        .with_model_info("gpt-4o", Some("openai".to_string()))
        .with_token_count(42)
        .with_project("abc123");

    assert_eq!(event.project_hash, Some("abc123".to_string()));
    assert_eq!(event.token_count, Some(42));
    assert!(event.model_info.is_some());

    let model = event.model_info.unwrap();
    assert_eq!(model.model_name, "gpt-4o");
    assert_eq!(model.provider, Some("openai".to_string()));
}

#[test]
fn test_tool_call_event() {
    let tool_call = ToolCall {
        id: "call_abc123".to_string(),
        name: "search".to_string(),
        arguments: serde_json::json!({"query": "rust programming"}),
    };

    let event = ToolCallEvent::new("session_1", 3, "msg_event_1", tool_call);

    assert!(event.event_id.starts_with("evt_"));
    assert_eq!(event.session_id, "session_1");
    assert_eq!(event.sequence, 3);
    assert_eq!(event.message_event_id, "msg_event_1");
    assert_eq!(event.tool_call.name, "search");
    assert_eq!(event.status, ToolCallStatus::Pending);
}

#[test]
fn test_tool_call_event_with_mcp() {
    let tool_call = ToolCall {
        id: "call_mcp123".to_string(),
        name: "mcp_server_do_thing".to_string(),
        arguments: serde_json::json!({}),
    };

    let event = ToolCallEvent::new("session_1", 4, "msg_event_2", tool_call)
        .with_mcp_context(McpContext {
            server_name: "my_server".to_string(),
            server_url: Some("http://localhost:9000".to_string()),
            transport: Some("stdio".to_string()),
        })
        .with_status(ToolCallStatus::Executing);

    assert!(event.mcp_context.is_some());
    assert_eq!(event.status, ToolCallStatus::Executing);

    let ctx = event.mcp_context.unwrap();
    assert_eq!(ctx.server_name, "my_server");
}

#[test]
fn test_tool_result_event_success() {
    let event = ToolResultEvent::success(
        "session_1",
        5,
        "tool_call_event_1",
        "call_abc123",
        serde_json::json!({"found": 42, "results": ["a", "b"]}),
    )
    .with_duration_ms(150);

    assert!(event.event_id.starts_with("evt_"));
    assert_eq!(event.session_id, "session_1");
    assert_eq!(event.sequence, 5);
    assert_eq!(event.tool_call_event_id, "tool_call_event_1");
    assert!(!event.result.is_error);
    assert_eq!(event.duration_ms, Some(150));
    assert!(event.error.is_none());
}

#[test]
fn test_tool_result_event_error() {
    let event = ToolResultEvent::error(
        "session_1",
        6,
        "tool_call_event_2",
        "call_xyz789",
        "Command timed out after 30s",
    );

    assert!(event.result.is_error);
    assert_eq!(event.error, Some("Command timed out after 30s".to_string()));
}

#[test]
fn test_event_envelope_roundtrip_message() {
    let msg_event = MessageEvent::user("session_1", 1, "Test message")
        .with_event_id("evt_test_123");

    let envelope = EventEnvelope::message(msg_event.clone());

    assert_eq!(envelope.event_id, "evt_test_123");
    assert_eq!(envelope.event_type, EventType::Message);
    assert_eq!(envelope.session_id, "session_1");
    assert_eq!(envelope.sequence, 1);

    // Roundtrip through JSON
    let json_line = envelope.to_json_line();
    let parsed = EventEnvelope::from_json_line(&json_line).unwrap();

    assert_eq!(parsed.event_id, envelope.event_id);
    assert_eq!(parsed.event_type, envelope.event_type);

    // Extract original event
    let extracted = parsed.as_message_event().unwrap();
    assert_eq!(extracted.event_id, "evt_test_123");
}

#[test]
fn test_event_envelope_roundtrip_tool_call() {
    let tool_call = ToolCall {
        id: "call_test".to_string(),
        name: "test_tool".to_string(),
        arguments: serde_json::json!({"key": "value"}),
    };

    let tc_event = ToolCallEvent::new("session_1", 2, "msg_1", tool_call)
        .with_event_id("evt_tc_456");

    let envelope = EventEnvelope::tool_call(tc_event);
    let json_line = envelope.to_json_line();
    let parsed = EventEnvelope::from_json_line(&json_line).unwrap();

    assert_eq!(parsed.event_type, EventType::ToolCall);

    let extracted = parsed.as_tool_call_event().unwrap();
    assert_eq!(extracted.tool_call.name, "test_tool");
}

#[test]
fn test_event_envelope_roundtrip_tool_result() {
    let tr_event = ToolResultEvent::success(
        "session_1",
        3,
        "evt_tc_456",
        "call_test",
        serde_json::json!("result data"),
    )
    .with_event_id("evt_tr_789");

    let envelope = EventEnvelope::tool_result(tr_event);
    let json_line = envelope.to_json_line();
    let parsed = EventEnvelope::from_json_line(&json_line).unwrap();

    assert_eq!(parsed.event_type, EventType::ToolResult);

    let extracted = parsed.as_tool_result_event().unwrap();
    assert_eq!(extracted.result.tool_call_id, "call_test");
}

#[test]
fn test_event_type_mismatch_returns_none() {
    let msg_event = MessageEvent::user("session_1", 1, "Test");
    let envelope = EventEnvelope::message(msg_event);

    // Should return None for wrong type
    assert!(envelope.as_tool_call_event().is_none());
    assert!(envelope.as_tool_result_event().is_none());

    // Should return Some for correct type
    assert!(envelope.as_message_event().is_some());
}

#[test]
fn test_tool_result_helper_methods() {
    let success = ToolResult::success("call_1", "result text");
    assert!(!success.is_error);
    assert_eq!(success.content, serde_json::json!("result text"));

    let success_json = ToolResult::success_json("call_2", serde_json::json!({"key": "value"}));
    assert!(!success_json.is_error);
    assert_eq!(success_json.content, serde_json::json!({"key": "value"}));

    let error = ToolResult::error("call_3", "Something went wrong");
    assert!(error.is_error);
    assert_eq!(error.content, serde_json::json!("Something went wrong"));
}

#[test]
fn test_event_trait_implementations() {
    use super::traits::Event;

    let msg = MessageEvent::user("sess", 1, "Hello");
    assert_eq!(msg.event_type(), EventType::Message);
    assert_eq!(msg.session_id(), "sess");
    assert_eq!(msg.sequence(), 1);

    let tool_call = ToolCall {
        id: "c1".to_string(),
        name: "test".to_string(),
        arguments: serde_json::json!({}),
    };
    let tc = ToolCallEvent::new("sess", 2, "m1", tool_call);
    assert_eq!(tc.event_type(), EventType::ToolCall);

    let tr = ToolResultEvent::success("sess", 3, "tc1", "c1", serde_json::json!("ok"));
    assert_eq!(tr.event_type(), EventType::ToolResult);
}

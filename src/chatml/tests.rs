use super::*;
use std::collections::HashMap;

#[test]
fn test_message_creation() {
    let msg = ChatMLMessage::new(
        MessageRole::User,
        "Hello, world!".to_string(),
        Some("alice".to_string()),
    );

    assert_eq!(msg.role, MessageRole::User);
    assert_eq!(msg.content, "Hello, world!");
    assert_eq!(msg.name, Some("alice".to_string()));
}

#[test]
fn test_chatml_string_format() {
    let msg = ChatMLMessage::new(
        MessageRole::System,
        "You are a helpful assistant.".to_string(),
        Some("assistant".to_string()),
    );

    let expected = "<|im_start|>system name=assistant\nYou are a helpful assistant.\n<|im_end|>";
    assert_eq!(msg.to_chatml_string(), expected);
}

#[test]
fn test_formatter() {
    let mut formatter = ChatMLFormatter::new();
    formatter.add_system_message("System prompt".to_string(), None);
    formatter.add_user_message("User message".to_string(), Some("user".to_string()));

    assert_eq!(formatter.get_message_count(), 2);
    assert!(formatter.get_last_message().unwrap().role == MessageRole::User);

    let openai_format = formatter.to_openai_format();
    assert_eq!(openai_format.len(), 2);
}

#[test]
fn test_format_thought_command() {
    let formatter = ChatMLFormatter::new();
    let result = formatter.format_thought_command("Testing ls command", "ls -la");

    assert!(result.contains("THOUGHT: Testing ls command"));
    assert!(result.contains("```bash\nls -la\n```"));
}

#[test]
fn test_replace_template_variables() {
    let formatter = ChatMLFormatter::new();
    let mut variables = HashMap::new();
    variables.insert("working_dir".to_string(), "/home/user".to_string());
    variables.insert("timeout_seconds".to_string(), "120".to_string());

    let template = "Working in: {working_dir}\nTimeout: {timeout_seconds} seconds";
    let result = formatter.replace_template_variables(template, &variables);

    assert_eq!(result, "Working in: /home/user\nTimeout: 120 seconds");
}

#[test]
fn test_validate_messages() {
    let mut formatter = ChatMLFormatter::new();

    // Valid messages
    formatter.add_system_message("System prompt".to_string(), Some("system".to_string()));
    formatter.add_user_message("User message".to_string(), None);
    formatter.add_assistant_message(
        "Assistant response".to_string(),
        Some("assistant".to_string()),
    );

    assert!(formatter.validate_messages());

    // Invalid: empty content
    let mut invalid_formatter = ChatMLFormatter::new();
    invalid_formatter.add_system_message("".to_string(), Some("system".to_string()));
    assert!(!invalid_formatter.validate_messages());

    // Invalid: system message without name
    let mut invalid_formatter2 = ChatMLFormatter::new();
    invalid_formatter2.add_system_message("System prompt".to_string(), None);
    assert!(!invalid_formatter2.validate_messages());
}

#[test]
fn test_resume_checkpoint_message_validation() {
    // Test that simulates the resume functionality creating properly named messages
    let mut formatter = ChatMLFormatter::new();

    // Simulate messages being restored from checkpoint with proper names (fixed behavior)
    formatter.add_system_message(
        "You are a helpful assistant".to_string(),
        Some("simpaticoder".to_string()),
    );
    formatter.add_user_message("Hello, how are you?".to_string(), None);
    formatter.add_assistant_message(
        "I'm doing great, thank you!".to_string(),
        Some("assistant".to_string()),
    );

    // The validation should now pass with the fix
    assert!(
        formatter.validate_messages(),
        "Resumed messages should pass validation with proper names"
    );

    // Verify the message count
    assert_eq!(formatter.get_message_count(), 3);

    // Verify the structure
    let messages = formatter.get_messages();

    // System message should have "simpaticoder" name
    assert_eq!(messages[0].role, MessageRole::System);
    assert_eq!(messages[0].name, Some("simpaticoder".to_string()));

    // User message should have no name
    assert_eq!(messages[1].role, MessageRole::User);
    assert_eq!(messages[1].name, None);

    // Assistant message should have "assistant" name
    assert_eq!(messages[2].role, MessageRole::Assistant);
    assert_eq!(messages[2].name, Some("assistant".to_string()));
}

#[test]
fn test_broken_resume_behavior_validation() {
    // Test what would happen with the old (broken) behavior where messages had None names
    let mut formatter = ChatMLFormatter::new();

    // Simulate the old broken behavior where all messages had None names
    formatter.add_system_message("System message".to_string(), None); // This would fail validation
    formatter.add_user_message("User message".to_string(), None);
    formatter.add_assistant_message("Assistant message".to_string(), None); // This would fail validation

    // This should fail validation (as it did before the fix)
    assert!(
        !formatter.validate_messages(),
        "Old behavior should fail validation due to missing names"
    );
}

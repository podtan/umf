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

#[test]
fn test_image_attachment_data_url() {
    let img = ImageAttachment::new("image/jpeg", "QUJD").with_filename("photo.jpg");
    assert_eq!(img.mime, "image/jpeg");
    assert_eq!(img.data, "QUJD");
    assert_eq!(img.filename.as_deref(), Some("photo.jpg"));
    assert_eq!(img.to_data_url(), "data:image/jpeg;base64,QUJD");

    // No filename → serializes without the key
    let bare = ImageAttachment::new("image/png", "XYZ=");
    let json = serde_json::to_value(&bare).unwrap();
    assert!(json.get("filename").is_none());
    assert_eq!(json["mime"], "image/png");
}

#[test]
fn test_add_user_message_with_images() {
    let mut formatter = ChatMLFormatter::new();
    formatter.add_system_message("System".to_string(), Some("sys".to_string()));
    let images = vec![
        ImageAttachment::new("image/jpeg", "QUJD").with_filename("a.jpg"),
        ImageAttachment::new("image/png", "WFla"),
    ];
    formatter.add_user_message_with_images(
        "describe these".to_string(),
        images,
        None,
    );

    assert_eq!(formatter.get_message_count(), 2);
    let last = formatter.get_last_message().unwrap();
    assert_eq!(last.role, MessageRole::User);
    assert_eq!(last.content, "describe these");
    assert_eq!(last.images.len(), 2);
    assert_eq!(last.images[0].mime, "image/jpeg");
    assert_eq!(last.images[1].filename, None);

    // Plain add_user_message still yields an empty sidecar
    formatter.add_user_message("text only".to_string(), None);
    assert!(formatter.get_last_message().unwrap().images.is_empty());
}

#[test]
fn test_images_serde_roundtrip_and_backward_compat() {
    // Round-trip: images serialize under the sidecar key and deserialize back.
    let mut formatter = ChatMLFormatter::new();
    formatter.add_user_message_with_images(
        "look".to_string(),
        vec![ImageAttachment::new("image/gif", "R0lGOD=")],
        None,
    );
    let msg = &formatter.get_messages()[0];
    let json = serde_json::to_value(msg).unwrap();
    assert_eq!(json["images"][0]["mime"], "image/gif");
    assert!(json["images"][0].get("filename").is_none());
    let round: ChatMLMessage = serde_json::from_value(json).unwrap();
    assert_eq!(round.role, msg.role);
    assert_eq!(round.content, msg.content);
    assert_eq!(round.images, msg.images);

    // Backward compat: pre-0.3.0 JSON has no "images" key → deserializes to empty vec.
    let legacy = serde_json::json!({
        "role": "user",
        "content": "old checkpoint message"
    });
    let old: ChatMLMessage = serde_json::from_value(legacy).unwrap();
    assert_eq!(old.content, "old checkpoint message");
    assert!(old.images.is_empty());

    // Text-only messages omit the images key entirely (compact wire form).
    let plain = ChatMLMessage::new(MessageRole::User, "hi".to_string(), None);
    let plain_json = serde_json::to_value(&plain).unwrap();
    assert!(plain_json.get("images").is_none());
}

#[test]
fn test_validation_allows_image_only_user_message() {
    let mut formatter = ChatMLFormatter::new();
    formatter.add_system_message("System".to_string(), Some("sys".to_string()));
    // Empty content but images present → valid (image-only turn).
    formatter.add_user_message_with_images(
        String::new(),
        vec![ImageAttachment::new("image/webp", "UklGRg")],
        None,
    );
    assert!(
        formatter.validate_messages(),
        "image-only user message should pass validation"
    );

    // Empty content, no images, no tool calls → still invalid.
    formatter.add_user_message(String::new(), None);
    assert!(!formatter.validate_messages());
}

#[test]
fn test_get_messages_mut_sidecar_extension() {
    let mut formatter = ChatMLFormatter::new();
    formatter.add_user_message("task text".to_string(), None);

    // Embedding layer extends the just-seeded user message in place.
    let msgs = formatter.get_messages_mut();
    msgs.last_mut()
        .unwrap()
        .images
        .push(ImageAttachment::new("image/png", "iVBOR"));

    assert_eq!(formatter.get_last_message().unwrap().images.len(), 1);
    assert_eq!(
        formatter.get_last_message().unwrap().images[0].mime,
        "image/png"
    );
    assert_eq!(formatter.get_last_message().unwrap().content, "task text");
}

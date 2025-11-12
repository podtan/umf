//! ChatML message formatter for simpaticoder.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tiktoken_rs::cl100k_base;
use crate::InternalMessage;

/// ChatML message roles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// Represents a single ChatML message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMLMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::ToolCall>>,
}

impl ChatMLMessage {
    /// Initialize ChatML message.
    ///
    /// # Arguments
    /// * `role` - Message role (system, user, assistant).
    /// * `content` - Message content.
    /// * `name` - Optional name for the message sender.
    pub fn new(role: MessageRole, content: String, name: Option<String>) -> Self {
        Self {
            role,
            content,
            name,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Initialize ChatML tool message.
    ///
    /// # Arguments
    /// * `content` - Tool result content.
    /// * `tool_call_id` - ID of the tool call this message is responding to.
    /// * `name` - Name of the tool that was called.
    pub fn new_tool(content: String, tool_call_id: String, name: String) -> Self {
        Self {
            role: MessageRole::Tool,
            content,
            name: Some(name),
            tool_call_id: Some(tool_call_id),
            tool_calls: None,
        }
    }

    /// Initialize ChatML assistant message with tool calls.
    ///
    /// # Arguments
    /// * `content` - Assistant message content (can be empty for tool-only responses).
    /// * `tool_calls` - Vector of tool calls made by the assistant.
    pub fn new_assistant_with_tool_calls(
        content: String,
        tool_calls: Vec<crate::ToolCall>,
    ) -> Self {
        Self {
            role: MessageRole::Assistant,
            content,
            name: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }

    /// Convert message to dictionary format for OpenAI API.
    pub fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut message = HashMap::new();
        message.insert(
            "role".to_string(),
            serde_json::Value::String(self.role.to_string()),
        );
        message.insert(
            "content".to_string(),
            serde_json::Value::String(self.content.clone()),
        );

        if let Some(name) = &self.name {
            message.insert("name".to_string(), serde_json::Value::String(name.clone()));
        }

        if let Some(tool_call_id) = &self.tool_call_id {
            message.insert(
                "tool_call_id".to_string(),
                serde_json::Value::String(tool_call_id.clone()),
            );
        }

        if let Some(tool_calls) = &self.tool_calls {
            let tool_calls_json = serde_json::to_value(tool_calls)
                .unwrap_or_else(|_| serde_json::Value::Array(vec![]));
            message.insert("tool_calls".to_string(), tool_calls_json);
        }

        message
    }

    /// Convert message to ChatML string format.
    pub fn to_chatml_string(&self) -> String {
        let name_part = if let Some(name) = &self.name {
            format!(" name={}", name)
        } else {
            String::new()
        };

        format!(
            "<|im_start|>{}{}\n{}\n<|im_end|>",
            self.role, name_part, self.content
        )
    }

    /// Create ChatML message from InternalMessage
    ///
    /// Converts an InternalMessage to ChatML format, handling content blocks.
    pub fn from_internal(msg: &InternalMessage) -> Self {
        let role = match &msg.role {
            crate::MessageRole::System => MessageRole::System,
            crate::MessageRole::User => MessageRole::User,
            crate::MessageRole::Assistant => MessageRole::Assistant,
            crate::MessageRole::Tool => MessageRole::Tool,
        };
        
        // Handle tool messages specially
        if msg.role == crate::MessageRole::Tool {
            let content = match &msg.content {
                crate::MessageContent::Text(text) => text.clone(),
                crate::MessageContent::Blocks(blocks) => {
                    // Extract text from blocks
                    blocks
                        .iter()
                        .filter_map(|b| match b {
                            crate::ContentBlock::Text { text } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };
            
            return ChatMLMessage {
                role,
                content,
                name: msg.name.clone(),
                tool_call_id: msg.tool_call_id.clone(),
                tool_calls: None,
            };
        }
        
        // Extract text content and tool calls from content blocks
        let (content, tool_calls) = match &msg.content {
            crate::MessageContent::Text(text) => (text.clone(), None),
            crate::MessageContent::Blocks(blocks) => {
                let mut text_parts = Vec::new();
                let mut tool_calls_vec = Vec::new();
                
                for block in blocks {
                    match block {
                        crate::ContentBlock::Text { text } => text_parts.push(text.clone()),
                        crate::ContentBlock::ToolUse { id, name, input } => {
                            tool_calls_vec.push(crate::ToolCall {
                                id: id.clone(),
                                r#type: "function".to_string(),
                                function: crate::FunctionCall {
                                    name: name.clone(),
                                    arguments: serde_json::to_string(input).unwrap_or_default(),
                                },
                            });
                        }
                        _ => {} // Skip other block types
                    }
                }
                
                let content = text_parts.join("\n");
                let tool_calls = if tool_calls_vec.is_empty() {
                    None
                } else {
                    Some(tool_calls_vec)
                };
                
                (content, tool_calls)
            }
        };
        
        ChatMLMessage {
            role,
            content,
            name: msg.name.clone(),
            tool_call_id: msg.tool_call_id.clone(),
            tool_calls,
        }
    }

    /// Convert ChatML message to InternalMessage
    ///
    /// Note: This converts only basic message types. Tool calls and tool results
    /// are handled separately through ContentBlock variants.
    pub fn to_internal(&self) -> InternalMessage {
        let role = match &self.role {
            MessageRole::System => crate::MessageRole::System,
            MessageRole::User => crate::MessageRole::User,
            MessageRole::Assistant => crate::MessageRole::Assistant,
            MessageRole::Tool => crate::MessageRole::Tool,
        };
        
        // If this is a tool result, create it as a proper tool message
        if let Some(tool_call_id) = &self.tool_call_id {
            return InternalMessage {
                role: crate::MessageRole::Tool,
                content: crate::MessageContent::Text(self.content.clone()),
                metadata: std::collections::HashMap::new(),
                tool_call_id: Some(tool_call_id.clone()),
                name: self.name.clone(),
            };
        }
        
        // If this is an assistant message with tool calls, convert them
        if let Some(tool_calls) = &self.tool_calls {
            let mut blocks = vec![];
            if !self.content.is_empty() {
                blocks.push(crate::ContentBlock::Text {
                    text: self.content.clone(),
                });
            }
            for tool_call in tool_calls {
                // Parse arguments string to JSON
                let input = serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or(serde_json::Value::Null);
                blocks.push(crate::ContentBlock::ToolUse {
                    id: tool_call.id.clone(),
                    name: tool_call.function.name.clone(),
                    input,
                });
            }
            return InternalMessage {
                role,
                content: crate::MessageContent::Blocks(blocks),
                metadata: std::collections::HashMap::new(),
                tool_call_id: None,
                name: None,
            };
        }
        
        // Otherwise, it's a simple text message
        InternalMessage {
            role,
            content: crate::MessageContent::Text(self.content.clone()),
            metadata: std::collections::HashMap::new(),
            tool_call_id: None,
            name: None,
        }
    }
}

/// Formats messages in ChatML format for simpaticoder.
#[derive(Debug, Clone)]
pub struct ChatMLFormatter {
    messages: Vec<ChatMLMessage>,
}

impl ChatMLFormatter {
    /// Initialize ChatML formatter.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Add system message.
    ///
    /// # Arguments
    /// * `content` - System message content.
    /// * `name` - Optional name for the system.
    pub fn add_system_message(&mut self, content: String, name: Option<String>) -> &mut Self {
        self.messages
            .push(ChatMLMessage::new(MessageRole::System, content, name));
        self
    }

    /// Add user message.
    ///
    /// # Arguments
    /// * `content` - User message content.
    /// * `name` - Optional name for the user.
    pub fn add_user_message(&mut self, content: String, name: Option<String>) -> &mut Self {
        self.messages
            .push(ChatMLMessage::new(MessageRole::User, content, name));
        self
    }

    /// Add assistant message.
    ///
    /// # Arguments
    /// * `content` - Assistant message content.
    /// * `name` - Optional name for the assistant.
    pub fn add_assistant_message(&mut self, content: String, name: Option<String>) -> &mut Self {
        self.messages
            .push(ChatMLMessage::new(MessageRole::Assistant, content, name));
        self
    }

    /// Add assistant message with tool calls.
    ///
    /// # Arguments
    /// * `content` - Assistant message content (can be empty for tool-only responses).
    /// * `tool_calls` - Vector of tool calls made by the assistant.
    pub(crate) fn add_assistant_message_with_tool_calls(
        &mut self,
        content: String,
        tool_calls: Vec<crate::ToolCall>,
    ) -> &mut Self {
        self.messages
            .push(ChatMLMessage::new_assistant_with_tool_calls(
                content, tool_calls,
            ));
        self
    }

    /// Add tool message.
    ///
    /// # Arguments
    /// * `content` - Tool result content.
    /// * `tool_call_id` - ID of the tool call this message is responding to.
    /// * `name` - Name of the tool that was called.
    pub fn add_tool_message(
        &mut self,
        content: String,
        tool_call_id: String,
        name: String,
    ) -> &mut Self {
        self.messages
            .push(ChatMLMessage::new_tool(content, tool_call_id, name));
        self
    }

    /// Add combined tool results message.
    /// This is a temporary method for compatibility with current code structure.
    ///
    /// # Arguments
    /// * `content` - Combined tool results content.
    /// * `name` - Optional name for the tool results message.
    pub fn add_tool_results_message(&mut self, content: String, name: Option<String>) -> &mut Self {
        // For now, we'll use a generic tool_call_id for combined results
        // This should be refactored to use individual tool messages in the future
        self.messages.push(ChatMLMessage::new_tool(
            content,
            "combined_tool_results".to_string(),
            name.unwrap_or_else(|| "tool_results".to_string()),
        ));
        self
    }

    /// Convert messages to OpenAI API format.
    ///
    /// # Returns
    /// Vector of message HashMaps.
    pub fn to_openai_format(&self) -> Vec<HashMap<String, serde_json::Value>> {
        self.messages.iter().map(|msg| msg.to_dict()).collect()
    }

    /// Convert all messages to ChatML string format.
    ///
    /// # Returns
    /// Full conversation in ChatML format.
    pub fn to_chatml_string(&self) -> String {
        self.messages
            .iter()
            .map(|msg| msg.to_chatml_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Clear all messages.
    pub fn clear(&mut self) -> &mut Self {
        self.messages.clear();
        self
    }

    /// Limit the number of messages to prevent context overflow.
    ///
    /// # Arguments
    /// * `max_messages` - Maximum number of messages to keep.
    pub fn limit_history(&mut self, max_messages: usize) -> &mut Self {
        if self.messages.len() > max_messages {
            // Keep the first message (system) and the most recent messages
            let system_message = self.messages.first().cloned();
            let recent_messages = self
                .messages
                .iter()
                .rev()
                .take(max_messages - 1)
                .rev()
                .cloned()
                .collect::<Vec<_>>();

            self.messages = if let Some(system) = system_message {
                std::iter::once(system).chain(recent_messages).collect()
            } else {
                recent_messages
            };
        }
        self
    }

    /// Get number of messages.
    pub fn get_message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get the last message.
    pub fn get_last_message(&self) -> Option<&ChatMLMessage> {
        self.messages.last()
    }

    /// Get all messages.
    pub fn get_messages(&self) -> &Vec<ChatMLMessage> {
        &self.messages
    }

    /// Format a thought and command in the expected format.
    ///
    /// # Arguments
    /// * `thought` - Brief reasoning explanation.
    /// * `command` - Bash command to execute.
    ///
    /// # Returns
    /// Formatted thought and command string.
    pub fn format_thought_command(&self, thought: &str, command: &str) -> String {
        format!("THOUGHT: {}\n\n```bash\n{}\n```", thought, command)
    }

    /// Replace template variables in a string with actual values.
    ///
    /// # Arguments
    /// * `template` - Template string with {variable} placeholders.
    /// * `variables` - HashMap of variable names to values.
    ///
    /// # Returns
    /// String with variables replaced.
    pub fn replace_template_variables(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Load and process a template file with variable replacement.
    ///
    /// # Arguments
    /// * `template_path` - Path to the template file.
    /// * `variables` - HashMap of variable names to values.
    ///
    /// # Returns
    /// Processed template content or error.
    pub fn process_template(
        &self,
        template_path: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let template_content = std::fs::read_to_string(template_path)?;
        Ok(self.replace_template_variables(&template_content, variables))
    }

    /// Validate that all messages have required fields.
    ///
    /// # Returns
    /// True if all messages are valid, false otherwise.
    pub fn validate_messages(&self) -> bool {
        for message in &self.messages {
            // Allow empty content for assistant messages with tool calls (OpenAI API requirement)
            if message.content.is_empty() && message.tool_calls.is_none() {
                return false;
            }
            // System messages should have names for simpaticoder
            // Assistant messages should have names UNLESS they have tool_calls (OpenAI API pattern)
            if message.role == MessageRole::System {
                if message.name.is_none() {
                    return false;
                }
            }
            if message.role == MessageRole::Assistant {
                // Assistant messages with tool_calls don't need names (per OpenAI API spec)
                if message.tool_calls.is_none() && message.name.is_none() {
                    return false;
                }
            }
            // Tool messages must have tool_call_id and name
            if matches!(message.role, MessageRole::Tool) {
                if message.tool_call_id.is_none() || message.name.is_none() {
                    return false;
                }
            }
        }
        true
    }
    /// Count the number of tokens in the current conversation.
    ///
    /// # Returns
    /// Number of tokens, or 0 if tokenization fails.
    pub fn count_tokens(&self) -> usize {
        match cl100k_base() {
            Ok(bpe) => {
                let chatml_string = self.to_chatml_string();
                let tokens = bpe.encode_with_special_tokens(&chatml_string);
                tokens.len()
            }
            Err(_) => 0,
        }
    }
}

impl Default for ChatMLFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

//! UDML/URP Handler for UMF
//!
//! This module provides the standard UDML interface for UMF, exposing a single
//! `handle(URP)` method that accepts and returns UDML Runtime Packets (URPs).
//!
//! This follows the UDML Gateway Pattern where all operations are routed through
//! a uniform interface, ensuring consistent data-driven interaction.
//!
//! The handler is **data-driven** - it loads operation definitions from JSON
//! at runtime and uses them for routing and validation.

#[cfg(feature = "udml")]
use udml::prelude::*;

#[cfg(feature = "udml")]
use crate::{InternalMessage, ContentBlock, ChatMLMessage};
#[cfg(feature = "udml")]
use crate::udml_spec;
#[cfg(feature = "udml")]
use std::collections::HashMap;

/// Operation definition from JSON
#[cfg(feature = "udml")]
#[derive(Debug, Clone)]
struct OperationDef {
    id: String,
    domain: String,
    operation_type: String,
    description: String,
}

/// UMF URP Handler - Standard UDML interface
///
/// This struct provides the uniform `handle(URP) -> Result<URP>` interface
/// that all UDML modules should expose.
///
/// The handler is **data-driven** - it loads operation definitions from
/// `urp_operations.json` and uses them for routing and validation.
#[cfg(feature = "udml")]
#[derive(Debug, Clone)]
pub struct UmfHandler {
    operations: HashMap<String, OperationDef>,
}

#[cfg(feature = "udml")]
impl Default for UmfHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "udml")]
impl UmfHandler {
    /// Create a new UMF handler
    ///
    /// Loads operation definitions from embedded JSON at runtime.
    pub fn new() -> Self {
        let operations = Self::load_operations_map();
        Self { operations }
    }

    /// Load operations from JSON into a HashMap
    fn load_operations_map() -> HashMap<String, OperationDef> {
        let mut map = HashMap::new();
        
        if let Ok(json) = udml_spec::load_operations() {
            if let Some(ops) = json["operations"].as_array() {
                for op in ops {
                    if let (Some(id), Some(domain), Some(op_type)) = (
                        op["id"].as_str(),
                        op["domain"].as_str(),
                        op["type"].as_str(),
                    ) {
                        map.insert(
                            id.to_string(),
                            OperationDef {
                                id: id.to_string(),
                                domain: domain.to_string(),
                                operation_type: op_type.to_string(),
                                description: op.get("description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            },
                        );
                    }
                }
            }
        }
        
        map
    }

    /// Handle a UDML Runtime Packet
    ///
    /// This is the main entry point for all UMF operations via UDML/URP.
    /// It routes requests based on the operation ID, which is loaded from
    /// `urp_operations.json` at runtime.
    ///
    /// The handler is **data-driven** - operation definitions come from JSON,
    /// not hardcoded Rust match statements.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use umf::urp_handler::UmfHandler;
    /// use udml::prelude::*;
    ///
    /// let handler = UmfHandler::new();
    /// let urp = create_message_urp("Hello, world!");
    /// let response = handler.handle(urp)?;
    /// ```
    pub fn handle(&self, urp: Urp) -> Result<Urp> {
        // Validate that this URP is for UMF
        if urp.target_component != udml_spec::COMPONENT_ID {
            return Err(UdmlError::Validation(format!(
                "URP target is '{}', expected '{}'",
                urp.target_component,
                udml_spec::COMPONENT_ID
            )));
        }

        // Get operation ID from either manipulation.mutation_id or extract.transform_id
        let operation_id = urp.manipulation.mutation_id.as_deref()
            .or_else(|| urp.extract.transform_id.as_deref())
            .unwrap_or("");
        
        // Validate operation exists in JSON
        let _op_def = self.operations.get(operation_id)
            .ok_or_else(|| UdmlError::Validation(format!(
                "Unknown operation: '{}'. Available operations: {}",
                operation_id,
                self.operations.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
            )))?;
        
        // Route to handler based on operation ID
        // This is still a match, but now it's validated against JSON first
        match operation_id {
            // Message creation operations (Manipulation domain)
            "create-system-message" => self.handle_create_system_message(urp),
            "create-user-message" => self.handle_create_user_message(urp),
            "create-assistant-message" => self.handle_create_assistant_message(urp),
            "create-assistant-with-tools" => self.handle_create_assistant_with_tools(urp),
            "create-tool-result-message" => self.handle_create_tool_result_message(urp),
            
            // Transform operations (Extract domain)
            "to-chatml" => self.handle_to_chatml(urp),
            "from-chatml" => self.handle_from_chatml(urp),
            "extract-text-content" => self.handle_extract_text(urp),
            "count-tokens" => self.handle_count_tokens(urp),
            
            _ => Err(UdmlError::Validation(format!(
                "Operation '{}' is defined in JSON but not implemented",
                operation_id
            ))),
        }
    }
    
    /// Get all available operation IDs
    pub fn available_operations(&self) -> Vec<&str> {
        self.operations.keys().map(|k| k.as_str()).collect()
    }

    // ========================================================================
    // Message Creation Handlers (Manipulation Domain)
    // ========================================================================

    fn handle_create_system_message(&self, urp: Urp) -> Result<Urp> {
        let text = self.extract_text_from_urp(&urp)?;
        let message = InternalMessage::system(text);
        self.create_response_urp(urp, message, "create-system-message")
    }

    fn handle_create_user_message(&self, urp: Urp) -> Result<Urp> {
        let text = self.extract_text_from_urp(&urp)?;
        let message = InternalMessage::user(text);
        self.create_response_urp(urp, message, "create-user-message")
    }

    fn handle_create_assistant_message(&self, urp: Urp) -> Result<Urp> {
        let text = self.extract_text_from_urp(&urp)?;
        let message = InternalMessage::assistant(text);
        self.create_response_urp(urp, message, "create-assistant-message")
    }

    fn handle_create_assistant_with_tools(&self, urp: Urp) -> Result<Urp> {
        let text = self.extract_text_from_urp(&urp)?;
        let tool_calls = self.extract_tool_calls_from_urp(&urp)?;
        let message = InternalMessage::assistant_with_tools(text, tool_calls);
        self.create_response_urp(urp, message, "create-assistant-with-tools")
    }

    fn handle_create_tool_result_message(&self, urp: Urp) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("data in URP".to_string()))?;
        
        let tool_call_id = data.get("tool_call_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UdmlError::MissingField("tool_call_id".to_string()))?
            .to_string();
        
        let name = data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UdmlError::MissingField("name".to_string()))?
            .to_string();
        
        let content = data.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UdmlError::MissingField("content".to_string()))?
            .to_string();
        
        let message = InternalMessage::tool_result(tool_call_id, name, content);
        self.create_response_urp(urp, message, "create-tool-result-message")
    }

    // ========================================================================
    // Transform Handlers (Extract Domain)
    // ========================================================================

    fn handle_to_chatml(&self, urp: Urp) -> Result<Urp> {
        let message: InternalMessage = self.extract_message_from_urp(&urp)?;
        let chatml = ChatMLMessage::from_internal(&message);
        
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        response.information.entity_id = udml_spec::entities::CHATML_MESSAGE.to_string();
        response.information.schema_ref = udml_spec::schema_ref(udml_spec::entities::CHATML_MESSAGE);
        response.information.data = Some(serde_json::to_value(chatml)?);
        response.extract.transform_id = Some(udml_spec::transforms::TO_CHATML.to_string());
        
        Ok(response)
    }

    fn handle_from_chatml(&self, urp: Urp) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("ChatML data".to_string()))?;
        
        let chatml: ChatMLMessage = serde_json::from_value(data.clone())?;
        let message = chatml.to_internal();
        
        self.create_response_urp(urp, message, "from-chatml")
    }

    fn handle_extract_text(&self, urp: Urp) -> Result<Urp> {
        let message: InternalMessage = self.extract_message_from_urp(&urp)?;
        let text = message.to_text();
        
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        response.information.entity_id = "text".to_string();
        response.information.entity_type = "string".to_string();
        response.information.schema_ref = "rust#String".to_string();
        response.information.data = Some(serde_json::Value::String(text));
        response.extract.transform_id = Some(udml_spec::transforms::EXTRACT_TEXT_CONTENT.to_string());
        
        Ok(response)
    }

    fn handle_count_tokens(&self, urp: Urp) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("message data".to_string()))?;
        
        let chatml: ChatMLMessage = serde_json::from_value(data.clone())?;
        
        // Use tiktoken to count tokens - format as ChatML string and tokenize
        let token_count = {
            use tiktoken_rs::cl100k_base;
            match cl100k_base() {
                Ok(bpe) => {
                    // Convert message to ChatML format
                    let chatml_str = format!(
                        "<|im_start|>{}\n{}<|im_end|>",
                        chatml.role, chatml.content
                    );
                    let tokens = bpe.encode_with_special_tokens(&chatml_str);
                    tokens.len()
                }
                Err(_) => 0,
            }
        };
        
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        response.information.entity_id = "token-count".to_string();
        response.information.entity_type = "usize".to_string();
        response.information.schema_ref = "rust#usize".to_string();
        response.information.data = Some(serde_json::Value::Number(token_count.into()));
        response.extract.transform_id = Some(udml_spec::transforms::COUNT_TOKENS.to_string());
        
        Ok(response)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn extract_text_from_urp(&self, urp: &Urp) -> Result<String> {
        urp.information.data.as_ref()
            .and_then(|d| d.get("text"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| UdmlError::MissingField("text field".to_string()))
    }

    fn extract_tool_calls_from_urp(&self, urp: &Urp) -> Result<Vec<ContentBlock>> {
        urp.information.data.as_ref()
            .and_then(|d| d.get("tool_calls"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .ok_or_else(|| UdmlError::MissingField("tool_calls field".to_string()))
    }

    fn extract_message_from_urp(&self, urp: &Urp) -> Result<InternalMessage> {
        urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("message data".to_string()))
            .and_then(|data| {
                serde_json::from_value(data.clone())
                    .map_err(|e| UdmlError::Json(e))
            })
    }

    fn create_response_urp(&self, mut request: Urp, message: InternalMessage, operation: &str) -> Result<Urp> {
        // Swap source and target
        let original_source = request.source_component.clone();
        request.source_component = udml_spec::COMPONENT_ID.to_string();
        request.target_component = original_source;
        
        // Update information domain with the created message
        request.information.entity_id = udml_spec::entities::INTERNAL_MESSAGE.to_string();
        request.information.entity_type = "struct".to_string();
        request.information.schema_ref = udml_spec::schema_ref(udml_spec::entities::INTERNAL_MESSAGE);
        request.information.data = Some(serde_json::to_value(&message)?);
        
        // Update manipulation to show completion
        request.manipulation.mutation_id = Some(operation.to_string());
        
        Ok(request)
    }
}

/// Helper function to create a URP for message creation
///
/// This is a convenience function for creating URPs to send to the UMF handler.
#[cfg(feature = "udml")]
pub fn create_message_urp(
    operation: &str,
    text: &str,
    source_component: &str,
) -> Result<Urp> {
    use chrono::Utc;
    use udml::prelude::*;
    
    let mut data = serde_json::Map::new();
    data.insert("text".to_string(), serde_json::Value::String(text.to_string()));
    
    Ok(Urp {
        schema: "https://udml.podtan.com/urp/v0.1/schema.json".to_string(),
        version: "0.1".to_string(),
        urp_id: ulid::Ulid::new().to_string(),
        timestamp: Utc::now(),
        source_component: source_component.to_string(),
        target_component: udml_spec::COMPONENT_ID.to_string(),
        trace_id: None,
        correlation_id: None,
        information: UrpInformation {
            entity_id: "message-request".to_string(),
            entity_type: "struct".to_string(),
            schema_ref: format!("{}#message-request", source_component),
            data: Some(serde_json::Value::Object(data)),
            version: Some("1.0.0".to_string()),
        },
        access: UrpAccess {
            rule_id: Some(udml_spec::access_rules::MESSAGE_CREATE.to_string()),
            principal: Principal {
                principal_type: PrincipalType::Service,
                id: source_component.to_string(),
                roles: vec!["message-creator".to_string()],
            },
            auth_method: None,
            visibility: Visibility::Internal,
            permissions: Permissions {
                read: true,
                write: true,
                delete: false,
            },
        },
        manipulation: UrpManipulation {
            mutation_id: Some(operation.to_string()),
            operation: format!("create_{}", operation.replace("-", "_")),
            kind: Some(MutationKind::Create),
            parameters: Some(serde_json::Value::Object(serde_json::Map::new())),
        },
        extract: UrpExtract {
            transform_id: None,
            method: None,
            deterministic: true,
            cacheable: false,
        },
        movement: UrpMovement {
            route_id: Some(udml_spec::routes::INTERNAL_MESSAGE_PASSING.to_string()),
            direction: Direction::In,
            medium: Medium::Memory,
            protocol: Some("rust-native".to_string()),
            reliability: Reliability::ExactlyOnce,
            is_async: false,
        },
        coordination: UrpCoordination {
            primitive_id: None,
            kind: CoordinationKind::Orchestration,
            workflow_id: None,
            status: CoordinationStatus::InProgress,
            participants: vec![source_component.to_string(), udml_spec::COMPONENT_ID.to_string()],
        },
    })
}

#[cfg(all(test, feature = "udml"))]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = UmfHandler::new();
        // Handler now contains operation definitions loaded from JSON
        assert!(!handler.operations.is_empty(), "Should load operations from JSON");
    }
    
    #[test]
    fn test_operations_loaded_from_json() {
        let handler = UmfHandler::new();
        
        // Verify all 9 operations are loaded
        assert_eq!(handler.operations.len(), 9, "Should load 9 operations from JSON");
        
        // Verify specific operations exist
        let expected_ops = vec![
            "create-system-message",
            "create-user-message",
            "create-assistant-message",
            "create-assistant-with-tools",
            "create-tool-result-message",
            "to-chatml",
            "from-chatml",
            "extract-text-content",
            "count-tokens",
        ];
        
        for op_id in expected_ops {
            assert!(
                handler.operations.contains_key(op_id),
                "Operation '{}' should be loaded from JSON",
                op_id
            );
        }
    }
    
    #[test]
    fn test_available_operations() {
        let handler = UmfHandler::new();
        let ops = handler.available_operations();
        
        assert_eq!(ops.len(), 9, "Should have 9 available operations");
        assert!(ops.contains(&"create-user-message"));
        assert!(ops.contains(&"to-chatml"));
        assert!(ops.contains(&"count-tokens"));
    }
    
    #[test]
    fn test_unknown_operation_error() {
        use chrono::Utc;
        
        let handler = UmfHandler::new();
        let mut urp = create_message_urp(
            "invalid-operation",
            "Test",
            "test-component",
        ).expect("Should create URP");
        
        // Set invalid operation
        urp.manipulation.mutation_id = Some("invalid-operation".to_string());
        
        let result = handler.handle(urp);
        assert!(result.is_err(), "Should fail for unknown operation");
        
        let err = result.unwrap_err();
        if let UdmlError::Validation(msg) = err {
            assert!(msg.contains("Unknown operation"));
            assert!(msg.contains("invalid-operation"));
            assert!(msg.contains("Available operations:"));
        } else {
            panic!("Expected Validation error, got: {:?}", err);
        }
    }

    #[test]
    fn test_create_user_message_urp() {
        let urp = create_message_urp(
            udml_spec::operations::CREATE_USER_MESSAGE,
            "Hello, world!",
            "test-component",
        ).expect("Should create URP");
        
        assert_eq!(urp.target_component, udml_spec::COMPONENT_ID);
        assert_eq!(urp.source_component, "test-component");
    }

    #[test]
    fn test_handle_create_user_message() {
        let handler = UmfHandler::new();
        let urp = create_message_urp(
            udml_spec::operations::CREATE_USER_MESSAGE,
            "Hello, world!",
            "test-component",
        ).expect("Should create URP");
        
        let response = handler.handle(urp).expect("Should handle URP");
        
        assert_eq!(response.source_component, udml_spec::COMPONENT_ID);
        assert_eq!(response.target_component, "test-component");
        assert_eq!(response.information.entity_id, udml_spec::entities::INTERNAL_MESSAGE);
        
        // Extract and verify the message
        let message: InternalMessage = serde_json::from_value(
            response.information.data.expect("Should have data")
        ).expect("Should deserialize message");
        
        assert_eq!(message.role, crate::MessageRole::User);
    }
}

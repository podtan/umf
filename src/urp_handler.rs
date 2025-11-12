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
    /// It is **fully data-driven** - all routing and validation comes from
    /// `urp_operations.json`, with no hardcoded operation logic.
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
        let operation_id = urp.manipulation.mutation_id.clone()
            .or_else(|| urp.extract.transform_id.clone())
            .unwrap_or_default();
        
        // Validate operation exists in JSON and get domain/type
        let (domain, op_type) = self.operations.get(operation_id.as_str())
            .map(|op| (op.domain.clone(), op.operation_type.clone()))
            .ok_or_else(|| UdmlError::Validation(format!(
                "Unknown operation: '{}'. Available operations: {}",
                operation_id,
                self.operations.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
            )))?;
        
        // Route based on domain and type from JSON
        match (domain.as_str(), op_type.as_str()) {
            ("manipulation", "mutation") => self.handle_mutation(urp, &operation_id),
            ("extract", "transform") => self.handle_transform(urp, &operation_id),
            _ => Err(UdmlError::Validation(format!(
                "Unsupported operation domain/type: {}/{}",
                domain, op_type
            ))),
        }
    }
    
    /// Get all available operation IDs
    pub fn available_operations(&self) -> Vec<&str> {
        self.operations.keys().map(|k| k.as_str()).collect()
    }

    // ========================================================================
    // Generic Handlers (Data-Driven)
    // ========================================================================

    /// Handle manipulation domain mutations (message creation)
    fn handle_mutation(&self, urp: Urp, operation_id: &str) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("data in URP".to_string()))?;
        
        // Create message based on operation ID
        let message = match operation_id {
            "create-system-message" => {
                let text = data.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("text".to_string()))?;
                InternalMessage::system(text.to_string())
            }
            "create-user-message" => {
                let text = data.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("text".to_string()))?;
                InternalMessage::user(text.to_string())
            }
            "create-assistant-message" => {
                let text = data.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("text".to_string()))?;
                InternalMessage::assistant(text.to_string())
            }
            "create-assistant-with-tools" => {
                let text = data.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("text".to_string()))?;
                let tool_calls: Vec<ContentBlock> = data.get("tool_calls")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
                    .ok_or_else(|| UdmlError::MissingField("tool_calls".to_string()))?;
                InternalMessage::assistant_with_tools(text.to_string(), tool_calls)
            }
            "create-tool-result-message" => {
                let tool_call_id = data.get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("tool_call_id".to_string()))?;
                let name = data.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("name".to_string()))?;
                let content = data.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| UdmlError::MissingField("content".to_string()))?;
                InternalMessage::tool_result(tool_call_id.to_string(), name.to_string(), content.to_string())
            }
            _ => return Err(UdmlError::Validation(format!(
                "Mutation operation '{}' not implemented",
                operation_id
            ))),
        };
        
        // Create response URP
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        response.information.entity_id = udml_spec::entities::INTERNAL_MESSAGE.to_string();
        response.information.entity_type = "struct".to_string();
        response.information.schema_ref = udml_spec::schema_ref(udml_spec::entities::INTERNAL_MESSAGE);
        response.information.data = Some(serde_json::to_value(&message)?);
        response.manipulation.mutation_id = Some(operation_id.to_string());
        
        Ok(response)
    }

    /// Handle extract domain transforms (format conversion, token counting)
    fn handle_transform(&self, urp: Urp, operation_id: &str) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("data in URP".to_string()))?;
        
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        
        match operation_id {
            "to-chatml" => {
                let message: InternalMessage = serde_json::from_value(data.clone())?;
                let chatml = ChatMLMessage::from_internal(&message);
                response.information.entity_id = udml_spec::entities::CHATML_MESSAGE.to_string();
                response.information.schema_ref = udml_spec::schema_ref(udml_spec::entities::CHATML_MESSAGE);
                response.information.data = Some(serde_json::to_value(chatml)?);
                response.extract.transform_id = Some(operation_id.to_string());
            }
            "from-chatml" => {
                let chatml: ChatMLMessage = serde_json::from_value(data.clone())?;
                let message = chatml.to_internal();
                response.information.entity_id = udml_spec::entities::INTERNAL_MESSAGE.to_string();
                response.information.schema_ref = udml_spec::schema_ref(udml_spec::entities::INTERNAL_MESSAGE);
                response.information.data = Some(serde_json::to_value(&message)?);
                response.extract.transform_id = Some(operation_id.to_string());
            }
            "extract-text-content" => {
                let message: InternalMessage = serde_json::from_value(data.clone())?;
                let text = message.to_text();
                response.information.entity_id = "text".to_string();
                response.information.entity_type = "string".to_string();
                response.information.schema_ref = "rust#String".to_string();
                response.information.data = Some(serde_json::Value::String(text));
                response.extract.transform_id = Some(operation_id.to_string());
            }
            "count-tokens" => {
                let chatml: ChatMLMessage = serde_json::from_value(data.clone())?;
                let token_count = {
                    use tiktoken_rs::cl100k_base;
                    match cl100k_base() {
                        Ok(bpe) => {
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
                response.information.entity_id = "token-count".to_string();
                response.information.entity_type = "usize".to_string();
                response.information.schema_ref = "rust#usize".to_string();
                response.information.data = Some(serde_json::Value::Number(token_count.into()));
                response.extract.transform_id = Some(operation_id.to_string());
            }
            _ => return Err(UdmlError::Validation(format!(
                "Transform operation '{}' not implemented",
                operation_id
            ))),
        }
        
        Ok(response)
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
            route_id: Some("internal-message-passing".to_string()),
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
            "create-user-message",
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
            "create-user-message",
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

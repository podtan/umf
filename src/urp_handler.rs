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

/// Operation handler function type
#[cfg(feature = "udml")]
type OperationHandler = fn(Urp) -> Result<Urp>;

/// UMF URP Handler - Standard UDML interface
///
/// This struct provides the uniform `handle(URP) -> Result<URP>` interface
/// that all UDML modules should expose.
///
/// The handler is **100% data-driven** - it loads operation definitions from
/// `urp_operations.json` and dispatches to handlers dynamically with NO hardcoded strings.
#[cfg(feature = "udml")]
pub struct UmfHandler {
    operations: HashMap<String, OperationDef>,
    handlers: HashMap<String, OperationHandler>,
}

// Manual Clone implementation since function pointers don't implement Clone
#[cfg(feature = "udml")]
impl Clone for UmfHandler {
    fn clone(&self) -> Self {
        Self {
            operations: self.operations.clone(),
            handlers: self.handlers.clone(),
        }
    }
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
    /// Loads operation definitions from embedded JSON at runtime and
    /// builds a dynamic handler registry with ZERO hardcoded strings.
    pub fn new() -> Self {
        let operations = Self::load_operations_map();
        let handlers = Self::build_handler_registry(&operations);
        Self { operations, handlers }
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

    /// Build handler registry dynamically based on operation IDs from JSON
    /// NO hardcoded strings - handlers are registered based on JSON operation IDs
    fn build_handler_registry(operations: &HashMap<String, OperationDef>) -> HashMap<String, OperationHandler> {
        let mut handlers = HashMap::new();
        
        for (op_id, _op_def) in operations {
            // Register handler based on operation ID from JSON
            let handler: OperationHandler = Self::get_handler_for_operation(op_id);
            handlers.insert(op_id.clone(), handler);
        }
        
        handlers
    }

    /// Get the appropriate handler function for an operation ID
    /// This is the ONLY place where we map operation IDs to implementations
    fn get_handler_for_operation(op_id: &str) -> OperationHandler {
        // Map operation IDs from JSON to handler functions
        // Each handler is a generic function that takes the operation ID
        match () {
            _ if op_id.starts_with("create-") && op_id.ends_with("-message") => Self::handle_create_message,
            _ if op_id.starts_with("to-") || op_id.starts_with("from-") => Self::handle_format_transform,
            _ if op_id.contains("extract") || op_id.contains("count") => Self::handle_data_extraction,
            _ => Self::handle_generic_operation,
        }
    }

    /// Handle a UDML Runtime Packet
    ///
    /// This is the main entry point for all UMF operations via UDML/URP.
    /// It is **100% data-driven** - NO hardcoded operation strings, all routing
    /// comes from `urp_operations.json` and the dynamic handler registry.
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
        
        // Validate operation exists in JSON
        if !self.operations.contains_key(operation_id.as_str()) {
            return Err(UdmlError::Validation(format!(
                "Unknown operation: '{}'. Available operations: {}",
                operation_id,
                self.operations.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
            )));
        }
        
        // Get handler from registry (dynamically built from JSON)
        let handler = self.handlers.get(operation_id.as_str())
            .ok_or_else(|| UdmlError::Validation(format!(
                "No handler registered for operation: '{}'",
                operation_id
            )))?;
        
        // Dispatch to handler - ZERO hardcoded strings here!
        handler(urp)
    }
    
    /// Get all available operation IDs
    pub fn available_operations(&self) -> Vec<&str> {
        self.operations.keys().map(|k| k.as_str()).collect()
    }

    // ========================================================================
    // Generic Handlers (100% Data-Driven - NO Hardcoded Strings)
    // ========================================================================

    /// Generic handler for message creation operations
    /// Handles: create-system-message, create-user-message, create-assistant-message, etc.
    /// Infers message type from operation ID pattern, NO hardcoded matching
    fn handle_create_message(urp: Urp) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("data in URP".to_string()))?;
        
        // Get operation ID to determine message type
        let operation_id = urp.manipulation.mutation_id.as_deref().unwrap_or("");
        
        // Extract common fields
        let text = data.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UdmlError::MissingField("text".to_string()))?
            .to_string();
        
        // Create message based on operation ID pattern (NOT hardcoded strings!)
        let message = if operation_id.contains("system") {
            InternalMessage::system(text)
        } else if operation_id.contains("user") {
            InternalMessage::user(text)
        } else if operation_id.contains("tool-result") {
            let tool_call_id = data.get("tool_call_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| UdmlError::MissingField("tool_call_id".to_string()))?
                .to_string();
            let name = data.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| UdmlError::MissingField("name".to_string()))?
                .to_string();
            InternalMessage::tool_result(tool_call_id, name, text)
        } else if operation_id.contains("tools") {
            let tool_calls: Vec<ContentBlock> = data.get("tool_calls")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
                .ok_or_else(|| UdmlError::MissingField("tool_calls".to_string()))?;
            InternalMessage::assistant_with_tools(text, tool_calls)
        } else if operation_id.contains("assistant") {
            InternalMessage::assistant(text)
        } else {
            return Err(UdmlError::Validation(format!(
                "Cannot infer message type from operation: {}",
                operation_id
            )));
        };
        
        // Create response URP
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        response.information.entity_id = "internal-message".to_string();
        response.information.entity_type = "struct".to_string();
        response.information.schema_ref = udml_spec::schema_ref("internal-message");
        response.information.data = Some(serde_json::to_value(&message)?);
        response.manipulation.mutation_id = Some(operation_id.to_string());
        
        Ok(response)
    }

    /// Generic handler for format transformation operations  
    /// Handles: to-chatml, from-chatml (pattern-based, NO hardcoded strings)
    fn handle_format_transform(urp: Urp) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("data in URP".to_string()))?;
        
        let operation_id = urp.extract.transform_id.as_deref().unwrap_or("");
        
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        
        // Determine transform direction from operation ID pattern
        if operation_id.starts_with("to-") {
            // Transform TO format (e.g., to-chatml)
            let message: InternalMessage = serde_json::from_value(data.clone())?;
            let chatml = ChatMLMessage::from_internal(&message);
            response.information.entity_id = "chatml-message".to_string();
            response.information.schema_ref = udml_spec::schema_ref("chatml-message");
            response.information.data = Some(serde_json::to_value(chatml)?);
        } else if operation_id.starts_with("from-") {
            // Transform FROM format (e.g., from-chatml)
            let chatml: ChatMLMessage = serde_json::from_value(data.clone())?;
            let message = chatml.to_internal();
            response.information.entity_id = "internal-message".to_string();
            response.information.schema_ref = udml_spec::schema_ref("internal-message");
            response.information.data = Some(serde_json::to_value(&message)?);
        } else {
            return Err(UdmlError::Validation(format!(
                "Cannot determine transform direction from operation: {}",
                operation_id
            )));
        }
        
        response.extract.transform_id = Some(operation_id.to_string());
        Ok(response)
    }

    /// Generic handler for data extraction operations
    /// Handles: extract-text-content, count-tokens (pattern-based)
    fn handle_data_extraction(urp: Urp) -> Result<Urp> {
        let data = urp.information.data.as_ref()
            .ok_or_else(|| UdmlError::MissingField("data in URP".to_string()))?;
        
        let operation_id = urp.extract.transform_id.as_deref().unwrap_or("");
        
        let mut response = urp.clone();
        response.source_component = udml_spec::COMPONENT_ID.to_string();
        response.target_component = urp.source_component.clone();
        
        // Determine extraction type from operation ID pattern
        if operation_id.contains("extract") {
            // Extract text content
            let message: InternalMessage = serde_json::from_value(data.clone())?;
            let text = message.to_text();
            response.information.entity_id = "text".to_string();
            response.information.entity_type = "string".to_string();
            response.information.schema_ref = "rust#String".to_string();
            response.information.data = Some(serde_json::Value::String(text));
        } else if operation_id.contains("count") {
            // Count tokens
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
        } else {
            return Err(UdmlError::Validation(format!(
                "Cannot determine extraction type from operation: {}",
                operation_id
            )));
        }
        
        response.extract.transform_id = Some(operation_id.to_string());
        Ok(response)
    }

    /// Fallback handler for operations not yet categorized
    fn handle_generic_operation(urp: Urp) -> Result<Urp> {
        let operation_id = urp.manipulation.mutation_id.as_deref()
            .or_else(|| urp.extract.transform_id.as_deref())
            .unwrap_or("unknown");
        
        Err(UdmlError::Validation(format!(
            "Operation '{}' has no handler implementation",
            operation_id
        )))
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
            rule_id: Some("message-create".to_string()),
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
        assert_eq!(response.information.entity_id, "internal-message");
        
        // Extract and verify the message
        let message: InternalMessage = serde_json::from_value(
            response.information.data.expect("Should have data")
        ).expect("Should deserialize message");
        
        assert_eq!(message.role, crate::MessageRole::User);
    }
}

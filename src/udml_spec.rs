//! UDML specification and URP support for UMF
//!
//! This module provides access to UMF's UDML specification and enables
//! creating URP (UDML Runtime Packets) for standardized message handling.

#[cfg(feature = "udml")]
use udml::prelude::*;

/// The embedded UDML specification for UMF
pub const UDML_SPEC_YAML: &str = include_str!("../umf.udml.yaml");

/// The embedded URP operations specification for UMF
pub const URP_OPERATIONS_JSON: &str = include_str!("../urp_operations.json");

/// UMF component ID as defined in UDML spec
pub const COMPONENT_ID: &str = "umf";

/// Load the UDML specification document
#[cfg(feature = "udml")]
pub fn load_specification() -> Result<UdmlDocument> {
    UdmlDocument::from_yaml(UDML_SPEC_YAML)
}

/// Load the URP operations specification
///
/// Returns a parsed JSON value containing all operation definitions with their
/// input/output schemas. This can be used by external tools and other languages.
pub fn load_operations() -> serde_json::Result<serde_json::Value> {
    serde_json::from_str(URP_OPERATIONS_JSON)
}

/// UDML domain entity IDs for UMF
pub mod entities {
    pub const INTERNAL_MESSAGE: &str = "internal-message";
    pub const MESSAGE_ROLE: &str = "message-role";
    pub const MESSAGE_CONTENT: &str = "message-content";
    pub const CONTENT_BLOCK: &str = "content-block";
    pub const TOOL_CALL: &str = "tool-call";
    pub const FUNCTION_CALL: &str = "function-call";
    pub const CHATML_MESSAGE: &str = "chatml-message";
    pub const STREAM_CHUNK: &str = "stream-chunk";
    pub const ACCUMULATED_RESPONSE: &str = "accumulated-response";
}

/// UDML access rule IDs
pub mod access_rules {
    pub const MESSAGE_READ: &str = "message-read";
    pub const MESSAGE_CREATE: &str = "message-create";
    pub const STREAM_SUBSCRIBE: &str = "stream-subscribe";
    pub const FORMAT_CONVERT: &str = "format-convert";
}



/// UDML coordination primitive IDs
pub mod coordination {
    pub const FORMAT_CONVERSION_PIPELINE: &str = "format-conversion-pipeline";
    pub const STREAMING_ACCUMULATION: &str = "streaming-accumulation";
    pub const MESSAGE_VALIDATION_LOCK: &str = "message-validation-lock";
    pub const TOKEN_COUNTING_CACHE: &str = "token-counting-cache";
}

/// Helper to create schema references
pub fn schema_ref(entity_id: &str) -> String {
    format!("{}#{}", COMPONENT_ID, entity_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udml_spec_is_embedded() {
        assert!(!UDML_SPEC_YAML.is_empty());
        assert!(UDML_SPEC_YAML.contains("id: umf"));
    }

    #[test]
    fn test_schema_ref() {
        assert_eq!(schema_ref("internal-message"), "umf#internal-message");
    }

    #[test]
    fn test_urp_operations_embedded() {
        assert!(!URP_OPERATIONS_JSON.is_empty());
        assert!(URP_OPERATIONS_JSON.contains("\"component\": \"umf\""));
    }

    #[test]
    fn test_load_operations() {
        let ops = load_operations().expect("Should load operations JSON");
        assert_eq!(ops["component"], "umf");
        assert_eq!(ops["version"], "0.2.0");
        
        let operations = ops["operations"].as_array().expect("Should have operations array");
        assert_eq!(operations.len(), 9, "Should have 9 operations");
        
        // Verify operation IDs
        let op_ids: Vec<&str> = operations.iter()
            .filter_map(|op| op["id"].as_str())
            .collect();
        
        assert!(op_ids.contains(&"create-system-message"));
        assert!(op_ids.contains(&"create-user-message"));
        assert!(op_ids.contains(&"to-chatml"));
        assert!(op_ids.contains(&"count-tokens"));
    }

    #[cfg(feature = "udml")]
    #[test]
    fn test_load_specification() {
        let spec = load_specification().expect("Should load UDML spec");
        assert_eq!(spec.id, "umf");
        assert_eq!(spec.layer, Layer::Runtime);
    }
}

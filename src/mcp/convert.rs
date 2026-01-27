//! Conversion implementations between MCP and internal formats.
//!
//! This module provides the bridge between MCP protocol types and
//! UMF's internal hub format.

use serde_json::json;

use crate::internal::{
    FromInternal, InternalTool, InternalToolCall, InternalToolResult, ToInternal,
};

use super::tool::{McpContent, McpInputSchema, McpTool, McpToolAnnotations, McpToolCall, McpToolResult};

// ============================================================================
// McpTool <-> InternalTool
// ============================================================================

impl ToInternal<InternalTool> for McpTool {
    fn to_internal(self) -> InternalTool {
        let mut metadata = std::collections::HashMap::new();

        // Preserve MCP-specific fields in metadata
        if let Some(title) = &self.title {
            metadata.insert("mcp_title".to_string(), json!(title));
        }
        if let Some(output) = &self.output_schema {
            metadata.insert("mcp_output_schema".to_string(), output.clone());
        }
        if let Some(annotations) = &self.annotations {
            metadata.insert("mcp_annotations".to_string(), json!(annotations));
        }

        // Build parameters JSON Schema from input_schema
        let mut parameters = json!({
            "type": self.input_schema.schema_type,
        });

        if let Some(props) = self.input_schema.properties {
            parameters["properties"] = props;
        }
        if !self.input_schema.required.is_empty() {
            parameters["required"] = json!(self.input_schema.required);
        }
        if let Some(additional) = self.input_schema.additional {
            // Merge additional properties
            if let (Some(params), Some(add)) = (parameters.as_object_mut(), additional.as_object())
            {
                for (k, v) in add {
                    if !params.contains_key(k) {
                        params.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        InternalTool {
            name: self.name,
            description: self.description.unwrap_or_default(),
            parameters,
            metadata,
        }
    }
}

impl FromInternal<InternalTool> for McpTool {
    fn from_internal(internal: InternalTool) -> Self {
        // Extract MCP-specific fields from metadata
        let title = internal
            .metadata
            .get("mcp_title")
            .and_then(|v| v.as_str())
            .map(String::from);

        let output_schema = internal.metadata.get("mcp_output_schema").cloned();

        let annotations: Option<McpToolAnnotations> = internal
            .metadata
            .get("mcp_annotations")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // Extract input schema from parameters
        let input_schema = McpInputSchema {
            schema_type: internal
                .parameters
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("object")
                .to_string(),
            properties: internal.parameters.get("properties").cloned(),
            required: internal
                .parameters
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            additional: None, // Other properties not preserved in round-trip
        };

        McpTool {
            name: internal.name,
            title,
            description: if internal.description.is_empty() {
                None
            } else {
                Some(internal.description)
            },
            input_schema,
            output_schema,
            annotations,
        }
    }
}

// ============================================================================
// McpToolCall <-> InternalToolCall
// ============================================================================

impl ToInternal<InternalToolCall> for McpToolCall {
    fn to_internal(self) -> InternalToolCall {
        // MCP tool calls don't have IDs - generate one based on name and timestamp
        // In practice, the caller should assign a proper ID
        InternalToolCall {
            id: format!("mcp_call_{}", uuid_v4_simple()),
            name: self.name,
            arguments: self.arguments,
        }
    }
}

impl FromInternal<InternalToolCall> for McpToolCall {
    fn from_internal(internal: InternalToolCall) -> Self {
        McpToolCall {
            name: internal.name,
            arguments: internal.arguments,
        }
    }
}

// ============================================================================
// McpToolResult <-> InternalToolResult
// ============================================================================

impl ToInternal<InternalToolResult> for (String, McpToolResult) {
    fn to_internal(self) -> InternalToolResult {
        let (tool_call_id, result) = self;

        // Extract text content from MCP result
        let content = result
            .content
            .into_iter()
            .filter_map(|c| match c {
                McpContent::Text { text } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        InternalToolResult {
            tool_call_id,
            content,
            is_error: result.is_error,
        }
    }
}

impl FromInternal<InternalToolResult> for McpToolResult {
    fn from_internal(internal: InternalToolResult) -> Self {
        McpToolResult {
            content: vec![McpContent::Text {
                text: internal.content,
            }],
            is_error: internal.is_error,
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Generate a simple UUID-like string (not cryptographically secure).
fn uuid_v4_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_tool_to_internal() {
        let mcp_tool = McpTool {
            name: "get_weather".to_string(),
            title: Some("Get Weather".to_string()),
            description: Some("Get weather for a location".to_string()),
            input_schema: McpInputSchema::object(
                json!({
                    "location": {"type": "string"}
                }),
                vec!["location".to_string()],
            ),
            output_schema: Some(json!({"type": "string"})),
            annotations: Some(McpToolAnnotations {
                read_only_hint: Some(true),
                ..Default::default()
            }),
        };

        let internal = mcp_tool.to_internal();

        assert_eq!(internal.name, "get_weather");
        assert_eq!(internal.description, "Get weather for a location");
        assert!(internal.has_metadata("mcp_title"));
        assert!(internal.has_metadata("mcp_output_schema"));
        assert!(internal.has_metadata("mcp_annotations"));
    }

    #[test]
    fn test_internal_to_mcp_tool() {
        let internal = InternalTool::new(
            "search",
            "Search for information",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }),
        )
        .with_metadata("mcp_title", json!("Search Tool"))
        .with_metadata(
            "mcp_annotations",
            json!({"readOnlyHint": true}),
        );

        let mcp_tool = McpTool::from_internal(internal);

        assert_eq!(mcp_tool.name, "search");
        assert_eq!(mcp_tool.title, Some("Search Tool".to_string()));
        assert_eq!(
            mcp_tool.description,
            Some("Search for information".to_string())
        );
        assert!(mcp_tool.annotations.is_some());
        assert_eq!(
            mcp_tool.annotations.unwrap().read_only_hint,
            Some(true)
        );
    }

    #[test]
    fn test_mcp_tool_roundtrip() {
        let original = McpTool {
            name: "test_tool".to_string(),
            title: Some("Test".to_string()),
            description: Some("A test tool".to_string()),
            input_schema: McpInputSchema::object(
                json!({"param": {"type": "string"}}),
                vec!["param".to_string()],
            ),
            output_schema: None,
            annotations: Some(McpToolAnnotations {
                destructive_hint: Some(true),
                ..Default::default()
            }),
        };

        let internal = original.clone().to_internal();
        let roundtrip = McpTool::from_internal(internal);

        assert_eq!(roundtrip.name, original.name);
        assert_eq!(roundtrip.title, original.title);
        assert_eq!(roundtrip.description, original.description);
        assert_eq!(
            roundtrip.annotations.as_ref().map(|a| a.destructive_hint),
            original.annotations.as_ref().map(|a| a.destructive_hint)
        );
    }

    #[test]
    fn test_mcp_tool_result_conversion() {
        let mcp_result = McpToolResult::text("Result content");
        let internal = ("call_123".to_string(), mcp_result).to_internal();

        assert_eq!(internal.tool_call_id, "call_123");
        assert_eq!(internal.content, "Result content");
        assert!(!internal.is_error);

        let roundtrip = McpToolResult::from_internal(internal);
        assert!(!roundtrip.is_error);
        assert!(matches!(&roundtrip.content[0], McpContent::Text { text } if text == "Result content"));
    }
}

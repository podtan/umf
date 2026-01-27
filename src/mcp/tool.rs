//! MCP tool types matching the Model Context Protocol specification.
//!
//! This module provides type definitions for tools as defined in the MCP
//! specification, including support for tool annotations and content types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP Tool definition matching the MCP specification.
///
/// See: https://spec.modelcontextprotocol.io/specification/server/tools/
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    /// The name of the tool (unique identifier).
    pub name: String,

    /// Human-readable title for the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description of what the tool does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema describing the tool's input parameters.
    pub input_schema: McpInputSchema,

    /// Optional JSON Schema for the tool's output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,

    /// Optional annotations with hints about tool behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<McpToolAnnotations>,
}

impl McpTool {
    /// Create a new MCP tool with required fields.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: McpInputSchema,
    ) -> Self {
        Self {
            name: name.into(),
            title: None,
            description: Some(description.into()),
            input_schema,
            output_schema: None,
            annotations: None,
        }
    }

    /// Create a tool from raw JSON schema value.
    ///
    /// This is useful when parsing tool definitions from servers
    /// where the input schema is provided as a JSON Value.
    pub fn from_schema(
        name: impl Into<String>,
        description: impl Into<String>,
        schema: Value,
    ) -> Self {
        // Extract properties and required from the schema
        let properties = schema.get("properties").cloned();
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let input_schema = McpInputSchema {
            schema_type: schema
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("object")
                .to_string(),
            properties,
            required,
            additional: None,
        };

        Self::new(name, description, input_schema)
    }

    /// Set the human-readable title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set read-only hint annotation.
    pub fn with_read_only_hint(mut self, read_only: bool) -> Self {
        self.annotations
            .get_or_insert_with(Default::default)
            .read_only_hint = Some(read_only);
        self
    }

    /// Set destructive hint annotation.
    pub fn with_destructive_hint(mut self, destructive: bool) -> Self {
        self.annotations
            .get_or_insert_with(Default::default)
            .destructive_hint = Some(destructive);
        self
    }

    /// Set idempotent hint annotation.
    pub fn with_idempotent_hint(mut self, idempotent: bool) -> Self {
        self.annotations
            .get_or_insert_with(Default::default)
            .idempotent_hint = Some(idempotent);
        self
    }

    /// Set open-world hint annotation.
    pub fn with_open_world_hint(mut self, open_world: bool) -> Self {
        self.annotations
            .get_or_insert_with(Default::default)
            .open_world_hint = Some(open_world);
        self
    }
}

/// JSON Schema for MCP tool input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpInputSchema {
    /// The schema type (typically "object").
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Properties defining the tool's parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Value>,

    /// List of required property names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    /// Additional schema properties (additionalProperties, items, etc.)
    #[serde(flatten)]
    pub additional: Option<Value>,
}

impl McpInputSchema {
    /// Create a new input schema for an object type.
    pub fn object(properties: Value, required: Vec<String>) -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required,
            additional: None,
        }
    }

    /// Create an empty input schema (no parameters).
    pub fn empty() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: None,
            required: vec![],
            additional: None,
        }
    }
}

/// Annotations providing hints about tool behavior.
///
/// These annotations help LLMs and clients understand how to use the tool
/// safely and appropriately.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolAnnotations {
    /// Human-readable title for display purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// If true, the tool does not modify any state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    /// If true, the tool may perform destructive operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,

    /// If true, calling the tool multiple times with the same
    /// arguments has the same effect as calling it once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,

    /// If true, the tool interacts with external entities
    /// beyond the local environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// MCP Tool Call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    /// The name of the tool to call.
    pub name: String,

    /// Arguments to pass to the tool.
    #[serde(default)]
    pub arguments: Value,
}

impl McpToolCall {
    /// Create a new tool call.
    pub fn new(name: impl Into<String>, arguments: Value) -> Self {
        Self {
            name: name.into(),
            arguments,
        }
    }
}

/// MCP Tool Result response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolResult {
    /// The content of the result.
    pub content: Vec<McpContent>,

    /// Whether this result represents an error.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl McpToolResult {
    /// Create a successful text result.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent::Text {
                text: content.into(),
            }],
            is_error: false,
        }
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent::Text {
                text: message.into(),
            }],
            is_error: true,
        }
    }
}

/// MCP Content types for tool results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpContent {
    /// Text content.
    Text { text: String },

    /// Image content (base64 encoded).
    Image { data: String, mime_type: String },

    /// Resource reference.
    Resource {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_tool_new() {
        let input_schema = McpInputSchema::object(
            json!({
                "query": {"type": "string"}
            }),
            vec!["query".to_string()],
        );

        let tool = McpTool::new("search", "Search for items", input_schema);
        assert_eq!(tool.name, "search");
        assert_eq!(tool.description, Some("Search for items".to_string()));
        assert!(tool.title.is_none());
        assert!(tool.annotations.is_none());
    }

    #[test]
    fn test_mcp_tool_from_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"},
                "units": {"type": "string", "enum": ["celsius", "fahrenheit"]}
            },
            "required": ["location"]
        });

        let tool = McpTool::from_schema("get_weather", "Get current weather", schema);
        assert_eq!(tool.name, "get_weather");
        assert_eq!(
            tool.description,
            Some("Get current weather".to_string())
        );
        assert_eq!(tool.input_schema.schema_type, "object");
        assert_eq!(tool.input_schema.required, vec!["location"]);
        assert!(tool.input_schema.properties.is_some());
    }

    #[test]
    fn test_mcp_tool_builder_methods() {
        let input_schema = McpInputSchema::empty();

        let tool = McpTool::new("list_files", "List files in directory", input_schema)
            .with_title("List Files")
            .with_read_only_hint(true)
            .with_idempotent_hint(true);

        assert_eq!(tool.title, Some("List Files".to_string()));
        let annotations = tool.annotations.unwrap();
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert!(annotations.destructive_hint.is_none());
    }

    #[test]
    fn test_mcp_tool_destructive_hint() {
        let input_schema = McpInputSchema::empty();

        let tool = McpTool::new("delete_file", "Delete a file", input_schema)
            .with_destructive_hint(true)
            .with_open_world_hint(false);

        let annotations = tool.annotations.unwrap();
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "get_weather".to_string(),
            title: Some("Get Weather".to_string()),
            description: Some("Get current weather for a location".to_string()),
            input_schema: McpInputSchema::object(
                json!({
                    "location": {
                        "type": "string",
                        "description": "City name"
                    }
                }),
                vec!["location".to_string()],
            ),
            output_schema: None,
            annotations: Some(McpToolAnnotations {
                read_only_hint: Some(true),
                ..Default::default()
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("inputSchema"));
        assert!(json.contains("readOnlyHint"));

        let parsed: McpTool = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "get_weather");
    }

    #[test]
    fn test_mcp_tool_result() {
        let result = McpToolResult::text("72°F, sunny");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);

        let error = McpToolResult::error("Location not found");
        assert!(error.is_error);
    }

    #[test]
    fn test_mcp_content_types() {
        let text = McpContent::Text {
            text: "Hello".to_string(),
        };
        let json = serde_json::to_string(&text).unwrap();
        assert!(json.contains("\"type\":\"text\""));

        let image = McpContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };
        let json = serde_json::to_string(&image).unwrap();
        assert!(json.contains("\"type\":\"image\""));
    }
}

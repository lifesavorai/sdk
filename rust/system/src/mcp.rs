//! MCP tool definition helpers for system component tool registration.
//!
//! System components expose their operations as MCP tools. This module
//! provides builders and types for declaring tool schemas that the agent
//! registers with connected MCP clients.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An MCP tool definition that describes an operation exposed by a system component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The tool name (e.g., `system.memory_store.sqlite-vec.search`).
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema describing the tool's input parameters.
    pub input_schema: Value,
}

/// Type alias for backwards compatibility with system components.
pub type McpToolDefinition = ToolDefinition;

/// Builder for constructing MCP tool definitions.
pub struct ToolDefinitionBuilder {
    name: String,
    description: String,
    properties: serde_json::Map<String, Value>,
    required: Vec<String>,
}

impl ToolDefinitionBuilder {
    /// Create a new tool definition builder.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            properties: serde_json::Map::new(),
            required: Vec::new(),
        }
    }

    /// Add a string parameter.
    pub fn string_param(mut self, name: &str, description: &str, required: bool) -> Self {
        self.properties.insert(
            name.to_string(),
            serde_json::json!({
                "type": "string",
                "description": description,
            }),
        );
        if required {
            self.required.push(name.to_string());
        }
        self
    }

    /// Add an integer parameter.
    pub fn integer_param(mut self, name: &str, description: &str, required: bool) -> Self {
        self.properties.insert(
            name.to_string(),
            serde_json::json!({
                "type": "integer",
                "description": description,
            }),
        );
        if required {
            self.required.push(name.to_string());
        }
        self
    }

    /// Add a boolean parameter.
    pub fn boolean_param(mut self, name: &str, description: &str, required: bool) -> Self {
        self.properties.insert(
            name.to_string(),
            serde_json::json!({
                "type": "boolean",
                "description": description,
            }),
        );
        if required {
            self.required.push(name.to_string());
        }
        self
    }

    /// Add a parameter with a custom JSON Schema.
    pub fn param(mut self, name: &str, schema: Value, required: bool) -> Self {
        self.properties.insert(name.to_string(), schema);
        if required {
            self.required.push(name.to_string());
        }
        self
    }

    /// Build the tool definition.
    pub fn build(self) -> ToolDefinition {
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": self.properties,
            "required": self.required,
        });

        ToolDefinition {
            name: self.name,
            description: self.description,
            input_schema,
        }
    }
}

/// Generate the fully-qualified MCP tool name for a system component operation.
///
/// Format: `system.{component_type}.{instance_name}.{operation}`
pub fn tool_name(component_type: &str, instance_name: &str, operation: &str) -> String {
    format!("system.{}.{}.{}", component_type, instance_name, operation)
}

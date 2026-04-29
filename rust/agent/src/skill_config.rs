//! Configuration schema, setup workflow, and documentation types for skills.
//!
//! These types are the canonical definitions used by the agent runtime,
//! Rust SDK, and (via JSON serialization) the JS SDK. They live here in
//! `lifesavor-agent-types` so every consumer shares a single source of truth.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// JSON Schema-based configuration schema for a skill.
/// Embedded in skill.json under the `config_schema` field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigSchema {
    /// JSON Schema draft identifier
    #[serde(rename = "$schema", default = "default_schema_draft")]
    pub schema_draft: String,
    /// Must be "object"
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Map of field name → field definition
    pub properties: BTreeMap<String, ConfigFieldDefinition>,
    /// List of required field names
    #[serde(default)]
    pub required: Vec<String>,
}

fn default_schema_draft() -> String {
    "https://json-schema.org/draft/2020-12/schema".to_string()
}

/// Definition of a single config field within the schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigFieldDefinition {
    /// JSON Schema type: "string", "number", "boolean", "integer", or "array"
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for UI display
    pub title: String,
    /// Help text describing the field's purpose
    pub description: String,
    /// Default value (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Whether this field holds a secret value (API key, token, etc.)
    #[serde(rename = "x-secret", default, skip_serializing_if = "is_false")]
    pub secret: bool,
    /// For array fields: schema of array items
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ConfigFieldDefinition>>,
    /// Enum of allowed values (optional)
    #[serde(rename = "enum", default, skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

/// Helper for `skip_serializing_if` on boolean fields — skips when `false`.
fn is_false(v: &bool) -> bool {
    !v
}

/// A single step in a multi-step setup workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetupStep {
    /// Unique identifier within the skill (e.g., "credentials", "preferences")
    pub step_id: String,
    /// Display title (3-100 characters)
    pub title: String,
    /// Description text (10-500 characters)
    pub description: String,
    /// Config field names from config_schema that belong to this step
    pub fields: Vec<String>,
    /// Optional skill operation name to invoke for step validation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_command: Option<String>,
}

/// Setup status tracking for a skill's configuration workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

impl Default for SetupStatus {
    fn default() -> Self {
        Self::Completed
    }
}

/// Documentation references bundled with a skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillDocumentation {
    /// Relative path to the main usage guide Markdown file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_guide: Option<String>,
    /// Additional example documentation files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<DocumentationExample>,
}

/// A single documentation example reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentationExample {
    /// Display title for the example
    pub title: String,
    /// Relative path to the Markdown file
    pub file: String,
}

/// Request payload sent to a skill's validation_command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationRequest {
    /// The step being validated
    pub step_id: String,
    /// Field values submitted by the user
    pub values: serde_json::Value,
    /// Metadata about the validation context
    #[serde(default)]
    pub context: ValidationContext,
}

/// Metadata about the validation invocation context.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ValidationContext {
    /// The skill ID being configured
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    /// Whether this is initial setup or reconfiguration
    #[serde(default)]
    pub is_reconfigure: bool,
}

/// Response payload returned by a skill's validation_command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResponse {
    /// "success" or "failure"
    pub status: String,
    /// Human-readable message (error details on failure)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional transformed values or additional data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Computed setup complexity based on step count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupComplexity {
    None,
    Simple,
    Moderate,
    Advanced,
}

impl SetupComplexity {
    /// Compute complexity from step count.
    /// 0 → None, 1 → Simple, 2-3 → Moderate, 4+ → Advanced
    pub fn from_step_count(count: usize) -> Self {
        match count {
            0 => Self::None,
            1 => Self::Simple,
            2..=3 => Self::Moderate,
            _ => Self::Advanced,
        }
    }
}

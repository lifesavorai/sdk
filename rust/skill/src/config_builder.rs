//! Fluent builders for constructing [`ConfigSchema`] and [`SetupStep`] objects.
//!
//! These builders provide compile-time safety and ergonomic APIs for skill
//! developers defining configuration schemas and multi-step setup workflows.
//!
//! # Example
//!
//! ```rust,ignore
//! use lifesavor_skill_sdk::config_builder::{ConfigSchemaBuilder, SetupWorkflowBuilder};
//!
//! let schema = ConfigSchemaBuilder::new()
//!     .add_string_field("api_key", "API Key", "Your service API key")
//!         .required()
//!         .secret()
//!         .done()
//!     .add_boolean_field("enabled", "Enabled", "Whether the integration is active")
//!         .default_value(serde_json::json!(true))
//!         .done()
//!     .build()?;
//!
//! let steps = SetupWorkflowBuilder::new(schema.clone())
//!     .add_step_with_validation("creds", "Credentials", "Enter your API key", &["api_key"], "validate_creds")
//!     .add_step("prefs", "Preferences", "Configure preferences", &["enabled"])
//!     .build()?;
//! ```

use std::collections::BTreeMap;

use lifesavor_agent_types::skill_config::{ConfigFieldDefinition, ConfigSchema, SetupStep};

use crate::error::SkillSdkError;

// ---------------------------------------------------------------------------
// ConfigSchemaBuilder
// ---------------------------------------------------------------------------

/// Builder for constructing [`ConfigSchema`] objects with a fluent API.
///
/// Each `add_*_field` method returns a [`FieldBuilder`] that lets you
/// configure the field (required, secret, default) before calling
/// [`FieldBuilder::done`] to return to this builder.
pub struct ConfigSchemaBuilder {
    properties: BTreeMap<String, ConfigFieldDefinition>,
    required: Vec<String>,
}

impl ConfigSchemaBuilder {
    /// Create a new, empty schema builder.
    pub fn new() -> Self {
        Self {
            properties: BTreeMap::new(),
            required: Vec::new(),
        }
    }

    /// Add a `string` field to the schema.
    pub fn add_string_field(self, name: &str, title: &str, desc: &str) -> FieldBuilder {
        FieldBuilder::new(self, name, "string", title, desc, None)
    }

    /// Add a `number` field to the schema.
    pub fn add_number_field(self, name: &str, title: &str, desc: &str) -> FieldBuilder {
        FieldBuilder::new(self, name, "number", title, desc, None)
    }

    /// Add a `boolean` field to the schema.
    pub fn add_boolean_field(self, name: &str, title: &str, desc: &str) -> FieldBuilder {
        FieldBuilder::new(self, name, "boolean", title, desc, None)
    }

    /// Add an `integer` field to the schema.
    pub fn add_integer_field(self, name: &str, title: &str, desc: &str) -> FieldBuilder {
        FieldBuilder::new(self, name, "integer", title, desc, None)
    }

    /// Add an `array` field to the schema.
    ///
    /// `item_type` specifies the JSON Schema type of array elements
    /// (e.g. `"string"`, `"number"`).
    pub fn add_array_field(
        self,
        name: &str,
        title: &str,
        desc: &str,
        item_type: &str,
    ) -> FieldBuilder {
        FieldBuilder::new(self, name, "array", title, desc, Some(item_type))
    }

    /// Consume the builder and produce a validated [`ConfigSchema`].
    ///
    /// Returns an error if the schema has no properties.
    pub fn build(self) -> Result<ConfigSchema, SkillSdkError> {
        if self.properties.is_empty() {
            return Err(SkillSdkError::ConfigBuilder(
                "ConfigSchema must have at least one property".to_string(),
            ));
        }

        Ok(ConfigSchema {
            schema_draft: "https://json-schema.org/draft/2020-12/schema".to_string(),
            schema_type: "object".to_string(),
            properties: self.properties,
            required: self.required,
        })
    }
}

impl Default for ConfigSchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FieldBuilder
// ---------------------------------------------------------------------------

/// Intermediate builder for configuring a single config field.
///
/// Created by the `add_*_field` methods on [`ConfigSchemaBuilder`].
/// Call [`done`](FieldBuilder::done) to finalize the field and return to the
/// parent schema builder.
pub struct FieldBuilder {
    parent: ConfigSchemaBuilder,
    name: String,
    field_type: String,
    title: String,
    description: String,
    default: Option<serde_json::Value>,
    secret: bool,
    is_required: bool,
    items: Option<Box<ConfigFieldDefinition>>,
}

impl FieldBuilder {
    fn new(
        parent: ConfigSchemaBuilder,
        name: &str,
        field_type: &str,
        title: &str,
        desc: &str,
        item_type: Option<&str>,
    ) -> Self {
        let items = item_type.map(|t| {
            Box::new(ConfigFieldDefinition {
                field_type: t.to_string(),
                title: String::new(),
                description: String::new(),
                default: None,
                secret: false,
                items: None,
                allowed_values: None,
            })
        });

        Self {
            parent,
            name: name.to_string(),
            field_type: field_type.to_string(),
            title: title.to_string(),
            description: desc.to_string(),
            default: None,
            secret: false,
            is_required: false,
            items,
        }
    }

    /// Mark this field as required in the schema.
    pub fn required(mut self) -> Self {
        self.is_required = true;
        self
    }

    /// Mark this field as a secret (sets `x-secret: true`).
    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    /// Set a default value for this field.
    pub fn default_value(mut self, val: serde_json::Value) -> Self {
        self.default = Some(val);
        self
    }

    /// Finalize this field and return to the parent [`ConfigSchemaBuilder`].
    pub fn done(mut self) -> ConfigSchemaBuilder {
        let definition = ConfigFieldDefinition {
            field_type: self.field_type,
            title: self.title,
            description: self.description,
            default: self.default,
            secret: self.secret,
            items: self.items,
            allowed_values: None,
        };

        if self.is_required {
            self.parent.required.push(self.name.clone());
        }

        self.parent.properties.insert(self.name, definition);
        self.parent
    }
}

// ---------------------------------------------------------------------------
// SetupWorkflowBuilder
// ---------------------------------------------------------------------------

/// Builder for constructing an ordered list of [`SetupStep`] objects.
///
/// The [`build`](SetupWorkflowBuilder::build) method validates that every
/// field reference exists in the associated [`ConfigSchema`] and that no
/// field appears in more than one step.
pub struct SetupWorkflowBuilder {
    steps: Vec<SetupStep>,
    schema: ConfigSchema,
}

impl SetupWorkflowBuilder {
    /// Create a new workflow builder backed by the given schema.
    pub fn new(schema: ConfigSchema) -> Self {
        Self {
            steps: Vec::new(),
            schema,
        }
    }

    /// Add a setup step without a validation command.
    pub fn add_step(
        mut self,
        step_id: &str,
        title: &str,
        desc: &str,
        fields: &[&str],
    ) -> Self {
        self.steps.push(SetupStep {
            step_id: step_id.to_string(),
            title: title.to_string(),
            description: desc.to_string(),
            fields: fields.iter().map(|s| s.to_string()).collect(),
            validation_command: None,
        });
        self
    }

    /// Add a setup step with a validation command.
    pub fn add_step_with_validation(
        mut self,
        step_id: &str,
        title: &str,
        desc: &str,
        fields: &[&str],
        cmd: &str,
    ) -> Self {
        self.steps.push(SetupStep {
            step_id: step_id.to_string(),
            title: title.to_string(),
            description: desc.to_string(),
            fields: fields.iter().map(|s| s.to_string()).collect(),
            validation_command: Some(cmd.to_string()),
        });
        self
    }

    /// Consume the builder and return the validated list of steps.
    ///
    /// # Errors
    ///
    /// Returns [`SkillSdkError::ConfigBuilder`] if:
    /// - Any field reference does not exist in the schema
    /// - A field appears in more than one step
    /// - Duplicate `step_id` values are found
    pub fn build(self) -> Result<Vec<SetupStep>, SkillSdkError> {
        let mut seen_fields: BTreeMap<String, String> = BTreeMap::new();
        let mut seen_step_ids: Vec<String> = Vec::new();

        for step in &self.steps {
            // Check for duplicate step IDs.
            if seen_step_ids.contains(&step.step_id) {
                return Err(SkillSdkError::ConfigBuilder(format!(
                    "Duplicate step_id: '{}'",
                    step.step_id
                )));
            }
            seen_step_ids.push(step.step_id.clone());

            for field in &step.fields {
                // Check field exists in schema.
                if !self.schema.properties.contains_key(field) {
                    return Err(SkillSdkError::ConfigBuilder(format!(
                        "Field '{}' in step '{}' does not exist in the config schema",
                        field, step.step_id
                    )));
                }

                // Check field is not duplicated across steps.
                if let Some(prev_step) = seen_fields.get(field) {
                    return Err(SkillSdkError::ConfigBuilder(format!(
                        "Field '{}' appears in both step '{}' and step '{}'",
                        field, prev_step, step.step_id
                    )));
                }
                seen_fields.insert(field.clone(), step.step_id.clone());
            }
        }

        Ok(self.steps)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_schema_with_all_field_types() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("name", "Name", "Your name")
                .required()
                .done()
            .add_number_field("score", "Score", "A numeric score")
                .default_value(serde_json::json!(0.0))
                .done()
            .add_boolean_field("active", "Active", "Is active")
                .default_value(serde_json::json!(true))
                .done()
            .add_integer_field("count", "Count", "Item count")
                .done()
            .add_array_field("tags", "Tags", "Tag list", "string")
                .done()
            .build()
            .expect("should build successfully");

        assert_eq!(schema.properties.len(), 5);
        assert_eq!(schema.required, vec!["name".to_string()]);
        assert_eq!(schema.schema_type, "object");
        assert!(schema.schema_draft.contains("2020-12"));

        // Verify field types
        assert_eq!(schema.properties["name"].field_type, "string");
        assert_eq!(schema.properties["score"].field_type, "number");
        assert_eq!(schema.properties["active"].field_type, "boolean");
        assert_eq!(schema.properties["count"].field_type, "integer");
        assert_eq!(schema.properties["tags"].field_type, "array");
        assert!(schema.properties["tags"].items.is_some());
    }

    #[test]
    fn builder_secret_field() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("api_key", "API Key", "Secret key")
                .required()
                .secret()
                .done()
            .build()
            .expect("should build");

        assert!(schema.properties["api_key"].secret);
    }

    #[test]
    fn builder_rejects_empty_schema() {
        let result = ConfigSchemaBuilder::new().build();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least one property"));
    }

    #[test]
    fn workflow_builder_valid_steps() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("api_key", "API Key", "Key")
                .required()
                .done()
            .add_string_field("location", "Location", "City")
                .done()
            .build()
            .unwrap();

        let steps = SetupWorkflowBuilder::new(schema)
            .add_step_with_validation(
                "creds",
                "Credentials",
                "Enter your key",
                &["api_key"],
                "validate_key",
            )
            .add_step("prefs", "Preferences", "Set preferences", &["location"])
            .build()
            .expect("should build");

        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].step_id, "creds");
        assert_eq!(steps[0].validation_command, Some("validate_key".to_string()));
        assert_eq!(steps[1].step_id, "prefs");
        assert!(steps[1].validation_command.is_none());
    }

    #[test]
    fn workflow_builder_rejects_invalid_field_ref() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("api_key", "API Key", "Key")
                .done()
            .build()
            .unwrap();

        let result = SetupWorkflowBuilder::new(schema)
            .add_step("s1", "Step", "Description text", &["nonexistent"])
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent"));
        assert!(err.contains("does not exist"));
    }

    #[test]
    fn workflow_builder_rejects_duplicate_field_across_steps() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("api_key", "API Key", "Key")
                .done()
            .add_string_field("location", "Location", "City")
                .done()
            .build()
            .unwrap();

        let result = SetupWorkflowBuilder::new(schema)
            .add_step("s1", "Step 1", "First step here", &["api_key"])
            .add_step("s2", "Step 2", "Second step here", &["api_key"])
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("api_key"));
        assert!(err.contains("appears in both"));
    }

    #[test]
    fn workflow_builder_rejects_duplicate_step_id() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("a", "A", "Field A")
                .done()
            .add_string_field("b", "B", "Field B")
                .done()
            .build()
            .unwrap();

        let result = SetupWorkflowBuilder::new(schema)
            .add_step("same", "Step 1", "First step here", &["a"])
            .add_step("same", "Step 2", "Second step here", &["b"])
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Duplicate step_id"));
    }

    #[test]
    fn schema_round_trip_through_json() {
        let schema = ConfigSchemaBuilder::new()
            .add_string_field("key", "Key", "A key")
                .required()
                .secret()
                .done()
            .add_integer_field("count", "Count", "A count")
                .default_value(serde_json::json!(42))
                .done()
            .add_array_field("items", "Items", "Item list", "string")
                .done()
            .build()
            .unwrap();

        let json = serde_json::to_string(&schema).unwrap();
        let deserialized: ConfigSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(schema, deserialized);
    }
}

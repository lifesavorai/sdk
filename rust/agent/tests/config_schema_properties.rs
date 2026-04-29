/**
 * Property-based tests for ConfigSchema serialization round-trip.
 *
 * Property 1: ConfigSchema serialization round-trip — any valid ConfigSchema
 * with all field types, required fields, x-secret annotations, defaults, and
 * enums serializes to JSON and deserializes back to an equivalent object.
 *
 * **Validates: Requirements 1.9, 6.7**
 */

use lifesavor_agent_types::skill_config::{ConfigFieldDefinition, ConfigSchema};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Arbitrary generators
// ---------------------------------------------------------------------------

/// Generate a valid JSON Schema field type string.
fn arb_field_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("string".to_string()),
        Just("number".to_string()),
        Just("boolean".to_string()),
        Just("integer".to_string()),
        Just("array".to_string()),
    ]
}

/// Generate a non-empty title string (1-80 chars, alphanumeric + spaces).
fn arb_title() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,79}"
}

/// Generate a non-empty description string (1-120 chars).
fn arb_description() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 .,]{0,119}"
}

/// Generate an optional default value appropriate for any field type.
fn arb_default_value() -> impl Strategy<Value = Option<serde_json::Value>> {
    prop_oneof![
        Just(None),
        Just(Some(serde_json::Value::String("default_val".to_string()))),
        Just(Some(serde_json::Value::Bool(true))),
        Just(Some(serde_json::Value::Bool(false))),
        Just(Some(serde_json::json!(42))),
        Just(Some(serde_json::json!(3.14))),
        Just(Some(serde_json::json!(["a", "b"]))),
    ]
}

/// Generate an optional enum of allowed values.
fn arb_allowed_values() -> impl Strategy<Value = Option<Vec<serde_json::Value>>> {
    prop_oneof![
        Just(None),
        Just(Some(vec![
            serde_json::Value::String("option_a".to_string()),
            serde_json::Value::String("option_b".to_string()),
            serde_json::Value::String("option_c".to_string()),
        ])),
        Just(Some(vec![
            serde_json::json!(1),
            serde_json::json!(2),
            serde_json::json!(3),
        ])),
    ]
}

/// Generate a leaf ConfigFieldDefinition (no nested items).
fn arb_leaf_field_definition() -> impl Strategy<Value = ConfigFieldDefinition> {
    (
        arb_field_type(),
        arb_title(),
        arb_description(),
        arb_default_value(),
        any::<bool>(),
        arb_allowed_values(),
    )
        .prop_map(
            |(field_type, title, description, default, secret, allowed_values)| {
                ConfigFieldDefinition {
                    field_type,
                    title,
                    description,
                    default,
                    secret,
                    items: None,
                    allowed_values,
                }
            },
        )
}

/// Generate a ConfigFieldDefinition for an array field with nested items.
fn arb_array_field_definition() -> impl Strategy<Value = ConfigFieldDefinition> {
    (
        arb_title(),
        arb_description(),
        arb_default_value(),
        any::<bool>(),
        arb_allowed_values(),
        arb_leaf_field_definition(),
    )
        .prop_map(
            |(title, description, default, secret, allowed_values, item_def)| {
                ConfigFieldDefinition {
                    field_type: "array".to_string(),
                    title,
                    description,
                    default,
                    secret,
                    items: Some(Box::new(item_def)),
                    allowed_values,
                }
            },
        )
}

/// Generate a ConfigFieldDefinition covering all variants including arrays with items.
fn arb_field_definition() -> impl Strategy<Value = ConfigFieldDefinition> {
    prop_oneof![
        8 => arb_leaf_field_definition(),
        2 => arb_array_field_definition(),
    ]
}

/// Generate a valid field name (lowercase alphanumeric with underscores).
fn arb_field_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,19}"
}

/// Generate a ConfigSchema with 1-6 properties, a subset marked as required,
/// and covering all field types, x-secret annotations, defaults, and enums.
fn arb_config_schema() -> impl Strategy<Value = ConfigSchema> {
    prop::collection::btree_map(arb_field_name(), arb_field_definition(), 1..=6).prop_flat_map(
        |properties| {
            let keys: Vec<String> = properties.keys().cloned().collect();
            let len = keys.len();
            // Generate a boolean mask to decide which fields are required
            prop::collection::vec(any::<bool>(), len).prop_map(move |mask| {
                let required: Vec<String> = keys
                    .iter()
                    .zip(mask.iter())
                    .filter(|(_, &is_req)| is_req)
                    .map(|(k, _)| k.clone())
                    .collect();
                ConfigSchema {
                    schema_draft: "https://json-schema.org/draft/2020-12/schema".to_string(),
                    schema_type: "object".to_string(),
                    properties: properties.clone(),
                    required,
                }
            })
        },
    )
}

// ---------------------------------------------------------------------------
// Property 1: ConfigSchema serialization round-trip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Serializing a ConfigSchema to JSON and deserializing back produces an
    /// equivalent ConfigSchema object.
    #[test]
    fn prop_config_schema_json_round_trip(schema in arb_config_schema()) {
        let json = serde_json::to_string(&schema)
            .expect("ConfigSchema should serialize to JSON");
        let deserialized: ConfigSchema = serde_json::from_str(&json)
            .expect("ConfigSchema should deserialize from JSON");

        // Structural equality
        prop_assert_eq!(&schema.schema_draft, &deserialized.schema_draft,
            "schema_draft must survive round-trip");
        prop_assert_eq!(&schema.schema_type, &deserialized.schema_type,
            "schema_type must survive round-trip");
        prop_assert_eq!(schema.properties.len(), deserialized.properties.len(),
            "properties count must survive round-trip");
        prop_assert_eq!(&schema.required, &deserialized.required,
            "required fields must survive round-trip");

        // Full equality
        prop_assert_eq!(&schema, &deserialized,
            "Round-tripped ConfigSchema must be equivalent to original");
    }

    /// Each ConfigFieldDefinition within a round-tripped ConfigSchema preserves
    /// all annotations: type, title, description, default, x-secret, items, and enum.
    #[test]
    fn prop_config_field_annotations_preserved(schema in arb_config_schema()) {
        let json = serde_json::to_string(&schema)
            .expect("ConfigSchema should serialize to JSON");
        let deserialized: ConfigSchema = serde_json::from_str(&json)
            .expect("ConfigSchema should deserialize from JSON");

        for (name, original_field) in &schema.properties {
            let rt_field = deserialized.properties.get(name)
                .expect(&format!("Field '{}' must exist after round-trip", name));

            prop_assert_eq!(&original_field.field_type, &rt_field.field_type,
                "field_type for '{}' must survive round-trip", name);
            prop_assert_eq!(&original_field.title, &rt_field.title,
                "title for '{}' must survive round-trip", name);
            prop_assert_eq!(&original_field.description, &rt_field.description,
                "description for '{}' must survive round-trip", name);
            prop_assert_eq!(&original_field.default, &rt_field.default,
                "default for '{}' must survive round-trip", name);
            prop_assert_eq!(original_field.secret, rt_field.secret,
                "x-secret for '{}' must survive round-trip", name);
            prop_assert_eq!(&original_field.items, &rt_field.items,
                "items for '{}' must survive round-trip", name);
            prop_assert_eq!(&original_field.allowed_values, &rt_field.allowed_values,
                "enum for '{}' must survive round-trip", name);
        }
    }
}

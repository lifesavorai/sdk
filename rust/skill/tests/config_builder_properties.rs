//! Property-based tests for `SetupWorkflowBuilder` field reference validation.
//!
//! **Property 7: SetupWorkflowBuilder field reference validation**
//!
//! *For any* `SetupWorkflowBuilder` constructed with a `ConfigSchema` and steps
//! referencing field names not present in that schema, the `build()` method
//! SHALL return an error identifying the invalid field reference.
//!
//! **Validates: Requirements 6.6**

use lifesavor_skill_sdk::config_builder::{ConfigSchemaBuilder, SetupWorkflowBuilder};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid field name (lowercase alphanumeric with underscores, 2-20 chars).
fn arb_field_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,19}"
}

/// Generate a set of 1-4 distinct valid field names for the schema.
fn arb_schema_field_names() -> impl Strategy<Value = Vec<String>> {
    prop::collection::hash_set(arb_field_name(), 1..=4)
        .prop_map(|s| s.into_iter().collect::<Vec<_>>())
}

/// Generate a field name guaranteed NOT to be in the provided set.
/// Uses an "invalid_" prefix plus a random suffix to avoid collisions.
fn arb_invalid_field_name() -> impl Strategy<Value = String> {
    "[a-z]{1,10}".prop_map(|s| format!("invalid_{}", s))
}

/// Generate a valid step_id string.
fn arb_step_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,14}"
}

/// Generate a valid step title (3-100 chars).
fn arb_step_title() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{2,40}"
}

/// Generate a valid step description (10-500 chars).
fn arb_step_description() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 .,]{9,60}"
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `ConfigSchema` from a list of field names using `ConfigSchemaBuilder`.
fn build_schema_from_names(names: &[String]) -> lifesavor_agent_types::skill_config::ConfigSchema {
    let mut builder = ConfigSchemaBuilder::new();
    for name in names {
        builder = builder
            .add_string_field(name, &format!("Title for {}", name), &format!("Description for {}", name))
            .done();
    }
    builder.build().expect("schema with valid fields should build")
}

// ---------------------------------------------------------------------------
// Property 7: SetupWorkflowBuilder field reference validation
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 7: SetupWorkflowBuilder rejects invalid field references**
    ///
    /// **Validates: Requirements 6.6**
    ///
    /// For any schema with known field names and a step referencing a field
    /// name NOT in the schema, `build()` SHALL return an `Err` whose message
    /// contains the invalid field name.
    #[test]
    fn prop_builder_rejects_invalid_field_reference(
        schema_fields in arb_schema_field_names(),
        invalid_field in arb_invalid_field_name(),
        step_id in arb_step_id(),
        title in arb_step_title(),
        description in arb_step_description(),
    ) {
        // Ensure the invalid field is truly not in the schema
        prop_assume!(!schema_fields.contains(&invalid_field));

        let schema = build_schema_from_names(&schema_fields);

        let result = SetupWorkflowBuilder::new(schema)
            .add_step(&step_id, &title, &description, &[invalid_field.as_str()])
            .build();

        prop_assert!(result.is_err(),
            "build() should fail when step references field '{}' not in schema {:?}",
            invalid_field, schema_fields);

        let err_msg = result.unwrap_err().to_string();
        prop_assert!(err_msg.contains(&invalid_field),
            "Error message should contain the invalid field name '{}', got: {}",
            invalid_field, err_msg);
    }

    /// **Property 7 (variant): Mixed valid and invalid field references**
    ///
    /// **Validates: Requirements 6.6**
    ///
    /// For any schema and a step that references both valid and invalid field
    /// names, `build()` SHALL return an `Err` identifying at least one invalid
    /// field reference.
    #[test]
    fn prop_builder_rejects_mixed_valid_and_invalid_refs(
        schema_fields in arb_schema_field_names(),
        invalid_field in arb_invalid_field_name(),
        step_id in arb_step_id(),
        title in arb_step_title(),
        description in arb_step_description(),
    ) {
        prop_assume!(!schema_fields.contains(&invalid_field));
        prop_assume!(!schema_fields.is_empty());

        let schema = build_schema_from_names(&schema_fields);

        // Build a fields list with one valid field and one invalid field
        let valid_field = &schema_fields[0];
        let fields: Vec<&str> = vec![valid_field.as_str(), invalid_field.as_str()];

        let result = SetupWorkflowBuilder::new(schema)
            .add_step(&step_id, &title, &description, &fields)
            .build();

        prop_assert!(result.is_err(),
            "build() should fail when step mixes valid field '{}' with invalid field '{}'",
            valid_field, invalid_field);

        let err_msg = result.unwrap_err().to_string();
        prop_assert!(err_msg.contains(&invalid_field),
            "Error message should identify the invalid field '{}', got: {}",
            invalid_field, err_msg);
    }

    /// **Property 7 (variant): Multiple steps with invalid references**
    ///
    /// **Validates: Requirements 6.6**
    ///
    /// For any schema and multiple steps where at least one references an
    /// invalid field, `build()` SHALL return an `Err`.
    #[test]
    fn prop_builder_rejects_invalid_ref_across_multiple_steps(
        schema_fields in prop::collection::hash_set(arb_field_name(), 2..=4)
            .prop_map(|s| s.into_iter().collect::<Vec<_>>()),
        invalid_field in arb_invalid_field_name(),
        title1 in arb_step_title(),
        desc1 in arb_step_description(),
        title2 in arb_step_title(),
        desc2 in arb_step_description(),
    ) {
        prop_assume!(!schema_fields.contains(&invalid_field));
        prop_assume!(schema_fields.len() >= 2);

        let schema = build_schema_from_names(&schema_fields);

        // First step uses a valid field, second step uses the invalid field
        let valid_field = &schema_fields[0];

        let result = SetupWorkflowBuilder::new(schema)
            .add_step("step_valid", &title1, &desc1, &[valid_field.as_str()])
            .add_step("step_bad", &title2, &desc2, &[invalid_field.as_str()])
            .build();

        prop_assert!(result.is_err(),
            "build() should fail when any step references invalid field '{}'",
            invalid_field);

        let err_msg = result.unwrap_err().to_string();
        prop_assert!(err_msg.contains(&invalid_field),
            "Error message should contain the invalid field name '{}', got: {}",
            invalid_field, err_msg);
    }
}

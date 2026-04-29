//! Configuration schema and setup workflow example.
//!
//! Demonstrates defining a `ConfigSchema` with multiple field types, secret
//! fields, and a multi-step `SetupWorkflow` using the SDK builders.
//!
//! Run with: `cargo run --example config_schema_skill`

use lifesavor_skill_sdk::config_builder::{ConfigSchemaBuilder, SetupWorkflowBuilder};
use lifesavor_skill_sdk::{
    ConfigSchema, DocumentationExample, SetupComplexity, SetupStep, SkillDocumentation,
    ValidationResponse,
};
use lifesavor_skill_sdk::validation_handler::{
    validation_error_connection_failed, validation_error_invalid_credentials,
    validation_error_timeout,
};

fn main() {
    println!("=== Configuration Schema & Setup Workflow Example ===\n");

    // -----------------------------------------------------------------------
    // 1. Build a configuration schema with all supported field types
    // -----------------------------------------------------------------------

    let schema: ConfigSchema = ConfigSchemaBuilder::new()
        // Secret string field (API key stored in Vault)
        .add_string_field("api_key", "API Key", "Your OpenWeatherMap API key")
            .required()
            .secret()
            .done()
        // Required string field
        .add_string_field("location", "Default Location", "City name or coordinates for weather alerts")
            .required()
            .done()
        // String field with a default value
        .add_string_field("units", "Temperature Units", "Preferred temperature unit system")
            .default_value(serde_json::json!("metric"))
            .done()
        // Array field with string items
        .add_array_field("alert_types", "Alert Types", "Types of weather alerts to receive", "string")
            .default_value(serde_json::json!(["severe", "warning"]))
            .done()
        // Integer field with a default
        .add_integer_field("polling_interval", "Polling Interval", "How often to check for new alerts (in minutes)")
            .default_value(serde_json::json!(15))
            .done()
        // Boolean field with a default
        .add_boolean_field("notifications_enabled", "Enable Notifications", "Whether to send push notifications for new alerts")
            .default_value(serde_json::json!(true))
            .done()
        // Number field (floating-point)
        .add_number_field("alert_threshold", "Alert Threshold", "Minimum severity score to trigger an alert")
            .default_value(serde_json::json!(0.7))
            .done()
        .build()
        .expect("schema should be valid");

    println!("Schema built with {} fields", schema.properties.len());
    println!("Required fields: {:?}", schema.required);
    println!(
        "Secret fields: {:?}",
        schema
            .properties
            .iter()
            .filter(|(_, def)| def.secret)
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
    );

    // -----------------------------------------------------------------------
    // 2. Build a multi-step setup workflow
    // -----------------------------------------------------------------------

    let steps: Vec<SetupStep> = SetupWorkflowBuilder::new(schema.clone())
        // Step 1: Credentials — with server-side validation
        .add_step_with_validation(
            "credentials",
            "API Credentials",
            "Enter your OpenWeatherMap API key to connect the weather service",
            &["api_key"],
            "validate_api_key",
        )
        // Step 2: Preferences — no validation needed
        .add_step(
            "preferences",
            "Alert Preferences",
            "Configure your location and alert preferences",
            &[
                "location",
                "units",
                "alert_types",
                "polling_interval",
                "notifications_enabled",
                "alert_threshold",
            ],
        )
        .build()
        .expect("workflow should be valid");

    println!("\nSetup workflow: {} steps", steps.len());
    for (i, step) in steps.iter().enumerate() {
        println!(
            "  Step {}: {} ({} fields, validation: {})",
            i + 1,
            step.title,
            step.fields.len(),
            step.validation_command
                .as_deref()
                .unwrap_or("none"),
        );
    }

    // -----------------------------------------------------------------------
    // 3. Compute setup complexity
    // -----------------------------------------------------------------------

    let complexity = SetupComplexity::from_step_count(steps.len());
    println!("\nSetup complexity: {:?}", complexity);

    // -----------------------------------------------------------------------
    // 4. Define documentation references
    // -----------------------------------------------------------------------

    let docs = SkillDocumentation {
        usage_guide: Some("docs/USAGE.md".to_string()),
        examples: vec![DocumentationExample {
            title: "Setting Up Multiple Locations".to_string(),
            file: "docs/examples/multi-location.md".to_string(),
        }],
    };

    println!("\nDocumentation:");
    println!("  Usage guide: {:?}", docs.usage_guide);
    for ex in &docs.examples {
        println!("  Example: {} -> {}", ex.title, ex.file);
    }

    // -----------------------------------------------------------------------
    // 5. Serialize everything to JSON (as it would appear in skill.json)
    // -----------------------------------------------------------------------

    println!("\n--- Serialized config_schema ---");
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    println!("\n--- Serialized setup_steps ---");
    println!("{}", serde_json::to_string_pretty(&steps).unwrap());

    println!("\n--- Serialized documentation ---");
    println!("{}", serde_json::to_string_pretty(&docs).unwrap());

    // -----------------------------------------------------------------------
    // 6. Demonstrate validation error helpers
    // -----------------------------------------------------------------------

    println!("\n--- Validation Error Helpers ---");

    let err1: ValidationResponse =
        validation_error_invalid_credentials("API key is expired");
    println!("Invalid credentials: {:?}", err1);

    let err2: ValidationResponse =
        validation_error_connection_failed("Could not reach api.openweathermap.org");
    println!("Connection failed: {:?}", err2);

    let err3: ValidationResponse =
        validation_error_timeout("Validation exceeded 30s limit");
    println!("Timeout: {:?}", err3);

    println!("\n=== Example complete ===");
}

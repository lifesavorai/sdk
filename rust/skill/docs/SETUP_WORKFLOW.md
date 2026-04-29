# Setup Workflow Guide

This guide covers how to define configuration schemas, multi-step setup
workflows, validation commands, and documentation bundling for Life Savor
skills using the Rust SDK.

## Overview

The setup workflow system lets skill developers declare what configuration a
skill needs and how users should provide it. The platform renders a guided
wizard from your declarations — you define the schema and steps, the platform
handles the UI.

Three pieces work together:

1. **ConfigSchema** — a JSON Schema object describing your skill's config fields
2. **SetupStep** — ordered steps that group fields into a guided experience
3. **SkillDocumentation** — Markdown files bundled with your skill for users and the LLM

All three are declared in your `skill.json` manifest and can be constructed
programmatically using the SDK builders.

## ConfigSchema

`ConfigSchema` represents a JSON Schema Draft 2020-12 object that defines
your skill's configurable parameters. Each property has a type, title,
description, and optional annotations like `x-secret` for sensitive values.

### Supported Field Types

| Type | Rust builder method | Description |
|------|-------------------|-------------|
| `string` | `add_string_field` | Text values (API keys, URLs, names) |
| `number` | `add_number_field` | Floating-point numbers |
| `integer` | `add_integer_field` | Whole numbers |
| `boolean` | `add_boolean_field` | True/false toggles |
| `array` | `add_array_field` | Lists of primitive values |

### ConfigSchemaBuilder

The `ConfigSchemaBuilder` provides a fluent API for constructing schemas with
compile-time safety. Each `add_*_field` method returns a `FieldBuilder` that
lets you configure the field before calling `done()` to return to the schema
builder.

```rust
use lifesavor_skill_sdk::config_builder::ConfigSchemaBuilder;

let schema = ConfigSchemaBuilder::new()
    // Required secret string field
    .add_string_field("api_key", "API Key", "Your service API key")
        .required()
        .secret()
        .done()
    // String field with enum constraint and default
    .add_string_field("region", "Region", "Deployment region")
        .default_value(serde_json::json!("us-east-1"))
        .done()
    // Integer field with default
    .add_integer_field("max_retries", "Max Retries", "Maximum retry attempts")
        .default_value(serde_json::json!(3))
        .done()
    // Boolean field with default
    .add_boolean_field("verbose", "Verbose Logging", "Enable detailed log output")
        .default_value(serde_json::json!(false))
        .done()
    // Array field
    .add_array_field("tags", "Tags", "Resource tags", "string")
        .done()
    .build()
    .expect("schema should be valid");

assert_eq!(schema.properties.len(), 5);
assert_eq!(schema.required, vec!["api_key"]);
```

### FieldBuilder Methods

| Method | Description |
|--------|-------------|
| `required()` | Marks the field as required in the schema |
| `secret()` | Sets `x-secret: true` — value is stored in the Vault and masked in UI |
| `default_value(val)` | Sets a default value for the field |
| `done()` | Finalizes the field and returns to the parent `ConfigSchemaBuilder` |

### Validation

`ConfigSchemaBuilder::build()` returns `Err(SkillSdkError::ConfigBuilder(..))` if
the schema has no properties. The resulting `ConfigSchema` serializes to JSON
Schema Draft 2020-12 format with deterministic key ordering (via `BTreeMap`).

## SetupStep

A `SetupStep` groups related config fields into a single step of the setup
wizard. Each step has:

- `step_id` — unique identifier (e.g., `"credentials"`, `"preferences"`)
- `title` — display title (3–100 characters)
- `description` — help text (10–500 characters)
- `fields` — list of config field names from the schema
- `validation_command` — optional operation name for server-side validation

### SetupWorkflowBuilder

The `SetupWorkflowBuilder` constructs an ordered list of steps and validates
that all field references exist in the associated schema.

```rust
use lifesavor_skill_sdk::config_builder::{ConfigSchemaBuilder, SetupWorkflowBuilder};

let schema = ConfigSchemaBuilder::new()
    .add_string_field("api_key", "API Key", "Your API key")
        .required()
        .secret()
        .done()
    .add_string_field("location", "Location", "Default city")
        .required()
        .done()
    .add_boolean_field("notifications", "Notifications", "Enable alerts")
        .default_value(serde_json::json!(true))
        .done()
    .build()
    .unwrap();

let steps = SetupWorkflowBuilder::new(schema)
    // Step with a validation command
    .add_step_with_validation(
        "credentials",
        "API Credentials",
        "Enter your API key to connect the service",
        &["api_key"],
        "validate_api_key",
    )
    // Step without validation
    .add_step(
        "preferences",
        "Preferences",
        "Configure your alert preferences",
        &["location", "notifications"],
    )
    .build()
    .expect("workflow should be valid");

assert_eq!(steps.len(), 2);
assert_eq!(steps[0].validation_command, Some("validate_api_key".to_string()));
assert!(steps[1].validation_command.is_none());
```

### Builder Validation Rules

`SetupWorkflowBuilder::build()` returns an error if:

- A field name referenced in a step does not exist in the `ConfigSchema`
- The same field appears in more than one step
- Two steps share the same `step_id`

```rust
// This will fail — "nonexistent" is not in the schema
let result = SetupWorkflowBuilder::new(schema)
    .add_step("s1", "Step One", "A step referencing a bad field", &["nonexistent"])
    .build();

assert!(result.is_err());
```

## Validation Commands

A `validation_command` is a skill operation that the agent invokes to validate
a step's field values before advancing the user to the next step. This is how
you implement server-side validation — for example, testing that an API key
is valid by making a real API call.

### How It Works

1. User submits values for a setup step
2. Agent validates values against the `ConfigSchema` (type checking)
3. If the step has a `validation_command`, the agent invokes it via the Skill Executor
4. The skill receives a `ValidationRequest` on stdin and returns a `ValidationResponse` on stdout
5. On success, the agent advances to the next step
6. On failure, the error message is shown to the user

### Implementing a Validation Command

Use the `validation_handler` function to wrap your validation logic:

```rust
use lifesavor_skill_sdk::validation_handler::{
    validation_handler,
    validation_error_invalid_credentials,
    validation_error_connection_failed,
};
use lifesavor_skill_sdk::{ValidationRequest, ValidationResponse};

fn main() {
    validation_handler(|req: ValidationRequest| {
        let api_key = req.values["api_key"].as_str().unwrap_or("");

        if api_key.is_empty() {
            return validation_error_invalid_credentials("API key is required");
        }

        // In a real skill, you would call the external API here
        // to verify the key is valid.

        ValidationResponse {
            status: "success".to_string(),
            message: Some("API key verified".to_string()),
            data: None,
        }
    })
    .expect("validation handler failed");
}
```

### ValidationRequest

The agent sends a `ValidationRequest` with:

| Field | Type | Description |
|-------|------|-------------|
| `step_id` | `String` | The step being validated |
| `values` | `serde_json::Value` | Field values submitted by the user |
| `context.skill_id` | `Option<String>` | The skill being configured |
| `context.is_reconfigure` | `bool` | Whether this is initial setup or reconfiguration |

### ValidationResponse

Your handler returns a `ValidationResponse`:

| Field | Type | Description |
|-------|------|-------------|
| `status` | `String` | `"success"` or `"failure"` |
| `message` | `Option<String>` | Human-readable message (error details on failure) |
| `data` | `Option<Value>` | Optional transformed values or additional data |

### Error Helpers

The SDK provides pre-formatted error responses for common failure modes:

| Helper | Use Case |
|--------|----------|
| `validation_error_invalid_credentials(msg)` | API key or token is invalid/expired |
| `validation_error_connection_failed(msg)` | Cannot reach the external service |
| `validation_error_timeout(msg)` | Validation took too long |

### Testing Validation Commands

Use the Developer CLI to test validation commands locally:

```bash
# Test the credentials step with sample values
lsai-cli skill config test-validation \
    --step credentials \
    --values '{"api_key": "sk-test-12345"}'

# Preview the full setup workflow
lsai-cli skill config preview
```

## Documentation Bundling

Skills can bundle Markdown documentation that gets ingested into the agent's
Knowledge Store for LLM retrieval. This means Savo can answer questions about
your skill using your own documentation.

### Manifest Declaration

Add a `documentation` object to your `skill.json`:

```json
{
  "documentation": {
    "usage_guide": "docs/USAGE.md",
    "examples": [
      {
        "title": "Setting Up Multiple Locations",
        "file": "docs/examples/multi-location.md"
      }
    ]
  }
}
```

### Directory Structure

```
my-skill/
├── skill.json
├── index.js (or compiled binary)
├── docs/
│   ├── USAGE.md              ← main usage guide
│   └── examples/
│       └── multi-location.md ← additional example docs
└── ...
```

### Requirements

- All documentation files must use the `.md` extension
- Each file must not exceed 100 KB
- Paths are relative to the skill directory
- The `usage_guide` is the primary document — write it for both human readers
  and LLM retrieval

### What to Include

A good `USAGE.md` covers:

- **Overview** — what the skill does in 2–3 sentences
- **Getting Started** — step-by-step setup instructions
- **Configuration Reference** — table of all config fields with types and defaults
- **Usage Examples** — how to interact with the skill via Savo
- **Troubleshooting** — common issues and fixes

See `templates/docs/USAGE.md` for a complete example.

### How Ingestion Works

When a skill with documentation is installed:

1. The agent reads each referenced Markdown file
2. Content is chunked using the existing chunking pipeline
3. Embeddings are generated and stored in the Knowledge Store
4. Chunks are tagged with `source_type=skill_doc` and `source_id={skill_id}:usage_guide`

When a user asks Savo about the skill, the Knowledge Store performs semantic
search against these chunks and includes the most relevant ones in the LLM
context.

## Putting It All Together

Here is a complete example combining all three pieces:

```rust
use lifesavor_skill_sdk::config_builder::{ConfigSchemaBuilder, SetupWorkflowBuilder};
use lifesavor_skill_sdk::{SkillDocumentation, DocumentationExample};

// 1. Define the configuration schema
let schema = ConfigSchemaBuilder::new()
    .add_string_field("api_key", "API Key", "Your OpenWeatherMap API key")
        .required()
        .secret()
        .done()
    .add_string_field("location", "Location", "City name or coordinates")
        .required()
        .done()
    .add_integer_field("polling_interval", "Polling Interval", "Minutes between checks")
        .default_value(serde_json::json!(15))
        .done()
    .add_boolean_field("notifications", "Notifications", "Enable push notifications")
        .default_value(serde_json::json!(true))
        .done()
    .build()
    .expect("schema is valid");

// 2. Define the setup workflow
let steps = SetupWorkflowBuilder::new(schema.clone())
    .add_step_with_validation(
        "credentials",
        "API Credentials",
        "Enter your OpenWeatherMap API key",
        &["api_key"],
        "validate_api_key",
    )
    .add_step(
        "preferences",
        "Preferences",
        "Configure your location and notification settings",
        &["location", "polling_interval", "notifications"],
    )
    .build()
    .expect("workflow is valid");

// 3. Define documentation references
let docs = SkillDocumentation {
    usage_guide: Some("docs/USAGE.md".to_string()),
    examples: vec![
        DocumentationExample {
            title: "Setting Up Multiple Locations".to_string(),
            file: "docs/examples/multi-location.md".to_string(),
        },
    ],
};

// The schema, steps, and docs are serialized into your skill.json manifest.
let schema_json = serde_json::to_string_pretty(&schema).unwrap();
let steps_json = serde_json::to_string_pretty(&steps).unwrap();
let docs_json = serde_json::to_string_pretty(&docs).unwrap();
```

## Further Reading

- [Getting Started](GETTING_STARTED.md) — building your first skill
- [Deployment Guide](DEPLOYMENT.md) — deploying skills to the agent
- [examples/config_schema_skill.rs](../examples/config_schema_skill.rs) — runnable example
- [Skill SDK README](../README.md) — full SDK reference

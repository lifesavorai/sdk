/**
 * Property-based tests for ValidationRequest serialization round-trip.
 *
 * Property 3: ValidationRequest serialization round-trip — any valid
 * ValidationRequest with varied step IDs, JSON value payloads, and context
 * metadata serializes to JSON and deserializes back to an equivalent object.
 *
 * **Validates: Requirements 11.9**
 */

use lifesavor_agent_types::skill_config::{ValidationContext, ValidationRequest, ValidationResponse};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Arbitrary generators
// ---------------------------------------------------------------------------

/// Generate a valid step ID (lowercase alphanumeric with underscores/hyphens).
fn arb_step_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{1,29}"
}

/// Generate a skill ID for the validation context.
fn arb_skill_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{2,39}"
}

/// Generate a JSON value payload representing user-submitted field values.
/// Produces objects with string, number, and boolean values to simulate
/// realistic config submissions.
fn arb_values_payload() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        // Empty object
        Just(serde_json::json!({})),
        // Object with a single string field
        "[a-z]{1,10}".prop_map(|v| serde_json::json!({ "field": v })),
        // Object with mixed types
        ("[a-z]{1,10}", 0i64..1000, any::<bool>()).prop_map(|(s, n, b)| {
            serde_json::json!({
                "api_key": s,
                "timeout": n,
                "enabled": b
            })
        }),
        // Object with nested values
        ("[a-z]{1,10}", "[a-z]{1,10}").prop_map(|(a, b)| {
            serde_json::json!({
                "name": a,
                "tags": [a, b],
                "config": { "nested_key": b }
            })
        }),
        // Object with numeric-only fields (use integers to avoid f64 precision loss)
        (0i64..10000, -1000i64..1000).prop_map(|(i, n)| {
            serde_json::json!({
                "count": i,
                "ratio": n
            })
        }),
    ]
}

/// Generate an arbitrary ValidationContext with optional skill_id and
/// varied is_reconfigure flags.
fn arb_validation_context() -> impl Strategy<Value = ValidationContext> {
    (
        prop_oneof![
            Just(None),
            arb_skill_id().prop_map(Some),
        ],
        any::<bool>(),
    )
        .prop_map(|(skill_id, is_reconfigure)| ValidationContext {
            skill_id,
            is_reconfigure,
        })
}

/// Generate an arbitrary ValidationRequest with varied step IDs, value
/// payloads, and context metadata.
fn arb_validation_request() -> impl Strategy<Value = ValidationRequest> {
    (arb_step_id(), arb_values_payload(), arb_validation_context()).prop_map(
        |(step_id, values, context)| ValidationRequest {
            step_id,
            values,
            context,
        },
    )
}

/// Generate an optional human-readable message string.
fn arb_optional_message() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[A-Za-z][A-Za-z0-9 .,!]{0,99}".prop_map(Some),
    ]
}

/// Generate an optional data payload for ValidationResponse.
fn arb_optional_data() -> impl Strategy<Value = Option<serde_json::Value>> {
    prop_oneof![
        Just(None),
        Just(Some(serde_json::json!({}))),
        Just(Some(serde_json::json!({ "transformed": true }))),
        "[a-z]{1,10}".prop_map(|v| Some(serde_json::json!({ "key": v }))),
    ]
}

/// Generate an arbitrary ValidationResponse with success/failure status,
/// optional message, and optional data payload.
fn arb_validation_response() -> impl Strategy<Value = ValidationResponse> {
    (
        prop_oneof![Just("success".to_string()), Just("failure".to_string())],
        arb_optional_message(),
        arb_optional_data(),
    )
        .prop_map(|(status, message, data)| ValidationResponse {
            status,
            message,
            data,
        })
}

// ---------------------------------------------------------------------------
// Property 3: ValidationRequest serialization round-trip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Serializing a ValidationRequest to JSON and deserializing back produces
    /// an equivalent ValidationRequest object.
    #[test]
    fn prop_validation_request_json_round_trip(request in arb_validation_request()) {
        let json = serde_json::to_string(&request)
            .expect("ValidationRequest should serialize to JSON");
        let deserialized: ValidationRequest = serde_json::from_str(&json)
            .expect("ValidationRequest should deserialize from JSON");

        // Structural field checks
        prop_assert_eq!(&request.step_id, &deserialized.step_id,
            "step_id must survive round-trip");
        prop_assert_eq!(&request.values, &deserialized.values,
            "values must survive round-trip");
        prop_assert_eq!(&request.context.skill_id, &deserialized.context.skill_id,
            "context.skill_id must survive round-trip");
        prop_assert_eq!(request.context.is_reconfigure, deserialized.context.is_reconfigure,
            "context.is_reconfigure must survive round-trip");

        // Full equality
        prop_assert_eq!(&request, &deserialized,
            "Round-tripped ValidationRequest must be equivalent to original");
    }

    /// ValidationContext round-trips correctly in isolation, covering the
    /// optional skill_id and boolean is_reconfigure fields.
    #[test]
    fn prop_validation_context_json_round_trip(context in arb_validation_context()) {
        let json = serde_json::to_string(&context)
            .expect("ValidationContext should serialize to JSON");
        let deserialized: ValidationContext = serde_json::from_str(&json)
            .expect("ValidationContext should deserialize from JSON");

        prop_assert_eq!(&context, &deserialized,
            "Round-tripped ValidationContext must be equivalent to original");
    }

    /// ValidationResponse round-trips correctly, covering success/failure
    /// status, optional message, and optional data payload.
    #[test]
    fn prop_validation_response_json_round_trip(response in arb_validation_response()) {
        let json = serde_json::to_string(&response)
            .expect("ValidationResponse should serialize to JSON");
        let deserialized: ValidationResponse = serde_json::from_str(&json)
            .expect("ValidationResponse should deserialize from JSON");

        prop_assert_eq!(&response.status, &deserialized.status,
            "status must survive round-trip");
        prop_assert_eq!(&response.message, &deserialized.message,
            "message must survive round-trip");
        prop_assert_eq!(&response.data, &deserialized.data,
            "data must survive round-trip");

        prop_assert_eq!(&response, &deserialized,
            "Round-tripped ValidationResponse must be equivalent to original");
    }
}

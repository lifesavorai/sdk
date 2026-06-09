//! Adapter dependency declaration and runtime loading types for fine-tuned
//! model adapters (LoRA/QLoRA).
//!
//! This module provides:
//! - [`AdapterDependency`] — declares an adapter dependency in a skill manifest
//! - [`AdapterType`] — enum distinguishing LoRA from QLoRA adapters
//! - [`AdapterLoadRequest`] — runtime request to load an adapter onto a base model
//! - [`AdapterLoadResult`] — successful outcomes of adapter load operations
//! - [`AdapterLoadError`] — error conditions from adapter load operations
//! - [`validate_adapter_dependency`] — validates an [`AdapterDependency`] for correctness
//!
//! # Manifest TOML Format
//!
//! Adapter dependencies are serialized into an `[[adapter_dependencies]]` array
//! in the skill's `provider-manifest.toml`:
//!
//! ```toml
//! [[adapter_dependencies]]
//! name = "medical-pii-lora"
//! adapter_type = "lora"
//! base_model_architecture = "llama3"
//! min_base_params = 7000
//! source_url = "https://cdn.lifesavor.dev/adapters/medical-pii-lora/1.2.0/"
//! version_constraint = ">=1.0.0, <2.0.0"
//! ```
//!
//! # Runtime Loading
//!
//! Skills request adapter loading at runtime via [`request_adapter_load`], which
//! sends a JSON-RPC `adapter.load` message to the host agent. The agent handles
//! state transitions, compatibility checks, and queuing.

use std::time::Duration;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AdapterType
// ---------------------------------------------------------------------------

/// Type of fine-tuned adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterType {
    /// Low-Rank Adaptation — full-precision adapter weights.
    LoRA,
    /// Quantized Low-Rank Adaptation — reduced-precision adapter weights.
    QLoRA,
}

impl std::fmt::Display for AdapterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoRA => write!(f, "lora"),
            Self::QLoRA => write!(f, "qlora"),
        }
    }
}

impl std::str::FromStr for AdapterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lora" => Ok(Self::LoRA),
            "qlora" => Ok(Self::QLoRA),
            other => Err(format!(
                "invalid adapter type '{}': expected 'lora' or 'qlora'",
                other
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// AdapterDependency
// ---------------------------------------------------------------------------

/// Declares an adapter dependency in a skill manifest.
///
/// This type is serialized into the `[[adapter_dependencies]]` section of the
/// skill's provider manifest TOML. Each field maps directly to a manifest key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterDependency {
    /// Human-readable adapter name (e.g., "medical-pii-lora").
    pub name: String,

    /// Adapter type (LoRA or QLoRA).
    pub adapter_type: AdapterType,

    /// Compatible base model architecture (e.g., "llama3", "mistral").
    pub base_model_architecture: String,

    /// Minimum parameter count of the base model (in millions).
    pub min_base_params: u64,

    /// Source URL for downloading the adapter artifact.
    pub source_url: String,

    /// SemVer version constraint (e.g., ">=1.0.0, <2.0.0").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_constraint: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validation error for an [`AdapterDependency`].
#[derive(Debug, Clone, PartialEq)]
pub struct AdapterDependencyValidationError {
    /// Which field(s) are invalid.
    pub field: String,
    /// Human-readable description of the problem.
    pub message: String,
}

impl std::fmt::Display for AdapterDependencyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for AdapterDependencyValidationError {}

/// Validate an [`AdapterDependency`] for correctness.
///
/// Returns a list of validation errors. An empty list means the dependency
/// is valid.
///
/// Checks:
/// - `name` must be non-empty
/// - `adapter_type` must be LoRA or QLoRA (enforced by the enum, but checked
///   for completeness in TOML round-trip scenarios)
/// - `base_model_architecture` must be non-empty
/// - `source_url` must be non-empty
pub fn validate_adapter_dependency_spec(
    dep: &AdapterDependency,
) -> Vec<AdapterDependencyValidationError> {
    let mut errors = Vec::new();

    if dep.name.trim().is_empty() {
        errors.push(AdapterDependencyValidationError {
            field: "name".to_string(),
            message: "adapter name must not be empty".to_string(),
        });
    }

    if dep.base_model_architecture.trim().is_empty() {
        errors.push(AdapterDependencyValidationError {
            field: "base_model_architecture".to_string(),
            message: "base model architecture must not be empty".to_string(),
        });
    }

    if dep.source_url.trim().is_empty() {
        errors.push(AdapterDependencyValidationError {
            field: "source_url".to_string(),
            message: "source URL must not be empty".to_string(),
        });
    }

    errors
}

// ---------------------------------------------------------------------------
// AdapterLoadRequest
// ---------------------------------------------------------------------------

/// Request to load an adapter at runtime.
///
/// Sent by the skill to the host agent via JSON-RPC `adapter.load`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterLoadRequest {
    /// Name of the adapter to load.
    pub adapter_name: String,

    /// Target base model name.
    pub base_model: String,

    /// Optional timeout for the load operation.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "optional_duration_millis"
    )]
    pub timeout: Option<Duration>,

    /// If true, demote the currently Hot adapter before applying this one.
    #[serde(default)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// AdapterLoadResult
// ---------------------------------------------------------------------------

/// Result of a successful adapter load request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AdapterLoadResult {
    /// Adapter successfully applied (now Hot).
    AdapterApplied {
        adapter_name: String,
        base_model: String,
    },

    /// Adapter loaded to Warm but not applied (another adapter is Hot).
    AdapterWarmOnly {
        adapter_name: String,
        current_hot: String,
    },

    /// Adapter request was queued (will be applied when current adapter is released).
    Queued {
        position: usize,
        queue_depth: usize,
    },
}

// ---------------------------------------------------------------------------
// AdapterLoadError
// ---------------------------------------------------------------------------

/// Errors from adapter load operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
pub enum AdapterLoadError {
    #[error("Base model '{model}' is not loaded (Cold state)")]
    BaseModelNotLoaded { model: String },

    #[error("Adapter '{adapter}' not found for model '{model}'")]
    AdapterNotFound { adapter: String, model: String },

    #[error("Incompatible architecture: adapter expects '{expected}', model is '{actual}'")]
    IncompatibleArchitecture { expected: String, actual: String },

    #[error("Insufficient base model: requires {required}M params, model has {available}M")]
    InsufficientBaseModel { required: u64, available: u64 },

    #[error("Checksum mismatch for file '{file}': expected {expected}, got {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    #[error("Adapter queue full (depth {depth}/{max})")]
    AdapterQueueFull { depth: usize, max: usize },

    #[error("Queue timeout after {elapsed_ms}ms")]
    QueueTimeout { elapsed_ms: u64 },

    #[error("Target module '{module}' not found in base model")]
    TargetModuleNotFound { module: String },

    #[error("Adapter disk limit exceeded: {used_gb:.2} GB / {max_gb:.2} GB")]
    AdapterDiskLimitExceeded { used_gb: f64, max_gb: f64 },

    #[error("Adapter limit exceeded for model '{model}': {count}/{max}")]
    AdapterLimitExceeded {
        model: String,
        count: usize,
        max: usize,
    },

    #[error("No capable fleet node available for adapter inference")]
    NoCapableNode,

    #[error("Delegation failed: target agent unreachable after {timeout_ms}ms")]
    DelegationTimeout { timeout_ms: u64 },
}

// ---------------------------------------------------------------------------
// TOML serialization for adapter_dependencies section
// ---------------------------------------------------------------------------

/// Wrapper for serializing/deserializing the `[[adapter_dependencies]]`
/// section in a skill manifest TOML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterDependenciesSection {
    /// Array of adapter dependencies declared by the skill.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adapter_dependencies: Vec<AdapterDependency>,
}

impl AdapterDependenciesSection {
    /// Create a new section from a list of adapter dependencies.
    pub fn new(deps: Vec<AdapterDependency>) -> Self {
        Self {
            adapter_dependencies: deps,
        }
    }

    /// Serialize the section to a TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserialize the section from a TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

// ---------------------------------------------------------------------------
// Runtime API — JSON-RPC integration via host channel
// ---------------------------------------------------------------------------

/// Request adapter loading from the agent runtime.
///
/// Sends a JSON-RPC `adapter.load` message to the host agent via the host
/// communication channel and awaits the result. The agent handles validation,
/// state transitions, queuing, and compatibility checks.
///
/// # Protocol
///
/// The function constructs a `SystemCallRequest` with:
/// - `component`: `"adapter"`
/// - `operation`: `"load"`
/// - `params`: serialized [`AdapterLoadRequest`]
///
/// The host agent responds with either a successful [`AdapterLoadResult`]
/// or an error that maps to [`AdapterLoadError`].
///
/// # Errors
///
/// Returns [`AdapterLoadError`] if the load operation fails (base model not
/// loaded, adapter not found, incompatible, queue full, etc.).
#[cfg(feature = "runtime-api")]
pub async fn request_adapter_load(
    request: AdapterLoadRequest,
) -> Result<AdapterLoadResult, AdapterLoadError> {
    use crate::host_channel::{current_skill_id, send_system_call, HostChannelError};

    let params = serde_json::to_value(&request).map_err(|e| AdapterLoadError::AdapterNotFound {
        adapter: request.adapter_name.clone(),
        model: format!("serialization failed: {}", e),
    })?;

    let result = send_system_call("adapter", "load", params, &current_skill_id()).await;

    match result {
        Ok(value) => {
            // Deserialize the successful response into AdapterLoadResult.
            serde_json::from_value::<AdapterLoadResult>(value).map_err(|e| {
                AdapterLoadError::AdapterNotFound {
                    adapter: request.adapter_name.clone(),
                    model: format!("failed to parse adapter load result: {}", e),
                }
            })
        }
        Err(HostChannelError::HostError { code, message }) => {
            // Map the host error code to the appropriate AdapterLoadError variant.
            Err(map_host_error_to_adapter_error(
                &code,
                &message,
                &request.adapter_name,
                &request.base_model,
            ))
        }
        Err(other) => {
            // Communication failure — wrap as a generic error.
            Err(AdapterLoadError::AdapterNotFound {
                adapter: request.adapter_name,
                model: format!("host communication failed: {}", other),
            })
        }
    }
}

/// Stub for `request_adapter_load` when runtime-api feature is not enabled.
///
/// This version is available for type-checking and documentation but will
/// panic at runtime. Enable the `runtime-api` feature to use the real
/// implementation.
#[cfg(not(feature = "runtime-api"))]
pub async fn request_adapter_load(
    _request: AdapterLoadRequest,
) -> Result<AdapterLoadResult, AdapterLoadError> {
    panic!(
        "request_adapter_load requires the 'runtime-api' feature. \
         Add `features = [\"runtime-api\"]` to your lifesavor-skill-sdk dependency."
    )
}

/// Signal the agent that this skill no longer needs the adapter Hot.
///
/// Allows queued adapter requests to proceed. Sends a JSON-RPC
/// `adapter.release` message to the host agent via the host communication
/// channel.
///
/// # Protocol
///
/// The function constructs a `SystemCallRequest` with:
/// - `component`: `"adapter"`
/// - `operation`: `"release"`
/// - `params`: `{ "adapter_name": "...", "base_model": "..." }`
///
/// # Errors
///
/// Returns [`AdapterLoadError`] if the release fails (adapter not found,
/// communication error).
#[cfg(feature = "runtime-api")]
pub async fn release_adapter(
    adapter_name: &str,
    base_model: &str,
) -> Result<(), AdapterLoadError> {
    use crate::host_channel::{current_skill_id, send_system_call, HostChannelError};

    let params = serde_json::json!({
        "adapter_name": adapter_name,
        "base_model": base_model,
    });

    let result = send_system_call("adapter", "release", params, &current_skill_id()).await;

    match result {
        Ok(_) => Ok(()),
        Err(HostChannelError::HostError { code, message }) => Err(
            map_host_error_to_adapter_error(&code, &message, adapter_name, base_model),
        ),
        Err(other) => Err(AdapterLoadError::AdapterNotFound {
            adapter: adapter_name.to_string(),
            model: format!("host communication failed: {}", other),
        }),
    }
}

/// Stub for `release_adapter` when runtime-api feature is not enabled.
#[cfg(not(feature = "runtime-api"))]
pub async fn release_adapter(
    _adapter_name: &str,
    _base_model: &str,
) -> Result<(), AdapterLoadError> {
    panic!(
        "release_adapter requires the 'runtime-api' feature. \
         Add `features = [\"runtime-api\"]` to your lifesavor-skill-sdk dependency."
    )
}

// ---------------------------------------------------------------------------
// Error mapping helper
// ---------------------------------------------------------------------------

/// Maps a host error code and message to the appropriate [`AdapterLoadError`]
/// variant.
///
/// The host agent returns structured error codes that correspond to specific
/// failure modes. This function parses the error message JSON (when present)
/// to extract typed error fields.
#[cfg(feature = "runtime-api")]
fn map_host_error_to_adapter_error(
    code: &str,
    message: &str,
    adapter_name: &str,
    base_model: &str,
) -> AdapterLoadError {
    // Try to parse the message as JSON for structured error data.
    let error_data: serde_json::Value =
        serde_json::from_str(message).unwrap_or(serde_json::Value::Null);

    match code {
        "BASE_MODEL_NOT_LOADED" => AdapterLoadError::BaseModelNotLoaded {
            model: error_data["model"]
                .as_str()
                .unwrap_or(base_model)
                .to_string(),
        },
        "ADAPTER_NOT_FOUND" => AdapterLoadError::AdapterNotFound {
            adapter: error_data["adapter"]
                .as_str()
                .unwrap_or(adapter_name)
                .to_string(),
            model: error_data["model"]
                .as_str()
                .unwrap_or(base_model)
                .to_string(),
        },
        "INCOMPATIBLE_ARCHITECTURE" => AdapterLoadError::IncompatibleArchitecture {
            expected: error_data["expected"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            actual: error_data["actual"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        },
        "INSUFFICIENT_BASE_MODEL" => AdapterLoadError::InsufficientBaseModel {
            required: error_data["required"].as_u64().unwrap_or(0),
            available: error_data["available"].as_u64().unwrap_or(0),
        },
        "CHECKSUM_MISMATCH" => AdapterLoadError::ChecksumMismatch {
            file: error_data["file"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            expected: error_data["expected"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            actual: error_data["actual"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        },
        "ADAPTER_QUEUE_FULL" => AdapterLoadError::AdapterQueueFull {
            depth: error_data["depth"].as_u64().unwrap_or(8) as usize,
            max: error_data["max"].as_u64().unwrap_or(8) as usize,
        },
        "QUEUE_TIMEOUT" => AdapterLoadError::QueueTimeout {
            elapsed_ms: error_data["elapsed_ms"].as_u64().unwrap_or(0),
        },
        "TARGET_MODULE_NOT_FOUND" => AdapterLoadError::TargetModuleNotFound {
            module: error_data["module"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        },
        "ADAPTER_DISK_LIMIT_EXCEEDED" => AdapterLoadError::AdapterDiskLimitExceeded {
            used_gb: error_data["used_gb"].as_f64().unwrap_or(0.0),
            max_gb: error_data["max_gb"].as_f64().unwrap_or(10.0),
        },
        "ADAPTER_LIMIT_EXCEEDED" => AdapterLoadError::AdapterLimitExceeded {
            model: error_data["model"]
                .as_str()
                .unwrap_or(base_model)
                .to_string(),
            count: error_data["count"].as_u64().unwrap_or(0) as usize,
            max: error_data["max"].as_u64().unwrap_or(16) as usize,
        },
        "NO_CAPABLE_NODE" => AdapterLoadError::NoCapableNode,
        "DELEGATION_TIMEOUT" => AdapterLoadError::DelegationTimeout {
            timeout_ms: error_data["timeout_ms"].as_u64().unwrap_or(30000),
        },
        _ => {
            // Unknown error code — use AdapterNotFound as a generic fallback.
            AdapterLoadError::AdapterNotFound {
                adapter: adapter_name.to_string(),
                model: format!("{}: {}", code, message),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Serde helper: Duration as milliseconds
// ---------------------------------------------------------------------------

mod optional_duration_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(d) => d.as_millis().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis: Option<u64> = Option::deserialize(deserializer)?;
        Ok(millis.map(Duration::from_millis))
    }
}

// ---------------------------------------------------------------------------
// Property-based tests (proptest)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Strategies
    // -----------------------------------------------------------------------

    /// Generate a valid (non-empty, non-whitespace-only) adapter name.
    fn arb_non_empty_name() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9\\-]{0,63}".prop_map(|s| s.to_string())
    }

    /// Generate an arbitrary AdapterType.
    fn arb_adapter_type() -> impl Strategy<Value = AdapterType> {
        prop_oneof![Just(AdapterType::LoRA), Just(AdapterType::QLoRA),]
    }

    /// Generate a valid (non-empty) base model architecture string.
    fn arb_non_empty_architecture() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9\\-]{0,31}".prop_map(|s| s.to_string())
    }

    /// Generate a valid (non-empty) source URL string.
    fn arb_non_empty_source_url() -> impl Strategy<Value = String> {
        "https://[a-z]{3,12}\\.[a-z]{2,6}/[a-z0-9/\\-]{1,64}".prop_map(|s| s.to_string())
    }

    /// Generate an optional version constraint string.
    fn arb_version_constraint() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            Just(None),
            Just(Some(">=1.0.0, <2.0.0".to_string())),
            Just(Some(">=0.1.0".to_string())),
            Just(Some("^1.2.3".to_string())),
        ]
    }

    /// Generate a non-negative min_base_params value that fits in TOML's i64.
    /// TOML integers are signed 64-bit, so we constrain to 0..=i64::MAX.
    fn arb_min_base_params() -> impl Strategy<Value = u64> {
        0u64..=(i64::MAX as u64)
    }

    /// Generate a valid AdapterDependency (all fields satisfy validation).
    fn arb_valid_adapter_dependency() -> impl Strategy<Value = AdapterDependency> {
        (
            arb_non_empty_name(),
            arb_adapter_type(),
            arb_non_empty_architecture(),
            arb_min_base_params(),
            arb_non_empty_source_url(),
            arb_version_constraint(),
        )
            .prop_map(
                |(name, adapter_type, base_model_architecture, min_base_params, source_url, version_constraint)| {
                    AdapterDependency {
                        name,
                        adapter_type,
                        base_model_architecture,
                        min_base_params,
                        source_url,
                        version_constraint,
                    }
                },
            )
    }

    // -----------------------------------------------------------------------
    // Property 1: AdapterDependency round-trip serialization
    // -----------------------------------------------------------------------

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 1: AdapterDependency round-trip serialization**
        ///
        /// For any valid AdapterDependency value (non-empty name, valid adapter
        /// type, non-empty architecture, non-negative params, non-empty source
        /// URL), serializing it to TOML and then deserializing the TOML back
        /// SHALL produce an AdapterDependency that is equal to the original.
        ///
        /// **Validates: Requirements 1.4, 1.5**
        #[test]
        fn prop_adapter_dependency_toml_roundtrip(dep in arb_valid_adapter_dependency()) {
            let section = AdapterDependenciesSection::new(vec![dep.clone()]);
            let toml_str = section.to_toml().expect("serialization should succeed");
            let parsed = AdapterDependenciesSection::from_toml(&toml_str)
                .expect("deserialization should succeed");

            prop_assert_eq!(parsed.adapter_dependencies.len(), 1);
            prop_assert_eq!(&parsed.adapter_dependencies[0], &dep);
        }

        /// Round-trip for multiple adapter dependencies in a single section.
        ///
        /// **Validates: Requirements 1.4, 1.5**
        #[test]
        fn prop_adapter_dependency_toml_roundtrip_multiple(
            dep1 in arb_valid_adapter_dependency(),
            dep2 in arb_valid_adapter_dependency(),
        ) {
            let section = AdapterDependenciesSection::new(vec![dep1.clone(), dep2.clone()]);
            let toml_str = section.to_toml().expect("serialization should succeed");
            let parsed = AdapterDependenciesSection::from_toml(&toml_str)
                .expect("deserialization should succeed");

            prop_assert_eq!(parsed.adapter_dependencies.len(), 2);
            prop_assert_eq!(&parsed.adapter_dependencies[0], &dep1);
            prop_assert_eq!(&parsed.adapter_dependencies[1], &dep2);
        }
    }

    // -----------------------------------------------------------------------
    // Property 2: AdapterDependency validation rejects invalid inputs
    // -----------------------------------------------------------------------

    /// Generate an empty or whitespace-only string.
    fn arb_empty_or_whitespace() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),
            Just(" ".to_string()),
            Just("  ".to_string()),
            Just("\t".to_string()),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 2a: Validation rejects empty name**
        ///
        /// For any AdapterDependency where the name is empty or whitespace-only,
        /// validation SHALL return an error.
        ///
        /// **Validates: Requirements 1.3**
        #[test]
        fn prop_validation_rejects_empty_name(
            empty_name in arb_empty_or_whitespace(),
            adapter_type in arb_adapter_type(),
            architecture in arb_non_empty_architecture(),
            min_base_params in arb_min_base_params(),
            source_url in arb_non_empty_source_url(),
        ) {
            let dep = AdapterDependency {
                name: empty_name,
                adapter_type,
                base_model_architecture: architecture,
                min_base_params,
                source_url,
                version_constraint: None,
            };
            let errors = validate_adapter_dependency_spec(&dep);
            prop_assert!(!errors.is_empty(), "expected validation error for empty name");
            prop_assert!(
                errors.iter().any(|e| e.field == "name"),
                "expected a 'name' field error"
            );
        }

        /// **Property 2b: Validation rejects empty base_model_architecture**
        ///
        /// For any AdapterDependency where the base_model_architecture is empty
        /// or whitespace-only, validation SHALL return an error.
        ///
        /// **Validates: Requirements 1.3**
        #[test]
        fn prop_validation_rejects_empty_architecture(
            name in arb_non_empty_name(),
            adapter_type in arb_adapter_type(),
            empty_arch in arb_empty_or_whitespace(),
            min_base_params in arb_min_base_params(),
            source_url in arb_non_empty_source_url(),
        ) {
            let dep = AdapterDependency {
                name,
                adapter_type,
                base_model_architecture: empty_arch,
                min_base_params,
                source_url,
                version_constraint: None,
            };
            let errors = validate_adapter_dependency_spec(&dep);
            prop_assert!(!errors.is_empty(), "expected validation error for empty architecture");
            prop_assert!(
                errors.iter().any(|e| e.field == "base_model_architecture"),
                "expected a 'base_model_architecture' field error"
            );
        }

        /// **Property 2c: Validation succeeds for valid inputs**
        ///
        /// For any AdapterDependency where all conditions are satisfied (non-empty
        /// name, valid adapter_type, non-empty architecture), validation SHALL
        /// succeed (return no errors).
        ///
        /// **Validates: Requirements 1.3**
        #[test]
        fn prop_validation_succeeds_for_valid_inputs(dep in arb_valid_adapter_dependency()) {
            let errors = validate_adapter_dependency_spec(&dep);
            prop_assert!(
                errors.is_empty(),
                "expected no validation errors for valid input, got: {:?}",
                errors
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_type_display() {
        assert_eq!(AdapterType::LoRA.to_string(), "lora");
        assert_eq!(AdapterType::QLoRA.to_string(), "qlora");
    }

    #[test]
    fn adapter_type_from_str() {
        assert_eq!("lora".parse::<AdapterType>().unwrap(), AdapterType::LoRA);
        assert_eq!("qlora".parse::<AdapterType>().unwrap(), AdapterType::QLoRA);
        assert_eq!("LoRA".parse::<AdapterType>().unwrap(), AdapterType::LoRA);
        assert_eq!("QLORA".parse::<AdapterType>().unwrap(), AdapterType::QLoRA);
        assert!("invalid".parse::<AdapterType>().is_err());
    }

    #[test]
    fn adapter_type_serde_roundtrip() {
        let lora = AdapterType::LoRA;
        let json = serde_json::to_string(&lora).unwrap();
        assert_eq!(json, "\"lora\"");
        let back: AdapterType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, lora);

        let qlora = AdapterType::QLoRA;
        let json = serde_json::to_string(&qlora).unwrap();
        assert_eq!(json, "\"qlora\"");
        let back: AdapterType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, qlora);
    }

    #[test]
    fn adapter_dependency_valid() {
        let dep = AdapterDependency {
            name: "medical-pii-lora".to_string(),
            adapter_type: AdapterType::LoRA,
            base_model_architecture: "llama3".to_string(),
            min_base_params: 7000,
            source_url: "https://cdn.lifesavor.dev/adapters/medical-pii-lora/1.2.0/".to_string(),
            version_constraint: Some(">=1.0.0, <2.0.0".to_string()),
        };
        let errors = validate_adapter_dependency_spec(&dep);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn adapter_dependency_empty_name_rejected() {
        let dep = AdapterDependency {
            name: "".to_string(),
            adapter_type: AdapterType::LoRA,
            base_model_architecture: "llama3".to_string(),
            min_base_params: 7000,
            source_url: "https://cdn.lifesavor.dev/x".to_string(),
            version_constraint: None,
        };
        let errors = validate_adapter_dependency_spec(&dep);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn adapter_dependency_empty_architecture_rejected() {
        let dep = AdapterDependency {
            name: "test-adapter".to_string(),
            adapter_type: AdapterType::QLoRA,
            base_model_architecture: "".to_string(),
            min_base_params: 7000,
            source_url: "https://cdn.lifesavor.dev/x".to_string(),
            version_constraint: None,
        };
        let errors = validate_adapter_dependency_spec(&dep);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "base_model_architecture");
    }

    #[test]
    fn adapter_dependency_empty_source_url_rejected() {
        let dep = AdapterDependency {
            name: "test-adapter".to_string(),
            adapter_type: AdapterType::LoRA,
            base_model_architecture: "mistral".to_string(),
            min_base_params: 7000,
            source_url: "".to_string(),
            version_constraint: None,
        };
        let errors = validate_adapter_dependency_spec(&dep);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "source_url");
    }

    #[test]
    fn adapter_dependency_multiple_errors() {
        let dep = AdapterDependency {
            name: "".to_string(),
            adapter_type: AdapterType::LoRA,
            base_model_architecture: "  ".to_string(),
            min_base_params: 0,
            source_url: "".to_string(),
            version_constraint: None,
        };
        let errors = validate_adapter_dependency_spec(&dep);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn adapter_dependency_toml_roundtrip() {
        let dep = AdapterDependency {
            name: "medical-pii-lora".to_string(),
            adapter_type: AdapterType::LoRA,
            base_model_architecture: "llama3".to_string(),
            min_base_params: 7000,
            source_url: "https://cdn.lifesavor.dev/adapters/medical-pii-lora/1.2.0/".to_string(),
            version_constraint: Some(">=1.0.0, <2.0.0".to_string()),
        };

        let section = AdapterDependenciesSection::new(vec![dep.clone()]);
        let toml_str = section.to_toml().unwrap();
        let parsed = AdapterDependenciesSection::from_toml(&toml_str).unwrap();
        assert_eq!(parsed.adapter_dependencies.len(), 1);
        assert_eq!(parsed.adapter_dependencies[0], dep);
    }

    #[test]
    fn adapter_dependency_toml_roundtrip_no_version_constraint() {
        let dep = AdapterDependency {
            name: "code-assist-qlora".to_string(),
            adapter_type: AdapterType::QLoRA,
            base_model_architecture: "mistral".to_string(),
            min_base_params: 7000,
            source_url: "https://cdn.lifesavor.dev/adapters/code-assist/2.0.0/".to_string(),
            version_constraint: None,
        };

        let section = AdapterDependenciesSection::new(vec![dep.clone()]);
        let toml_str = section.to_toml().unwrap();
        let parsed = AdapterDependenciesSection::from_toml(&toml_str).unwrap();
        assert_eq!(parsed.adapter_dependencies[0], dep);
    }

    #[test]
    fn adapter_dependency_toml_multiple_deps() {
        let deps = vec![
            AdapterDependency {
                name: "adapter-a".to_string(),
                adapter_type: AdapterType::LoRA,
                base_model_architecture: "llama3".to_string(),
                min_base_params: 7000,
                source_url: "https://cdn.lifesavor.dev/a".to_string(),
                version_constraint: None,
            },
            AdapterDependency {
                name: "adapter-b".to_string(),
                adapter_type: AdapterType::QLoRA,
                base_model_architecture: "mistral".to_string(),
                min_base_params: 3000,
                source_url: "https://cdn.lifesavor.dev/b".to_string(),
                version_constraint: Some(">=2.0.0".to_string()),
            },
        ];

        let section = AdapterDependenciesSection::new(deps.clone());
        let toml_str = section.to_toml().unwrap();
        let parsed = AdapterDependenciesSection::from_toml(&toml_str).unwrap();
        assert_eq!(parsed.adapter_dependencies, deps);
    }

    #[test]
    fn adapter_load_request_serialization() {
        let req = AdapterLoadRequest {
            adapter_name: "medical-pii-lora".to_string(),
            base_model: "llama3-8b".to_string(),
            timeout: Some(Duration::from_millis(5000)),
            force: true,
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["adapter_name"], "medical-pii-lora");
        assert_eq!(json["base_model"], "llama3-8b");
        assert_eq!(json["timeout"], 5000);
        assert_eq!(json["force"], true);

        let back: AdapterLoadRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back.adapter_name, req.adapter_name);
        assert_eq!(back.timeout, req.timeout);
        assert_eq!(back.force, req.force);
    }

    #[test]
    fn adapter_load_request_no_timeout() {
        let req = AdapterLoadRequest {
            adapter_name: "test".to_string(),
            base_model: "model".to_string(),
            timeout: None,
            force: false,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("timeout"));

        let back: AdapterLoadRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.timeout, None);
        assert!(!back.force);
    }

    #[test]
    fn adapter_load_result_variants() {
        let applied = AdapterLoadResult::AdapterApplied {
            adapter_name: "a".to_string(),
            base_model: "m".to_string(),
        };
        let json = serde_json::to_value(&applied).unwrap();
        assert_eq!(json["status"], "adapter_applied");

        let warm_only = AdapterLoadResult::AdapterWarmOnly {
            adapter_name: "a".to_string(),
            current_hot: "b".to_string(),
        };
        let json = serde_json::to_value(&warm_only).unwrap();
        assert_eq!(json["status"], "adapter_warm_only");

        let queued = AdapterLoadResult::Queued {
            position: 3,
            queue_depth: 5,
        };
        let json = serde_json::to_value(&queued).unwrap();
        assert_eq!(json["status"], "queued");
        assert_eq!(json["position"], 3);
    }

    #[test]
    fn adapter_load_error_display() {
        let err = AdapterLoadError::BaseModelNotLoaded {
            model: "llama3-8b".to_string(),
        };
        assert!(err.to_string().contains("llama3-8b"));
        assert!(err.to_string().contains("not loaded"));

        let err = AdapterLoadError::ChecksumMismatch {
            file: "weights.bin".to_string(),
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        assert!(err.to_string().contains("weights.bin"));
        assert!(err.to_string().contains("abc123"));
        assert!(err.to_string().contains("def456"));
    }

    #[test]
    fn adapter_dependencies_section_empty() {
        let section = AdapterDependenciesSection::new(vec![]);
        let toml_str = section.to_toml().unwrap();
        // Empty vec should produce minimal output
        let parsed = AdapterDependenciesSection::from_toml(&toml_str).unwrap();
        assert!(parsed.adapter_dependencies.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Runtime API tests (require runtime-api feature)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[cfg(feature = "runtime-api")]
mod runtime_api_tests {
    use super::*;

    #[test]
    fn map_base_model_not_loaded_error() {
        let err = map_host_error_to_adapter_error(
            "BASE_MODEL_NOT_LOADED",
            r#"{"model": "llama3-8b"}"#,
            "my-adapter",
            "llama3-8b",
        );
        assert!(matches!(err, AdapterLoadError::BaseModelNotLoaded { model } if model == "llama3-8b"));
    }

    #[test]
    fn map_adapter_not_found_error() {
        let err = map_host_error_to_adapter_error(
            "ADAPTER_NOT_FOUND",
            r#"{"adapter": "pii-lora", "model": "llama3-8b"}"#,
            "pii-lora",
            "llama3-8b",
        );
        assert!(
            matches!(err, AdapterLoadError::AdapterNotFound { adapter, model } if adapter == "pii-lora" && model == "llama3-8b")
        );
    }

    #[test]
    fn map_incompatible_architecture_error() {
        let err = map_host_error_to_adapter_error(
            "INCOMPATIBLE_ARCHITECTURE",
            r#"{"expected": "llama3", "actual": "mistral"}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::IncompatibleArchitecture { expected, actual } if expected == "llama3" && actual == "mistral")
        );
    }

    #[test]
    fn map_insufficient_base_model_error() {
        let err = map_host_error_to_adapter_error(
            "INSUFFICIENT_BASE_MODEL",
            r#"{"required": 7000, "available": 3000}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::InsufficientBaseModel { required, available } if required == 7000 && available == 3000)
        );
    }

    #[test]
    fn map_checksum_mismatch_error() {
        let err = map_host_error_to_adapter_error(
            "CHECKSUM_MISMATCH",
            r#"{"file": "weights.bin", "expected": "abc123", "actual": "def456"}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::ChecksumMismatch { file, expected, actual } if file == "weights.bin" && expected == "abc123" && actual == "def456")
        );
    }

    #[test]
    fn map_adapter_queue_full_error() {
        let err = map_host_error_to_adapter_error(
            "ADAPTER_QUEUE_FULL",
            r#"{"depth": 8, "max": 8}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::AdapterQueueFull { depth, max } if depth == 8 && max == 8)
        );
    }

    #[test]
    fn map_queue_timeout_error() {
        let err = map_host_error_to_adapter_error(
            "QUEUE_TIMEOUT",
            r#"{"elapsed_ms": 5000}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::QueueTimeout { elapsed_ms } if elapsed_ms == 5000)
        );
    }

    #[test]
    fn map_target_module_not_found_error() {
        let err = map_host_error_to_adapter_error(
            "TARGET_MODULE_NOT_FOUND",
            r#"{"module": "q_proj"}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::TargetModuleNotFound { module } if module == "q_proj")
        );
    }

    #[test]
    fn map_adapter_disk_limit_exceeded_error() {
        let err = map_host_error_to_adapter_error(
            "ADAPTER_DISK_LIMIT_EXCEEDED",
            r#"{"used_gb": 9.5, "max_gb": 10.0}"#,
            "my-adapter",
            "model",
        );
        match err {
            AdapterLoadError::AdapterDiskLimitExceeded { used_gb, max_gb } => {
                assert!((used_gb - 9.5).abs() < f64::EPSILON);
                assert!((max_gb - 10.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected AdapterDiskLimitExceeded"),
        }
    }

    #[test]
    fn map_adapter_limit_exceeded_error() {
        let err = map_host_error_to_adapter_error(
            "ADAPTER_LIMIT_EXCEEDED",
            r#"{"model": "llama3-8b", "count": 16, "max": 16}"#,
            "my-adapter",
            "llama3-8b",
        );
        assert!(
            matches!(err, AdapterLoadError::AdapterLimitExceeded { model, count, max } if model == "llama3-8b" && count == 16 && max == 16)
        );
    }

    #[test]
    fn map_no_capable_node_error() {
        let err = map_host_error_to_adapter_error(
            "NO_CAPABLE_NODE",
            "no nodes available",
            "my-adapter",
            "model",
        );
        assert!(matches!(err, AdapterLoadError::NoCapableNode));
    }

    #[test]
    fn map_delegation_timeout_error() {
        let err = map_host_error_to_adapter_error(
            "DELEGATION_TIMEOUT",
            r#"{"timeout_ms": 30000}"#,
            "my-adapter",
            "model",
        );
        assert!(
            matches!(err, AdapterLoadError::DelegationTimeout { timeout_ms } if timeout_ms == 30000)
        );
    }

    #[test]
    fn map_unknown_error_code_fallback() {
        let err = map_host_error_to_adapter_error(
            "SOME_FUTURE_ERROR",
            "unexpected thing happened",
            "my-adapter",
            "model",
        );
        assert!(matches!(err, AdapterLoadError::AdapterNotFound { .. }));
    }

    #[test]
    fn map_error_with_non_json_message_uses_defaults() {
        let err = map_host_error_to_adapter_error(
            "BASE_MODEL_NOT_LOADED",
            "plain text error message",
            "my-adapter",
            "llama3-8b",
        );
        // Should fall back to the base_model parameter
        assert!(
            matches!(err, AdapterLoadError::BaseModelNotLoaded { model } if model == "llama3-8b")
        );
    }
}

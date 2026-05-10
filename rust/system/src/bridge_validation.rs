//! Bridge request/response validation helpers.
//!
//! Provides utilities for validating [`BridgeRequest`] payloads before
//! dispatching to operation handlers, and for constructing well-formed
//! [`BridgeResponse`] values.

use crate::{BridgeRequest, BridgeResponse, BridgeError};
use serde_json::Value;

/// Validation error returned when a bridge request fails checks.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BridgeValidationError {
    /// The `operation` field is missing or empty.
    #[error("missing required field: operation")]
    MissingOperation,

    /// The `component` field is missing or empty.
    #[error("missing required field: component")]
    MissingComponent,

    /// The `correlation_id` field is missing or empty.
    #[error("missing required field: correlation_id")]
    MissingCorrelationId,

    /// A required parameter is missing from `params`.
    #[error("missing required parameter: {0}")]
    MissingParam(String),

    /// A parameter has an invalid type.
    #[error("invalid type for parameter '{name}': expected {expected}, got {actual}")]
    InvalidParamType {
        name: String,
        expected: String,
        actual: String,
    },
}

/// Validate that a bridge request has all required fields populated.
pub fn validate_request(request: &BridgeRequest) -> Result<(), BridgeValidationError> {
    if request.component.is_empty() {
        return Err(BridgeValidationError::MissingComponent);
    }
    if request.operation.is_empty() {
        return Err(BridgeValidationError::MissingOperation);
    }
    if request.correlation_id.as_ref().map_or(true, |id| id.is_empty()) {
        return Err(BridgeValidationError::MissingCorrelationId);
    }
    Ok(())
}

/// Validate that specific parameters exist in the request params.
pub fn require_params(request: &BridgeRequest, required: &[&str]) -> Result<(), BridgeValidationError> {
    for &param in required {
        match &request.params {
            Value::Object(map) => {
                if !map.contains_key(param) {
                    return Err(BridgeValidationError::MissingParam(param.to_string()));
                }
            }
            _ => return Err(BridgeValidationError::MissingParam(param.to_string())),
        }
    }
    Ok(())
}

/// Construct a successful bridge response.
pub fn success_response(result: Value) -> BridgeResponse {
    BridgeResponse::ok(result)
}

/// Construct an error bridge response.
pub fn error_response(code: &str, message: &str) -> BridgeResponse {
    BridgeResponse::err(code, message)
}

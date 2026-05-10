//! Standard bridge operation definitions (status, health, config).
//!
//! Every system component supports a set of standard operations that the agent
//! invokes for lifecycle management. This module defines constants and helpers
//! for handling these built-in operations.

use crate::{BridgeRequest, BridgeResponse, ComponentHealthStatus};
use serde_json::json;

/// Standard operation: return component status and metadata.
pub const OP_STATUS: &str = "status";

/// Standard operation: perform a health check.
pub const OP_HEALTH_CHECK: &str = "health_check";

/// Standard operation: return current configuration.
pub const OP_GET_CONFIG: &str = "get_config";

/// Standard operation: update configuration at runtime.
pub const OP_SET_CONFIG: &str = "set_config";

/// Standard operation: return component version and capabilities.
pub const OP_INFO: &str = "info";

/// Standard operation: graceful shutdown.
pub const OP_SHUTDOWN: &str = "shutdown";

/// All standard operations that every system component should handle.
pub const STANDARD_OPS: &[&str] = &[
    OP_STATUS,
    OP_HEALTH_CHECK,
    OP_GET_CONFIG,
    OP_SET_CONFIG,
    OP_INFO,
    OP_SHUTDOWN,
];

/// Check if an operation name is a standard built-in operation.
pub fn is_standard_op(operation: &str) -> bool {
    STANDARD_OPS.contains(&operation)
}

/// Generate a standard status response.
pub fn status_response(
    component_name: &str,
    version: &str,
    health: ComponentHealthStatus,
    uptime_seconds: u64,
) -> BridgeResponse {
    BridgeResponse::ok(json!({
        "component": component_name,
        "version": version,
        "health": format!("{:?}", health),
        "uptime_seconds": uptime_seconds,
    }))
}

/// Generate a standard health check response.
pub fn health_check_response(status: ComponentHealthStatus) -> BridgeResponse {
    let (success, health_str) = match &status {
        ComponentHealthStatus::Healthy => (true, "healthy"),
        ComponentHealthStatus::Degraded { .. } => (true, "degraded"),
        ComponentHealthStatus::Unhealthy { .. } => (false, "unhealthy"),
        ComponentHealthStatus::Unknown => (false, "unknown"),
    };

    if success {
        BridgeResponse::ok(json!({ "status": health_str }))
    } else {
        BridgeResponse::err("HEALTH_CHECK_FAILED", health_str)
    }
}

/// Generate a standard info response.
pub fn info_response(
    component_name: &str,
    version: &str,
    operations: &[&str],
) -> BridgeResponse {
    BridgeResponse::ok(json!({
        "name": component_name,
        "version": version,
        "operations": operations,
    }))
}

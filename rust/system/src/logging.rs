//! Structured logging helpers for system component lifecycle events.
//!
//! Provides pre-built log macros and helpers that emit structured events
//! with consistent field names for the agent's tracing pipeline.

use tracing::{info, warn, error};

/// Log a component initialization event.
pub fn log_initialized(component_name: &str, instance_id: &str) {
    info!(
        component = %component_name,
        instance_id = %instance_id,
        event = "component_initialized",
        "System component initialized"
    );
}

/// Log a component shutdown event.
pub fn log_shutdown(component_name: &str, instance_id: &str) {
    info!(
        component = %component_name,
        instance_id = %instance_id,
        event = "component_shutdown",
        "System component shut down"
    );
}

/// Log a health check result.
pub fn log_health_check(component_name: &str, instance_id: &str, healthy: bool) {
    if healthy {
        info!(
            component = %component_name,
            instance_id = %instance_id,
            event = "health_check",
            status = "healthy",
            "Health check passed"
        );
    } else {
        warn!(
            component = %component_name,
            instance_id = %instance_id,
            event = "health_check",
            status = "unhealthy",
            "Health check failed"
        );
    }
}

/// Log a bridge request received.
pub fn log_bridge_request(
    component_name: &str,
    operation: &str,
    correlation_id: &str,
    skill_id: &str,
) {
    info!(
        component = %component_name,
        operation = %operation,
        correlation_id = %correlation_id,
        skill_id = %skill_id,
        event = "bridge_request",
        "Bridge request received"
    );
}

/// Log a bridge request completed.
pub fn log_bridge_response(
    component_name: &str,
    operation: &str,
    correlation_id: &str,
    success: bool,
    duration_ms: u64,
) {
    info!(
        component = %component_name,
        operation = %operation,
        correlation_id = %correlation_id,
        success = %success,
        duration_ms = %duration_ms,
        event = "bridge_response",
        "Bridge request completed"
    );
}

/// Log a bridge request error.
pub fn log_bridge_error(
    component_name: &str,
    operation: &str,
    correlation_id: &str,
    error_code: &str,
    message: &str,
) {
    error!(
        component = %component_name,
        operation = %operation,
        correlation_id = %correlation_id,
        error_code = %error_code,
        error_message = %message,
        event = "bridge_error",
        "Bridge request failed"
    );
}

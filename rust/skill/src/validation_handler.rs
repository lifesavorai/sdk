//! Validation command handler helpers for skill setup workflows.
//!
//! When a skill defines a `validation_command` on a [`SetupStep`], the agent
//! invokes the skill binary with a [`ValidationRequest`] on stdin and expects
//! a [`ValidationResponse`] on stdout. This module provides:
//!
//! - [`validation_handler`] — wraps a developer's closure into a complete
//!   stdin → deserialize → invoke → serialize → stdout pipeline.
//! - Pre-formatted error response helpers for common failure modes:
//!   [`validation_error_invalid_credentials`],
//!   [`validation_error_connection_failed`], and
//!   [`validation_error_timeout`].
//!
//! # Example
//!
//! ```rust,ignore
//! use lifesavor_skill_sdk::validation_handler::validation_handler;
//! use lifesavor_agent_types::skill_config::{ValidationRequest, ValidationResponse};
//!
//! fn main() {
//!     validation_handler(|req: ValidationRequest| {
//!         // Custom validation logic here
//!         ValidationResponse {
//!             status: "success".to_string(),
//!             message: Some("Credentials verified".to_string()),
//!             data: None,
//!         }
//!     }).expect("validation handler failed");
//! }
//! ```

use std::io::{self, Read};

use lifesavor_agent_types::skill_config::{ValidationRequest, ValidationResponse};

/// Wraps a developer's validation closure into a stdin/stdout JSON-RPC handler.
///
/// Reads a JSON [`ValidationRequest`] from stdin, calls the provided handler
/// closure, serializes the returned [`ValidationResponse`] to stdout, and
/// flushes the output.
///
/// # Errors
///
/// Returns an error if stdin cannot be read, the input is not valid JSON, or
/// stdout cannot be written to.
pub fn validation_handler<F>(handler: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(ValidationRequest) -> ValidationResponse,
{
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let request: ValidationRequest = serde_json::from_str(&input)?;
    let response = handler(request);

    let output = serde_json::to_string(&response)?;
    println!("{}", output);

    Ok(())
}

/// Pre-formatted error response for invalid credentials.
///
/// Returns a failure [`ValidationResponse`] with a message indicating that
/// the provided credentials are invalid.
///
/// # Example
///
/// ```rust,ignore
/// use lifesavor_skill_sdk::validation_handler::validation_error_invalid_credentials;
///
/// let resp = validation_error_invalid_credentials("API key is expired");
/// assert_eq!(resp.status, "failure");
/// ```
pub fn validation_error_invalid_credentials(msg: &str) -> ValidationResponse {
    ValidationResponse {
        status: "failure".to_string(),
        message: Some(format!("Invalid credentials: {}", msg)),
        data: None,
    }
}

/// Pre-formatted error response for connection failure.
///
/// Returns a failure [`ValidationResponse`] with a message indicating that
/// the connection to the external service could not be established.
pub fn validation_error_connection_failed(msg: &str) -> ValidationResponse {
    ValidationResponse {
        status: "failure".to_string(),
        message: Some(format!("Connection failed: {}", msg)),
        data: None,
    }
}

/// Pre-formatted error response for timeout.
///
/// Returns a failure [`ValidationResponse`] with a message indicating that
/// the validation operation timed out.
pub fn validation_error_timeout(msg: &str) -> ValidationResponse {
    ValidationResponse {
        status: "failure".to_string(),
        message: Some(format!("Validation timed out: {}", msg)),
        data: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_invalid_credentials_returns_failure() {
        let resp = validation_error_invalid_credentials("bad key");
        assert_eq!(resp.status, "failure");
        assert!(resp.message.as_ref().unwrap().contains("Invalid credentials"));
        assert!(resp.message.as_ref().unwrap().contains("bad key"));
        assert!(resp.data.is_none());
    }

    #[test]
    fn error_connection_failed_returns_failure() {
        let resp = validation_error_connection_failed("host unreachable");
        assert_eq!(resp.status, "failure");
        assert!(resp.message.as_ref().unwrap().contains("Connection failed"));
        assert!(resp.message.as_ref().unwrap().contains("host unreachable"));
        assert!(resp.data.is_none());
    }

    #[test]
    fn error_timeout_returns_failure() {
        let resp = validation_error_timeout("30s exceeded");
        assert_eq!(resp.status, "failure");
        assert!(resp.message.as_ref().unwrap().contains("timed out"));
        assert!(resp.message.as_ref().unwrap().contains("30s exceeded"));
        assert!(resp.data.is_none());
    }

    #[test]
    fn error_responses_serialize_to_valid_json() {
        let responses = vec![
            validation_error_invalid_credentials("test"),
            validation_error_connection_failed("test"),
            validation_error_timeout("test"),
        ];

        for resp in responses {
            let json = serde_json::to_string(&resp).unwrap();
            let deserialized: ValidationResponse = serde_json::from_str(&json).unwrap();
            assert_eq!(resp, deserialized);
        }
    }
}

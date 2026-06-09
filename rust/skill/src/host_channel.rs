//! Host communication channel for skill-to-agent runtime calls.
//!
//! Skills run as sandboxed child processes communicating with the host agent
//! via JSON messages over stdin/stdout. This module provides an async
//! interface for sending [`SystemCallRequest`] messages and receiving
//! [`SystemCallResponse`] messages.
//!
//! # Architecture
//!
//! The host channel is initialized once at skill startup and accessed via
//! the module-level [`send_system_call`] function. Internally it uses tokio's
//! async stdin/stdout for non-blocking I/O, with a mutex to serialize access
//! (skills are single-threaded in practice, but this is safe for async).
//!
//! # Protocol
//!
//! Each message is a single JSON object terminated by a newline (`\n`).
//! The skill writes a `SystemCallRequest` to stdout and reads a
//! `SystemCallResponse` from stdin. The host agent routes the request
//! based on the `component` and `operation` fields.

use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use crate::{BridgeError, SystemCallRequest, SystemCallResponse};

// ---------------------------------------------------------------------------
// HostChannel
// ---------------------------------------------------------------------------

/// Internal host communication channel state.
struct HostChannel {
    /// Buffered reader for stdin (responses from host).
    reader: BufReader<tokio::io::Stdin>,
    /// Writer for stdout (requests to host).
    writer: tokio::io::Stdout,
}

/// Global host channel instance, lazily initialized.
static HOST_CHANNEL: std::sync::OnceLock<Arc<Mutex<HostChannel>>> = std::sync::OnceLock::new();

/// Initialize the host channel. Call this once at skill startup.
///
/// This is safe to call multiple times — subsequent calls are no-ops.
pub fn init_host_channel() {
    HOST_CHANNEL.get_or_init(|| {
        Arc::new(Mutex::new(HostChannel {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }))
    });
}

/// Get the host channel, initializing it if necessary.
fn get_channel() -> Arc<Mutex<HostChannel>> {
    HOST_CHANNEL
        .get_or_init(|| {
            Arc::new(Mutex::new(HostChannel {
                reader: BufReader::new(tokio::io::stdin()),
                writer: tokio::io::stdout(),
            }))
        })
        .clone()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Error type for host channel communication failures.
#[derive(Debug, Clone, PartialEq)]
pub enum HostChannelError {
    /// Failed to serialize the request.
    SerializationError(String),
    /// Failed to write to stdout.
    WriteError(String),
    /// Failed to read from stdin.
    ReadError(String),
    /// Failed to deserialize the response.
    DeserializationError(String),
    /// The host returned an error response.
    HostError {
        /// Machine-readable error code.
        code: String,
        /// Human-readable error message.
        message: String,
    },
}

impl std::fmt::Display for HostChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError(msg) => write!(f, "serialization error: {}", msg),
            Self::WriteError(msg) => write!(f, "write error: {}", msg),
            Self::ReadError(msg) => write!(f, "read error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "deserialization error: {}", msg),
            Self::HostError { code, message } => {
                write!(f, "host error [{}]: {}", code, message)
            }
        }
    }
}

impl std::error::Error for HostChannelError {}

/// Send a system call to the host agent and await the response.
///
/// This constructs a [`SystemCallRequest`] with the given component, operation,
/// and params, writes it as a newline-delimited JSON message to stdout, and
/// reads the response from stdin.
///
/// # Arguments
///
/// * `component` — Target component (e.g., `"adapter"`)
/// * `operation` — Operation name (e.g., `"load"`, `"release"`)
/// * `params` — Operation parameters as a JSON value
/// * `skill_id` — Identity of the calling skill
///
/// # Returns
///
/// The result payload from the host on success, or a [`HostChannelError`]
/// on failure.
pub async fn send_system_call(
    component: &str,
    operation: &str,
    params: Value,
    _skill_id: &str,
) -> Result<Value, HostChannelError> {
    let request = SystemCallRequest {
        operation_type: "system_call".to_string(),
        component: component.to_string(),
        operation: operation.to_string(),
        params,
    };

    let channel = get_channel();
    let mut guard = channel.lock().await;

    // Serialize and write the request as a single JSON line.
    let mut request_json = serde_json::to_string(&request)
        .map_err(|e| HostChannelError::SerializationError(e.to_string()))?;
    request_json.push('\n');

    guard
        .writer
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| HostChannelError::WriteError(e.to_string()))?;
    guard
        .writer
        .flush()
        .await
        .map_err(|e| HostChannelError::WriteError(e.to_string()))?;

    // Read the response line from stdin.
    let mut response_line = String::new();
    guard
        .reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| HostChannelError::ReadError(e.to_string()))?;

    if response_line.is_empty() {
        return Err(HostChannelError::ReadError(
            "unexpected EOF from host".to_string(),
        ));
    }

    // Deserialize the response.
    let response: SystemCallResponse = serde_json::from_str(response_line.trim())
        .map_err(|e| HostChannelError::DeserializationError(e.to_string()))?;

    if response.success {
        Ok(response.result)
    } else {
        let (code, message) = match response.error {
            Some(BridgeError { code, message }) => (code, message),
            None => (
                "UNKNOWN".to_string(),
                "host returned failure with no error details".to_string(),
            ),
        };
        Err(HostChannelError::HostError { code, message })
    }
}

// ---------------------------------------------------------------------------
// Skill context (for tracking skill identity)
// ---------------------------------------------------------------------------

/// Skill execution context holding the identity of the current skill.
///
/// Set at startup by the skill runtime and used by runtime API functions
/// (like adapter loading) to identify the calling skill in system calls.
static SKILL_CONTEXT: std::sync::OnceLock<SkillContext> = std::sync::OnceLock::new();

/// Context for the currently executing skill.
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// The skill's unique identifier.
    pub skill_id: String,
}

/// Initialize the skill context. Call once at skill startup.
///
/// # Panics
///
/// Panics if called more than once (the context is immutable once set).
pub fn init_skill_context(skill_id: impl Into<String>) {
    SKILL_CONTEXT
        .set(SkillContext {
            skill_id: skill_id.into(),
        })
        .expect("skill context already initialized");
}

/// Get the current skill ID, or a default if not initialized.
pub(crate) fn current_skill_id() -> String {
    SKILL_CONTEXT
        .get()
        .map(|ctx| ctx.skill_id.clone())
        .unwrap_or_else(|| "unknown-skill".to_string())
}

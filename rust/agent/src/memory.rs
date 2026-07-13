//! Memory bridge operations and types for the Life Savor SDK.
//!
//! This module defines the bridge protocol operations and request/response types
//! that SDK consumers (skills, assistants, system components) use to interact with
//! the agent's local Soul Memory Store through the bridge protocol.
//!
//! Operations follow the pattern `memory.<action>` and are dispatched by the agent's
//! bridge handler to the local `SoulMemoryStore`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Bridge Operation Constants
// ---------------------------------------------------------------------------

/// Memory bridge operation names.
///
/// These are the operation strings used in `BridgeRequest.operation` when
/// `BridgeRequest.component` is `"memory"`.
pub mod operations {
    /// Create a new memory record.
    pub const CREATE: &str = "memory.create";
    /// Read (get) a memory record by ID.
    pub const READ: &str = "memory.read";
    /// Update an existing memory record.
    pub const UPDATE: &str = "memory.update";
    /// Delete a memory record by ID.
    pub const DELETE: &str = "memory.delete";
    /// Propose a memory (creates a MemoryProposal for user approval).
    pub const PROPOSE: &str = "memory.propose";
    /// Perform semantic search over memory records.
    pub const SEARCH: &str = "memory.search";
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Classification of a memory record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Fact,
    Preference,
    Profile,
    Workflow,
    Reference,
}

/// Format of the memory record's value content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentFormat {
    Text,
    Json,
    Html,
}

/// Visibility scope of a memory record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeType {
    Global,
    Assistant,
}

/// Lifecycle status of a memory record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    Active,
    Deprecated,
}

/// Verbosity level for assistant communication style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    Minimal,
    Concise,
    Normal,
    Detailed,
    Verbose,
}

/// Formality level for assistant communication style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Formality {
    Casual,
    Neutral,
    Formal,
}

// ---------------------------------------------------------------------------
// Core Types (SDK-facing definitions)
// ---------------------------------------------------------------------------

/// Provenance metadata describing the origin of a memory record or change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Origin type: "user_explicit", "assistant_action", "seed", "proposal_approved".
    pub source_type: String,
    /// Reference to the source (e.g., file path for seeds, proposal ID for approvals).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    /// Identifier of the entity that created/modified the record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Human-readable reason for the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A stored memory record with full metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub key: String,
    pub value: String,
    pub memory_type: MemoryType,
    pub content_format: ContentFormat,
    pub scope_type: ScopeType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
    pub confidence: f64,
    pub status: MemoryStatus,
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_value: Option<String>,
    pub version_number: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A proposed memory record awaiting user approval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryProposal {
    pub id: String,
    pub key: String,
    pub value: String,
    pub memory_type: MemoryType,
    pub content_format: ContentFormat,
    pub scope_type: ScopeType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
    pub confidence: f64,
    pub status: String,
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_with_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_value: Option<String>,
    pub created_at: String,
}

/// Scope definition for a memory seed entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedScope {
    /// Scope type: global or assistant.
    pub scope_type: ScopeType,
    /// Optional scope identifier (e.g., assistant_id). Required when scope_type is "assistant".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
}

/// A memory seed entry defining an initial memory to pre-load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySeed {
    /// Key identifier (max 256 chars).
    pub key: String,
    /// Value content.
    pub value: String,
    /// Memory type classification.
    pub memory_type: MemoryType,
    /// Scope definition.
    pub scope: SeedScope,
    /// Confidence score in the range [0.0, 1.0]. Defaults to 1.0.
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Content format of the value. Defaults to "text".
    #[serde(default = "default_content_format")]
    pub content_format: ContentFormat,
}

/// A single behavioral trait with a strength weight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaTrait {
    /// Unique key identifier (max 64 chars, unique within the traits list).
    pub key: String,
    /// Strength weight in the range [0.0, 1.0].
    pub strength: f64,
    /// Optional human-readable description (max 300 chars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Communication style preferences for the assistant.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Desired tone (max 50 chars), e.g. "warm", "professional".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    /// Verbosity level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
    /// Formality level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formality: Option<Formality>,
    /// BCP-47 language tags (max 10 entries).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub language_preferences: Vec<String>,
}

/// Top-level persona definition parsed from persona.toml.
///
/// Defines an assistant's identity, behavioral traits, communication style,
/// constraints, and directives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaDefinition {
    /// Required identity string (max 200 chars).
    pub identity: String,
    /// Required purpose string (max 500 chars).
    pub purpose: String,
    /// Optional list of behavioral traits (max 50 entries).
    #[serde(default)]
    pub traits: Vec<PersonaTrait>,
    /// Optional communication style preferences.
    #[serde(default)]
    pub communication_style: CommunicationStyle,
    /// Optional constraints the assistant must never violate (max 50, each max 500 chars).
    #[serde(default)]
    pub constraints: Vec<String>,
    /// Optional behavioral directives (max 50, each max 500 chars).
    #[serde(default)]
    pub directives: Vec<String>,
}

// ---------------------------------------------------------------------------
// Bridge Request Types
// ---------------------------------------------------------------------------

/// Request payload for `memory.create` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryCreateRequest {
    pub key: String,
    pub value: String,
    pub memory_type: MemoryType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_format: Option<ContentFormat>,
    pub scope_type: ScopeType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
}

/// Request payload for `memory.read` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryReadRequest {
    /// The memory record ID to retrieve.
    pub id: String,
}

/// Request payload for `memory.update` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryUpdateRequest {
    /// The memory record ID to update.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
}

/// Request payload for `memory.delete` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryDeleteRequest {
    /// The memory record ID to delete.
    pub id: String,
}

/// Request payload for `memory.propose` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryProposeRequest {
    pub key: String,
    pub value: String,
    pub memory_type: MemoryType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_format: Option<ContentFormat>,
    pub scope_type: ScopeType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// The assistant proposing the memory.
    pub assistant_id: String,
    /// Reason for the proposal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Request payload for `memory.search` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    /// The search query string (max 1000 characters).
    pub query: String,
    /// Maximum number of results to return (default: 20).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Optional scope filter to bias results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<ScopeType>,
    /// Optional scope ID to filter by.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Bridge Response Types
// ---------------------------------------------------------------------------

/// Response payload for `memory.create` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryCreateResponse {
    pub record: MemoryRecord,
}

/// Response payload for `memory.read` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryReadResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<MemoryRecord>,
}

/// Response payload for `memory.update` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryUpdateResponse {
    pub record: MemoryRecord,
}

/// Response payload for `memory.delete` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryDeleteResponse {
    /// Whether the deletion succeeded.
    pub success: bool,
}

/// Response payload for `memory.propose` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryProposeResponse {
    pub proposal: MemoryProposal,
}

/// A memory record paired with a semantic relevance score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub record: MemoryRecord,
    pub relevance_score: f64,
}

/// Response payload for `memory.search` bridge operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    pub results: Vec<ScoredMemory>,
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

fn default_confidence() -> f64 {
    1.0
}

fn default_content_format() -> ContentFormat {
    ContentFormat::Text
}

// ---------------------------------------------------------------------------
// Helper: Convert request types to BridgeRequest params
// ---------------------------------------------------------------------------

impl MemoryCreateRequest {
    /// Serialize this request into a JSON `Value` suitable for `BridgeRequest.params`.
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl MemoryReadRequest {
    /// Serialize this request into a JSON `Value` suitable for `BridgeRequest.params`.
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl MemoryUpdateRequest {
    /// Serialize this request into a JSON `Value` suitable for `BridgeRequest.params`.
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl MemoryDeleteRequest {
    /// Serialize this request into a JSON `Value` suitable for `BridgeRequest.params`.
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl MemoryProposeRequest {
    /// Serialize this request into a JSON `Value` suitable for `BridgeRequest.params`.
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl MemorySearchRequest {
    /// Serialize this request into a JSON `Value` suitable for `BridgeRequest.params`.
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_constants_are_correct() {
        assert_eq!(operations::CREATE, "memory.create");
        assert_eq!(operations::READ, "memory.read");
        assert_eq!(operations::UPDATE, "memory.update");
        assert_eq!(operations::DELETE, "memory.delete");
        assert_eq!(operations::PROPOSE, "memory.propose");
        assert_eq!(operations::SEARCH, "memory.search");
    }

    #[test]
    fn memory_create_request_serde_round_trip() {
        let req = MemoryCreateRequest {
            key: "user_name".into(),
            value: "Alice".into(),
            memory_type: MemoryType::Fact,
            content_format: Some(ContentFormat::Text),
            scope_type: ScopeType::Global,
            scope_id: None,
            confidence: Some(0.95),
            source_type: Some("user_explicit".into()),
            source_ref: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemoryCreateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn memory_read_request_serde_round_trip() {
        let req = MemoryReadRequest {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemoryReadRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn memory_update_request_serde_round_trip() {
        let req = MemoryUpdateRequest {
            id: "some-uuid".into(),
            value: Some("new value".into()),
            confidence: Some(0.8),
            source_type: Some("user_explicit".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemoryUpdateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn memory_delete_request_serde_round_trip() {
        let req = MemoryDeleteRequest {
            id: "some-uuid".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemoryDeleteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn memory_propose_request_serde_round_trip() {
        let req = MemoryProposeRequest {
            key: "learned_preference".into(),
            value: "prefers dark mode".into(),
            memory_type: MemoryType::Preference,
            content_format: None,
            scope_type: ScopeType::Assistant,
            scope_id: Some("assistant_001".into()),
            confidence: Some(0.7),
            assistant_id: "assistant_001".into(),
            reason: Some("Observed user toggling dark mode".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemoryProposeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn memory_search_request_serde_round_trip() {
        let req = MemorySearchRequest {
            query: "what does the user prefer".into(),
            limit: Some(10),
            scope_type: Some(ScopeType::Assistant),
            scope_id: Some("assistant_001".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemorySearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn memory_record_serde_round_trip() {
        let record = MemoryRecord {
            id: "uuid-1".into(),
            key: "user_name".into(),
            value: "Alice".into(),
            memory_type: MemoryType::Fact,
            content_format: ContentFormat::Text,
            scope_type: ScopeType::Global,
            scope_id: None,
            confidence: 1.0,
            status: MemoryStatus::Active,
            provenance: Provenance {
                source_type: "user_explicit".into(),
                source_ref: None,
                created_by: Some("user".into()),
                reason: None,
            },
            conflict_notes: None,
            previous_value: None,
            version_number: 1,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&record).unwrap();
        let back: MemoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn memory_proposal_serde_round_trip() {
        let proposal = MemoryProposal {
            id: "prop-1".into(),
            key: "dark_mode".into(),
            value: "enabled".into(),
            memory_type: MemoryType::Preference,
            content_format: ContentFormat::Text,
            scope_type: ScopeType::Assistant,
            scope_id: Some("asst-1".into()),
            confidence: 0.7,
            status: "pending".into(),
            provenance: Provenance {
                source_type: "assistant_action".into(),
                source_ref: None,
                created_by: Some("asst-1".into()),
                reason: Some("User toggled dark mode".into()),
            },
            conflict_with_id: None,
            conflict_value: None,
            created_at: "2025-01-02T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&proposal).unwrap();
        let back: MemoryProposal = serde_json::from_str(&json).unwrap();
        assert_eq!(back, proposal);
    }

    #[test]
    fn memory_seed_serde_round_trip() {
        let seed = MemorySeed {
            key: "user_name".into(),
            value: "Alice".into(),
            memory_type: MemoryType::Fact,
            scope: SeedScope {
                scope_type: ScopeType::Global,
                scope_id: None,
            },
            confidence: 0.95,
            content_format: ContentFormat::Text,
        };
        let json = serde_json::to_string(&seed).unwrap();
        let back: MemorySeed = serde_json::from_str(&json).unwrap();
        assert_eq!(back, seed);
    }

    #[test]
    fn persona_definition_serde_round_trip() {
        let persona = PersonaDefinition {
            identity: "Aria".into(),
            purpose: "Creative writing assistant".into(),
            traits: vec![PersonaTrait {
                key: "empathy".into(),
                strength: 0.9,
                description: Some("Deeply empathetic".into()),
            }],
            communication_style: CommunicationStyle {
                tone: Some("warm".into()),
                verbosity: Some(Verbosity::Detailed),
                formality: Some(Formality::Casual),
                language_preferences: vec!["en-US".into()],
            },
            constraints: vec!["Never generate harmful content".into()],
            directives: vec!["Prioritize creativity".into()],
        };
        let json = serde_json::to_string(&persona).unwrap();
        let back: PersonaDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(back, persona);
    }

    #[test]
    fn scored_memory_serde_round_trip() {
        let scored = ScoredMemory {
            record: MemoryRecord {
                id: "uuid-1".into(),
                key: "test".into(),
                value: "value".into(),
                memory_type: MemoryType::Fact,
                content_format: ContentFormat::Text,
                scope_type: ScopeType::Global,
                scope_id: None,
                confidence: 1.0,
                status: MemoryStatus::Active,
                provenance: Provenance {
                    source_type: "user_explicit".into(),
                    source_ref: None,
                    created_by: None,
                    reason: None,
                },
                conflict_notes: None,
                previous_value: None,
                version_number: 1,
                created_at: "2025-01-01T00:00:00Z".into(),
                updated_at: "2025-01-01T00:00:00Z".into(),
            },
            relevance_score: 0.87,
        };
        let json = serde_json::to_string(&scored).unwrap();
        let back: ScoredMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(back, scored);
    }

    #[test]
    fn create_response_serde_round_trip() {
        let resp = MemoryCreateResponse {
            record: MemoryRecord {
                id: "uuid-1".into(),
                key: "k".into(),
                value: "v".into(),
                memory_type: MemoryType::Fact,
                content_format: ContentFormat::Text,
                scope_type: ScopeType::Global,
                scope_id: None,
                confidence: 1.0,
                status: MemoryStatus::Active,
                provenance: Provenance {
                    source_type: "user_explicit".into(),
                    source_ref: None,
                    created_by: None,
                    reason: None,
                },
                conflict_notes: None,
                previous_value: None,
                version_number: 1,
                created_at: "2025-01-01T00:00:00Z".into(),
                updated_at: "2025-01-01T00:00:00Z".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: MemoryCreateResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn search_response_serde_round_trip() {
        let resp = MemorySearchResponse {
            results: vec![ScoredMemory {
                record: MemoryRecord {
                    id: "uuid-1".into(),
                    key: "k".into(),
                    value: "v".into(),
                    memory_type: MemoryType::Preference,
                    content_format: ContentFormat::Json,
                    scope_type: ScopeType::Assistant,
                    scope_id: Some("asst-1".into()),
                    confidence: 0.9,
                    status: MemoryStatus::Active,
                    provenance: Provenance {
                        source_type: "seed".into(),
                        source_ref: Some("memory.toml".into()),
                        created_by: None,
                        reason: None,
                    },
                    conflict_notes: None,
                    previous_value: None,
                    version_number: 1,
                    created_at: "2025-01-01T00:00:00Z".into(),
                    updated_at: "2025-01-01T00:00:00Z".into(),
                },
                relevance_score: 0.92,
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: MemorySearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn to_params_produces_valid_json() {
        let req = MemoryCreateRequest {
            key: "test".into(),
            value: "value".into(),
            memory_type: MemoryType::Fact,
            content_format: None,
            scope_type: ScopeType::Global,
            scope_id: None,
            confidence: None,
            source_type: None,
            source_ref: None,
        };
        let params = req.to_params();
        assert!(params.is_object());
        assert_eq!(params["key"], "test");
        assert_eq!(params["value"], "value");
        assert_eq!(params["memory_type"], "fact");
        assert_eq!(params["scope_type"], "global");
    }

    #[test]
    fn optional_fields_skipped_in_serialization() {
        let req = MemoryCreateRequest {
            key: "k".into(),
            value: "v".into(),
            memory_type: MemoryType::Fact,
            content_format: None,
            scope_type: ScopeType::Global,
            scope_id: None,
            confidence: None,
            source_type: None,
            source_ref: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("content_format"));
        assert!(!json.contains("scope_id"));
        assert!(!json.contains("confidence"));
        assert!(!json.contains("source_type"));
        assert!(!json.contains("source_ref"));
    }

    #[test]
    fn memory_seed_defaults() {
        let json = r#"{
            "key": "test",
            "value": "val",
            "memory_type": "fact",
            "scope": {"scope_type": "global"}
        }"#;
        let seed: MemorySeed = serde_json::from_str(json).unwrap();
        assert_eq!(seed.confidence, 1.0);
        assert_eq!(seed.content_format, ContentFormat::Text);
    }

    #[test]
    fn persona_definition_defaults() {
        let json = r#"{
            "identity": "Test",
            "purpose": "Testing"
        }"#;
        let persona: PersonaDefinition = serde_json::from_str(json).unwrap();
        assert!(persona.traits.is_empty());
        assert_eq!(persona.communication_style, CommunicationStyle::default());
        assert!(persona.constraints.is_empty());
        assert!(persona.directives.is_empty());
    }
}

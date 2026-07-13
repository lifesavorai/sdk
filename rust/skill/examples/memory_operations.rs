//! Soul Memory operations example.
//!
//! Demonstrates how a skill interacts with the agent's local Soul Memory Store
//! through the bridge protocol: creating, reading, updating, deleting, proposing,
//! and searching memories.
//!
//! Run with: `cargo run --example memory_operations`

use lifesavor_skill_sdk::prelude::*;
use lifesavor_skill_sdk::memory::{
    operations,
    MemoryCreateRequest, MemoryCreateResponse,
    MemoryReadRequest, MemoryReadResponse,
    MemoryUpdateRequest, MemoryUpdateResponse,
    MemoryDeleteRequest, MemoryDeleteResponse,
    MemoryProposeRequest, MemoryProposeResponse,
    MemorySearchRequest, MemorySearchResponse,
    MemoryType, ContentFormat, ScopeType,
};

const SKILL_ID: &str = "memory-demo-skill";

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Soul Memory operations example");

    // -------------------------------------------------------------------------
    // 1. Create a memory record
    // -------------------------------------------------------------------------
    let create_req = MemoryCreateRequest {
        key: "user_name".into(),
        value: "Alice".into(),
        memory_type: MemoryType::Fact,
        content_format: Some(ContentFormat::Text),
        scope_type: ScopeType::Global,
        scope_id: None,
        confidence: Some(1.0),
        source_type: Some("user_explicit".into()),
        source_ref: Some(SKILL_ID.into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::CREATE.into(),
        params: create_req.to_params(),
        skill_id: SKILL_ID.into(),
        correlation_id: Some("demo-001".into()),
    };
    info!("CREATE request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // Simulated successful response
    let create_resp = BridgeResponse::ok(serde_json::to_value(MemoryCreateResponse {
        record: lifesavor_skill_sdk::memory::MemoryRecord {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            key: "user_name".into(),
            value: "Alice".into(),
            memory_type: MemoryType::Fact,
            content_format: ContentFormat::Text,
            scope_type: ScopeType::Global,
            scope_id: None,
            confidence: 1.0,
            status: lifesavor_skill_sdk::memory::MemoryStatus::Active,
            provenance: lifesavor_skill_sdk::memory::Provenance {
                source_type: "user_explicit".into(),
                source_ref: Some(SKILL_ID.into()),
                created_by: None,
                reason: None,
            },
            conflict_notes: None,
            previous_value: None,
            version_number: 1,
            created_at: "2025-07-15T10:00:00Z".into(),
            updated_at: "2025-07-15T10:00:00Z".into(),
        },
    }).unwrap());
    info!("CREATE response success={}", create_resp.success);

    // -------------------------------------------------------------------------
    // 2. Read a memory record by ID
    // -------------------------------------------------------------------------
    let read_req = MemoryReadRequest {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::READ.into(),
        params: read_req.to_params(),
        skill_id: SKILL_ID.into(),
        correlation_id: Some("demo-002".into()),
    };
    info!("READ request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // -------------------------------------------------------------------------
    // 3. Update a memory record
    // -------------------------------------------------------------------------
    let update_req = MemoryUpdateRequest {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        value: Some("Alice Johnson".into()),
        confidence: Some(1.0),
        source_type: Some("user_explicit".into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::UPDATE.into(),
        params: update_req.to_params(),
        skill_id: SKILL_ID.into(),
        correlation_id: Some("demo-003".into()),
    };
    info!("UPDATE request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // -------------------------------------------------------------------------
    // 4. Delete a memory record
    // -------------------------------------------------------------------------
    let delete_req = MemoryDeleteRequest {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::DELETE.into(),
        params: delete_req.to_params(),
        skill_id: SKILL_ID.into(),
        correlation_id: Some("demo-004".into()),
    };
    info!("DELETE request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // -------------------------------------------------------------------------
    // 5. Propose a memory (assistant flow)
    // -------------------------------------------------------------------------
    // Note: In production, this would be called from an assistant context.
    // Assistants propose memories for user approval rather than writing directly.
    let propose_req = MemoryProposeRequest {
        key: "preferred_writing_tone".into(),
        value: "conversational and witty".into(),
        memory_type: MemoryType::Preference,
        content_format: None, // defaults to "text"
        scope_type: ScopeType::Assistant,
        scope_id: Some("writing-assistant-001".into()),
        confidence: Some(0.75),
        assistant_id: "writing-assistant-001".into(),
        reason: Some("User consistently chooses casual phrasing over formal alternatives".into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::PROPOSE.into(),
        params: propose_req.to_params(),
        skill_id: "writing-assistant-001".into(),
        correlation_id: Some("demo-005".into()),
    };
    info!("PROPOSE request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // -------------------------------------------------------------------------
    // 6. Semantic search
    // -------------------------------------------------------------------------
    let search_req = MemorySearchRequest {
        query: "what writing style does the user prefer".into(),
        limit: Some(5),
        scope_type: Some(ScopeType::Assistant),
        scope_id: Some("writing-assistant-001".into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::SEARCH.into(),
        params: search_req.to_params(),
        skill_id: SKILL_ID.into(),
        correlation_id: Some("demo-006".into()),
    };
    info!("SEARCH request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // -------------------------------------------------------------------------
    // 7. Creating a memory with JSON content format
    // -------------------------------------------------------------------------
    let json_memory = MemoryCreateRequest {
        key: "project_config".into(),
        value: serde_json::json!({
            "name": "My Novel",
            "genre": "sci-fi",
            "target_words": 80000
        }).to_string(),
        memory_type: MemoryType::Reference,
        content_format: Some(ContentFormat::Json),
        scope_type: ScopeType::Assistant,
        scope_id: Some("writing-assistant-001".into()),
        confidence: Some(1.0),
        source_type: Some("user_explicit".into()),
        source_ref: None,
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::CREATE.into(),
        params: json_memory.to_params(),
        skill_id: SKILL_ID.into(),
        correlation_id: Some("demo-007".into()),
    };
    info!("CREATE (JSON format) request:\n{}", serde_json::to_string_pretty(&bridge_req).unwrap());

    // -------------------------------------------------------------------------
    // Error handling example
    // -------------------------------------------------------------------------
    // When a memory is not found:
    let not_found_resp = BridgeResponse::err(
        "MEMORY_NOT_FOUND",
        "No memory record with ID 'nonexistent-id' exists",
    );
    info!(
        "Error response: success={}, code={:?}",
        not_found_resp.success,
        not_found_resp.error.as_ref().map(|e| &e.code)
    );

    // When a duplicate key conflicts:
    let conflict_resp = BridgeResponse::err(
        "MEMORY_CONFLICT",
        "A record with key 'user_name' already exists in scope global",
    );
    info!(
        "Conflict response: success={}, code={:?}",
        conflict_resp.success,
        conflict_resp.error.as_ref().map(|e| &e.code)
    );

    info!("Soul Memory operations example complete");
}

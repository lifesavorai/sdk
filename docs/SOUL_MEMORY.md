# Soul Memory SDK Guide

## Overview

The Soul Memory system provides agent-local, encrypted storage for user memories. All data stays on the user's device — no cloud storage is involved. Components (skills, assistants, system modules) interact with the memory store through the bridge protocol.

The memory store supports:
- **CRUD operations** on typed memory records with scope isolation
- **Semantic search** using vector embeddings for similarity-based retrieval
- **Memory proposals** — assistants can suggest memories that require user approval
- **Version history** — every change to a memory is tracked
- **Persona and seed files** — TOML-based authoring of assistant identity and initial knowledge

## Architecture

```
┌──────────────┐    Bridge Protocol    ┌─────────────────────┐
│  Skill       │ ───────────────────── │                     │
│  Component   │                       │  Agent              │
├──────────────┤                       │  ┌───────────────┐  │
│  Assistant   │ ───────────────────── │  │ SoulMemory    │  │
│  Component   │                       │  │ Store (SQLite)│  │
├──────────────┤                       │  └───────────────┘  │
│  System      │ ───────────────────── │                     │
│  Component   │                       └─────────────────────┘
└──────────────┘
```

Components send `BridgeRequest` messages with `component: "memory"` and the appropriate operation. The agent routes these to the local `SoulMemoryStore`.

## Bridge Operations

| Operation        | Description                              |
|------------------|------------------------------------------|
| `memory.create`  | Create a new memory record               |
| `memory.read`    | Retrieve a memory record by ID           |
| `memory.update`  | Update an existing memory record         |
| `memory.delete`  | Delete a memory record by ID             |
| `memory.propose` | Propose a memory for user approval       |
| `memory.search`  | Semantic search over memory records      |

## Memory Types

| Type         | Description                                |
|--------------|--------------------------------------------|
| `fact`       | Objective information (e.g., "User's name is Alice") |
| `preference` | User preferences (e.g., "Prefers dark mode")         |
| `profile`    | User profile data (e.g., "Works at Acme Corp")       |
| `workflow`   | Workflow patterns (e.g., "Always reviews PRs first")  |
| `reference`  | Reference material (e.g., API docs, project notes)   |

## Content Formats

- `text` — plain text (default)
- `json` — structured JSON (validated on write)
- `html` — HTML content (stored as-is)

## Scoping

- `global` — visible to all assistants
- `assistant` — visible only to the specified assistant (requires `scope_id`)

---

## Rust SDK Usage

### Memory CRUD from a Skill

```rust
use lifesavor_skill_sdk::prelude::*;
use lifesavor_skill_sdk::memory::{
    operations, MemoryCreateRequest, MemoryReadRequest,
    MemoryUpdateRequest, MemoryDeleteRequest,
    MemoryType, ContentFormat, ScopeType,
};

/// Create a memory record from within a skill execution context.
async fn remember_user_name(ctx: &SkillContext, name: &str) -> Result<(), SkillError> {
    let request = MemoryCreateRequest {
        key: "user_name".into(),
        value: name.into(),
        memory_type: MemoryType::Fact,
        content_format: Some(ContentFormat::Text),
        scope_type: ScopeType::Global,
        scope_id: None,
        confidence: Some(1.0),
        source_type: Some("user_explicit".into()),
        source_ref: Some("onboarding-skill".into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::CREATE.into(),
        params: request.to_params(),
        skill_id: ctx.skill_id().into(),
        correlation_id: Some(ctx.correlation_id().into()),
    };

    let response = ctx.send_bridge_request(bridge_req).await?;
    if !response.success {
        tracing::warn!("Failed to create memory: {:?}", response.error);
    }
    Ok(())
}

/// Read a memory record by ID.
async fn get_memory(ctx: &SkillContext, id: &str) -> Result<(), SkillError> {
    let request = MemoryReadRequest { id: id.into() };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::READ.into(),
        params: request.to_params(),
        skill_id: ctx.skill_id().into(),
        correlation_id: Some(ctx.correlation_id().into()),
    };

    let response = ctx.send_bridge_request(bridge_req).await?;
    if response.success {
        if let Some(data) = response.data {
            let record: MemoryRecord = serde_json::from_value(data)?;
            tracing::info!("Memory: {} = {}", record.key, record.value);
        }
    }
    Ok(())
}

/// Update an existing memory record.
async fn update_preference(ctx: &SkillContext, id: &str, new_value: &str) -> Result<(), SkillError> {
    let request = MemoryUpdateRequest {
        id: id.into(),
        value: Some(new_value.into()),
        confidence: Some(0.9),
        source_type: Some("user_explicit".into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::UPDATE.into(),
        params: request.to_params(),
        skill_id: ctx.skill_id().into(),
        correlation_id: Some(ctx.correlation_id().into()),
    };

    ctx.send_bridge_request(bridge_req).await?;
    Ok(())
}

/// Delete a memory record.
async fn forget_memory(ctx: &SkillContext, id: &str) -> Result<(), SkillError> {
    let request = MemoryDeleteRequest { id: id.into() };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::DELETE.into(),
        params: request.to_params(),
        skill_id: ctx.skill_id().into(),
        correlation_id: Some(ctx.correlation_id().into()),
    };

    ctx.send_bridge_request(bridge_req).await?;
    Ok(())
}
```

### Proposing Memories from an Assistant

Assistants cannot directly write memories — they propose them for user approval:

```rust
use lifesavor_skill_sdk::memory::{
    operations, MemoryProposeRequest, MemoryType, ScopeType,
};

/// Propose a memory based on observed user behavior.
async fn propose_learned_preference(
    ctx: &AssistantContext,
    key: &str,
    value: &str,
    reason: &str,
) -> Result<(), AssistantError> {
    let request = MemoryProposeRequest {
        key: key.into(),
        value: value.into(),
        memory_type: MemoryType::Preference,
        content_format: None, // defaults to "text"
        scope_type: ScopeType::Assistant,
        scope_id: Some(ctx.assistant_id().into()),
        confidence: Some(0.7),
        assistant_id: ctx.assistant_id().into(),
        reason: Some(reason.into()),
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::PROPOSE.into(),
        params: request.to_params(),
        skill_id: ctx.assistant_id().into(),
        correlation_id: Some(ctx.correlation_id().into()),
    };

    let response = ctx.send_bridge_request(bridge_req).await?;
    if response.success {
        tracing::info!("Proposed memory '{}' for user approval", key);
    }
    Ok(())
}
```

The user will see the proposal in their memory management UI and can approve or reject it.

### Semantic Search from a Component

```rust
use lifesavor_skill_sdk::memory::{
    operations, MemorySearchRequest, MemorySearchResponse, ScopeType,
};

/// Search memories by semantic similarity.
async fn find_relevant_memories(
    ctx: &SkillContext,
    query: &str,
) -> Result<Vec<ScoredMemory>, SkillError> {
    let request = MemorySearchRequest {
        query: query.into(),
        limit: Some(10),
        scope_type: Some(ScopeType::Global),
        scope_id: None,
    };

    let bridge_req = BridgeRequest {
        component: "memory".into(),
        operation: operations::SEARCH.into(),
        params: request.to_params(),
        skill_id: ctx.skill_id().into(),
        correlation_id: Some(ctx.correlation_id().into()),
    };

    let response = ctx.send_bridge_request(bridge_req).await?;
    if response.success {
        if let Some(data) = response.data {
            let search_resp: MemorySearchResponse = serde_json::from_value(data)?;
            return Ok(search_resp.results);
        }
    }
    Ok(vec![])
}

/// Example: use semantic search to personalize a response.
async fn personalize_greeting(ctx: &SkillContext) -> String {
    let memories = find_relevant_memories(ctx, "user name and preferences").await
        .unwrap_or_default();

    let mut greeting = "Hello!".to_string();
    for scored in &memories {
        if scored.record.key == "user_name" {
            greeting = format!("Hello, {}!", scored.record.value);
        }
    }
    greeting
}
```

---

## TypeScript/Node.js SDK Usage

### Memory CRUD from a Skill

```javascript
const { MEMORY_OPERATIONS, validateCreateRequest } = require('lifesavor-memory-sdk');

/**
 * Create a memory from a Node.js skill.
 */
async function rememberFact(bridge, skillId, key, value) {
  const request = {
    key,
    value,
    memory_type: 'fact',
    scope_type: 'global',
    confidence: 1.0,
    source_type: 'user_explicit',
    source_ref: skillId,
  };

  // Validate before sending
  const { valid, errors } = validateCreateRequest(request);
  if (!valid) {
    throw new Error(`Invalid request: ${errors.map(e => e.message).join(', ')}`);
  }

  return bridge.send({
    component: 'memory',
    operation: MEMORY_OPERATIONS.CREATE,
    params: request,
    skill_id: skillId,
  });
}

/**
 * Read a memory by ID.
 */
async function readMemory(bridge, skillId, memoryId) {
  return bridge.send({
    component: 'memory',
    operation: MEMORY_OPERATIONS.READ,
    params: { id: memoryId },
    skill_id: skillId,
  });
}

/**
 * Update an existing memory.
 */
async function updateMemory(bridge, skillId, memoryId, newValue) {
  return bridge.send({
    component: 'memory',
    operation: MEMORY_OPERATIONS.UPDATE,
    params: {
      id: memoryId,
      value: newValue,
      source_type: 'user_explicit',
    },
    skill_id: skillId,
  });
}

/**
 * Delete a memory.
 */
async function deleteMemory(bridge, skillId, memoryId) {
  return bridge.send({
    component: 'memory',
    operation: MEMORY_OPERATIONS.DELETE,
    params: { id: memoryId },
    skill_id: skillId,
  });
}
```

### Proposing Memories from an Assistant (TypeScript)

```typescript
import { MEMORY_OPERATIONS } from 'lifesavor-memory-sdk';

async function proposeMemory(
  bridge: Bridge,
  assistantId: string,
  key: string,
  value: string,
  reason: string
) {
  return bridge.send({
    component: 'memory',
    operation: MEMORY_OPERATIONS.PROPOSE,
    params: {
      key,
      value,
      memory_type: 'preference',
      scope_type: 'assistant',
      scope_id: assistantId,
      confidence: 0.7,
      assistant_id: assistantId,
      reason,
    },
    skill_id: assistantId,
  });
}
```

### Semantic Search (TypeScript)

```typescript
import { MEMORY_OPERATIONS, MemorySearchResponse } from 'lifesavor-memory-sdk';

async function searchMemories(
  bridge: Bridge,
  skillId: string,
  query: string,
  limit = 20
): Promise<MemorySearchResponse> {
  const response = await bridge.send({
    component: 'memory',
    operation: MEMORY_OPERATIONS.SEARCH,
    params: { query, limit },
    skill_id: skillId,
  });
  return response.data;
}
```

---

## Persona and Memory Seed Files

### persona.toml

Defines an assistant's identity, behavioral traits, and communication style. Referenced from the assistant definition via the `persona_file` field.

See the annotated example at `developer/sdk/examples/persona.toml`.

### memory.toml

Defines initial memories to pre-load when an assistant is first activated. Referenced via the `memory_seed_file` field.

See the annotated example at `developer/sdk/examples/memory.toml`.

### Linking to AssistantDefinition

In your assistant definition file, reference the persona and seed files:

```toml
persona_file = "persona.toml"
memory_seed_file = "memory.toml"
```

Both paths are resolved relative to the assistant definition file's directory.

---

## Error Handling

All bridge responses include a `success` boolean. On failure:

```json
{
  "success": false,
  "error": {
    "code": "MEMORY_NOT_FOUND",
    "message": "No memory record with ID '...' exists"
  }
}
```

Common error codes:

| Code                     | Description                                        |
|--------------------------|----------------------------------------------------|
| `MEMORY_NOT_FOUND`       | The referenced record ID does not exist            |
| `MEMORY_CONFLICT`        | A record with the same key+scope already exists    |
| `MEMORY_VALIDATION`      | Request failed field validation                    |
| `MEMORY_STORE_UNAVAILABLE` | The memory store failed to initialize            |

---

## Best Practices

1. **Use specific keys** — prefer `user_preferred_language` over `language` to avoid collisions.
2. **Set appropriate confidence** — use 1.0 for user-stated facts, lower values for inferred data.
3. **Scope correctly** — use `global` for facts about the user, `assistant` for assistant-specific context.
4. **Propose, don't write** — assistants should use `memory.propose` rather than `memory.create` to respect user control.
5. **Handle failures gracefully** — the memory store may be unavailable; always have a fallback path.
6. **Keep values concise** — the 100KB limit is generous, but smaller values embed and search better.

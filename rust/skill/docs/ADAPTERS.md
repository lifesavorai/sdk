# Fine-Tuned Model Adapters

This guide covers how to declare, load, and manage LoRA/QLoRA adapters in your skill. Adapters let you ship task-specific fine-tuned weights that run on top of base models already loaded by the agent — no full model replacement needed.

## Overview

The adapter system lets you:

- **Declare** which adapters your skill needs in `provider-manifest.toml`
- **Load** adapters at runtime with a single async call
- **Release** adapters when inference is complete so other skills can use the model
- **Version** adapters with SemVer constraints for safe automatic upgrades

The agent handles everything else: downloading from CDN, verifying checksums, managing Hot/Warm/Cold states, queueing conflicting requests, and even delegating to more capable devices in the user's fleet.

## Quick Example

```rust
use lifesavor_skill_sdk::adapter::{
    AdapterDependency, AdapterType, AdapterLoadRequest, 
    request_adapter_load, release_adapter,
};
use std::time::Duration;

// At runtime — request the adapter
let result = request_adapter_load(AdapterLoadRequest {
    adapter_name: "medical-pii-lora".to_string(),
    base_model: "llama3-8b".to_string(),
    timeout: Some(Duration::from_secs(30)),
    force: false,
}).await?;

// ... run inference with the adapter-enhanced model ...

// Release when done
release_adapter("medical-pii-lora", "llama3-8b").await?;
```

## Step 1: Declare Adapter Dependencies

In your skill's `provider-manifest.toml`, declare which adapters your skill uses:

```toml
[[adapter_dependencies]]
name = "medical-pii-lora"
adapter_type = "lora"
base_model_architecture = "llama3"
min_base_params = 7000
source_url = "https://cdn.lifesavor.dev/adapters/medical-pii-lora/1.2.0/"
version_constraint = ">=1.0.0, <2.0.0"
```

Or use the builder in Rust:

```rust
use lifesavor_skill_sdk::adapter::{AdapterDependency, AdapterType};
use lifesavor_skill_sdk::builder::SkillProviderBuilder;

let provider = SkillProviderBuilder::new(manifest)?
    .adapter_dependency(AdapterDependency {
        name: "medical-pii-lora".to_string(),
        adapter_type: AdapterType::LoRA,
        base_model_architecture: "llama3".to_string(),
        min_base_params: 7000,
        source_url: "https://cdn.lifesavor.dev/adapters/medical-pii-lora/1.2.0/".to_string(),
        version_constraint: Some(">=1.0.0, <2.0.0".to_string()),
    })
    .tool(my_tool)
    .build()?;
```

### Field Reference

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Adapter identifier (e.g., `"medical-pii-lora"`) |
| `adapter_type` | Yes | `"lora"` or `"qlora"` |
| `base_model_architecture` | Yes | Target model architecture (e.g., `"llama3"`, `"mistral"`) |
| `min_base_params` | Yes | Minimum model size in millions (e.g., `7000` for 7B) |
| `source_url` | Yes | CDN URL where the adapter artifact lives |
| `version_constraint` | No | SemVer range (e.g., `">=1.0.0, <2.0.0"`) |

## Step 2: Create Your Adapter Artifact

Your adapter artifact is a directory containing:

```
medical-pii-lora/
├── adapter.toml          # Manifest (required)
├── adapter_model.safetensors  # Weight files
└── adapter_config.json   # Optional config
```

### adapter.toml Format

```toml
name = "medical-pii-lora"
version = "1.2.0"
adapter_type = "lora"
base_model_architecture = "llama3"
min_base_params = 7000
rank = 16
total_size_bytes = 52428800

# Optional
description = "Fine-tuned adapter for medical PII detection"
author = "YourName"
license = "Apache-2.0"
target_modules = ["q_proj", "v_proj", "k_proj", "o_proj"]

[[files]]
path = "adapter_model.safetensors"
sha256 = "a1b2c3d4e5f6..."

[[files]]
path = "adapter_config.json"
sha256 = "f6e5d4c3b2a1..."
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Adapter name (matches your dependency declaration) |
| `version` | string | SemVer version (e.g., `"1.2.0"`) |
| `adapter_type` | enum | `"lora"` or `"qlora"` |
| `base_model_architecture` | string | Compatible architecture |
| `min_base_params` | integer | Minimum base model size (millions) |
| `rank` | integer | LoRA rank (dimensionality) |
| `files` | array | File entries with `path` and `sha256` |
| `total_size_bytes` | integer | Total size of all adapter files |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | string | Human-readable description |
| `author` | string | Author name or organization |
| `license` | string | License identifier (e.g., `"Apache-2.0"`) |
| `quantization_bits` | integer | Required for QLoRA (2, 4, or 8) |
| `target_modules` | array | Model layers modified by this adapter |

## Step 3: Upload Your Adapter

### Using the MCP Tools (in your IDE)

```
> validate_adapter_manifest   # Check your adapter.toml locally
> upload_adapter              # Upload to the platform
> list_adapters               # See all adapters for your component
> list_adapter_versions       # See version history
```

### Using the Developer Portal

Navigate to your skill component → **Adapters** tab → drag and drop your adapter files.

### Using the Developer API

```
POST /api/v1/components/{component_id}/adapters
Content-Type: multipart/form-data

Fields:
  manifest: <adapter.toml file>
  files: <adapter weight files>
```

The API validates your manifest, verifies checksums, rejects unknown architectures, and checks for version conflicts. Maximum upload size: 2 GB.

## Step 4: Load Adapters at Runtime

When your skill runs, use `request_adapter_load` to apply an adapter to a base model:

```rust
use lifesavor_skill_sdk::adapter::*;
use std::time::Duration;

let result = request_adapter_load(AdapterLoadRequest {
    adapter_name: "medical-pii-lora".to_string(),
    base_model: "llama3-8b".to_string(),
    timeout: Some(Duration::from_secs(30)),
    force: false,  // Set true to preempt the current adapter
}).await?;

match result {
    AdapterLoadResult::AdapterApplied { .. } => {
        // Adapter is Hot — run inference now
    }
    AdapterLoadResult::AdapterWarmOnly { current_hot, .. } => {
        // Another adapter is active; yours is loaded but not applied
    }
    AdapterLoadResult::Queued { position, queue_depth } => {
        // Request queued — will be applied when current adapter is released
    }
}
```

### The `force` Flag

- `force: false` — If another adapter is Hot, your request is queued (FIFO, max depth 8). You'll get `Queued` back.
- `force: true` — The currently Hot adapter is demoted to Warm, and yours is applied immediately.

### Timeouts

If your request is queued and not served within `timeout`, you'll receive an `AdapterLoadError::QueueTimeout` error.

## Step 5: Release When Done

Always release the adapter when your skill finishes inference:

```rust
release_adapter("medical-pii-lora", "llama3-8b").await?;
```

This allows other queued skills to use the model. If you don't release, the adapter stays Hot until your skill process exits.

## Error Handling

```rust
use lifesavor_skill_sdk::adapter::AdapterLoadError;

match request_adapter_load(request).await {
    Ok(result) => { /* handle success */ }
    Err(AdapterLoadError::BaseModelNotLoaded { model }) => {
        // The base model isn't loaded — can't apply adapters
    }
    Err(AdapterLoadError::AdapterNotFound { adapter, model }) => {
        // Adapter not registered for this model
    }
    Err(AdapterLoadError::IncompatibleArchitecture { expected, actual }) => {
        // Adapter targets a different architecture
    }
    Err(AdapterLoadError::InsufficientBaseModel { required, available }) => {
        // Base model too small for this adapter
    }
    Err(AdapterLoadError::ChecksumMismatch { file, expected, actual }) => {
        // Adapter file corrupted or tampered
    }
    Err(AdapterLoadError::AdapterQueueFull { depth, max }) => {
        // Queue is full (8 requests) — try again later
    }
    Err(AdapterLoadError::QueueTimeout { elapsed_ms }) => {
        // Request timed out waiting in queue
    }
    Err(AdapterLoadError::NoCapableNode) => {
        // Device lacks hardware and no fleet node available
    }
    Err(other) => {
        // Handle remaining error variants
    }
}
```

## How It Works Under the Hood

When a user installs your skill:

1. Agent reads `[adapter_dependencies]` from your manifest
2. Downloads adapter artifacts from CDN in the background (non-blocking)
3. Verifies SHA-256 checksums of all files
4. Registers adapter with the runtime

When your skill calls `request_adapter_load`:

1. Agent validates adapter-to-model compatibility (architecture, params, modules)
2. Checks base model state (must be Hot or Warm)
3. If no conflict: promotes adapter Cold → Warm → Hot atomically
4. If conflict and `force=false`: enqueues request
5. If conflict and `force=true`: demotes current, applies yours
6. Emits `adapter_applied` lifecycle event

When the device lacks sufficient hardware:

1. Agent queries the user's fleet for a capable device
2. Presents user consent prompt
3. On approval, delegates inference to the remote agent over mTLS
4. Results relay back transparently to your skill

## Version Upgrades

When you publish a new adapter version:

- Users get it automatically (within their `version_constraint` range)
- The new version downloads while the current version stays active
- Swap happens on the next inference request (never mid-request)
- Old version cleaned up after 24-hour grace period
- If download fails, the current version continues operating

## Validation Rules

The platform enforces:

- Adapter name must be non-empty
- Version must be valid SemVer
- `adapter_type` must be `"lora"` or `"qlora"`
- `base_model_architecture` must be a known platform architecture
- `quantization_bits` is required when `adapter_type` is `"qlora"`
- All declared files must be present with matching SHA-256 checksums
- Total file size must match `total_size_bytes` within 1% tolerance
- Duplicate (name, version) uploads are rejected — increment the version

## Resource Limits

The agent enforces per-device limits:

| Limit | Default | Configurable |
|-------|---------|--------------|
| Max adapter disk space | 10 GB | `config.toml [adapters] max_adapter_disk_gb` |
| Max adapters per model | 16 | `config.toml [adapters] max_adapters_per_model` |
| Queue depth per model | 8 | Not configurable |
| Download retries | 3 | `config.toml [adapters] download_retry_count` |
| Retry backoff | 5s × 2^N | `config.toml [adapters] download_retry_base_delay_seconds` |

## Feature Flag

The runtime API (`request_adapter_load`, `release_adapter`) requires the `runtime-api` feature:

```toml
[dependencies]
lifesavor-skill-sdk = { version = "0.1", features = ["runtime-api"] }
```

Without this feature, the types are still available for manifest declaration, but the async runtime functions will panic if called.

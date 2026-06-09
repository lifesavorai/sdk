# Changelog — lifesavor-agent-types

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - Unreleased

### Added

- **`upgrade_manifest` module** — Types for the `[upgrade]` section of component manifests:
  - `UpgradeSection` — top-level upgrade configuration (min agent version, breaking flag, notes, data preservation, features)
  - `UpgradeHealthCheck` — developer-defined health check (HTTP, command, TCP, or none) with retries
  - `UpgradeHookPaths` — lifecycle hook script paths (pre_upgrade, post_upgrade, pre_rollback, post_rollback)
  - `MigrationEntry` — version-specific migration rules with glob pattern matching (`"1.*"` → `"2.0.0"`)
  - `AdapterDependencyDeclaration` — fine-tuned adapter dependency declaration for skills
  - `validate_upgrade_section()` — validates an upgrade section and returns a list of issues
  - `validate_adapter_dependency()` — validates an adapter dependency declaration
- Added `semver` dependency for version constraint validation

## [0.1.0] - 2026-04-07

### Added

- Initial release of the shared interface types crate
- `SystemComponentType` enum with all component variants including `MemoryStore` (renamed from `VectorStore`)
- `ProviderType` enum with `MemoryStore` variant (renamed from `VectorStore`)
- `ComponentDeclaration` unified type shared across all SDK crates
- `SystemComponent` trait with `tool_schemas()` and `declaration()` optional methods
- `BridgeRequest` and `BridgeResponse` types for skill ↔ component communication
- `ErrorChain` type for structured error propagation with subsystem, code, message, and correlation ID
- `ToolSchema` type for self-describing component operations
- `ProviderManifest` and related manifest types for component/skill configuration
- `SandboxConfig` and sandbox-related types
- `CredentialResolver` async trait for credential management
- `StreamingEnvelope` for streaming response support
- Property-based serde round-trip tests for all serializable types
- Zero runtime dependencies (no tokio, no agent-specific crates)

### Notes

- This crate is the root of the SDK dependency tree and must be published to crates.io first
- See `../PUBLISHING.md` for the full publishing workflow

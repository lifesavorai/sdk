// Skill SDK prelude — import the most commonly used types with a single
// `use lifesavor_skill_sdk::prelude::*;`

// Core trait and types
pub use crate::{
    SkillProvider,
    ToolSchema,
    SkillCapabilityDescriptor,
    ExecutionLifecycleEvent,
    SkillProviderError,
    HealthStatus,
    SkillExecutionResult,
};

// MCP transport (feature-gated)
#[cfg(feature = "mcp")]
pub use crate::McpTransport;

// Bridge protocol
pub use crate::{
    BridgeRequest,
    BridgeResponse,
    BridgeError,
    SystemCallRequest,
    SystemCallResponse,
};

// Error chain
pub use crate::{
    ErrorChain,
    ErrorContext,
    Subsystem,
};

// Manifest
pub use crate::{
    ProviderManifest,
    ProviderType,
    SandboxConfig,
    ManifestValidationError,
    parse_manifest,
    validate_manifest,
};

// Credentials
pub use crate::{
    CredentialResolver,
    ResolvedCredential,
    CredentialError,
    AuthConfig,
    CredentialSource,
};

// Upgrade manifest types
pub use crate::{
    UpgradeSection,
    UpgradeHealthCheck,
    UpgradeHookPaths,
    MigrationEntry,
    AdapterDependencyDeclaration,
    validate_upgrade_section,
    validate_adapter_dependency,
};

// Upgrade builders
pub use crate::upgrade_builder::{
    UpgradeSectionBuilder,
    AdapterDependencyBuilder,
};

// Memory types (Req 20.4)
pub use crate::{
    MemoryRecord,
    MemoryProposal,
    MemorySeed,
    PersonaDefinition,
    MemoryType,
    ContentFormat,
    ScopeType,
    MemoryStatus,
    MemoryCreateRequest,
    MemoryReadRequest,
    MemoryUpdateRequest,
    MemoryDeleteRequest,
    MemoryProposeRequest,
    MemorySearchRequest,
    MemoryCreateResponse,
    MemoryReadResponse,
    MemoryUpdateResponse,
    MemoryDeleteResponse,
    MemoryProposeResponse,
    MemorySearchResponse,
    ScoredMemory,
};

// Process sandbox (requires agent-runtime feature)
#[cfg(feature = "agent-runtime")]
pub use crate::ProcessSandbox;

// Tracing
pub use tracing::{info, warn, error, debug, trace, instrument};

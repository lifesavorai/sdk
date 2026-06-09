//! Upgrade manifest types for component developers.
//!
//! Defines the `[upgrade]` section that developers add to their
//! `ProviderManifest.toml` to control how the agent handles upgrades
//! of their component.
//!
//! # Example
//!
//! ```toml
//! [upgrade]
//! min_agent_version = "2.5.0"
//! breaking = false
//! notes = "Improved medical NER accuracy"
//! preserve_data = ["data/", "user_config/"]
//!
//! [upgrade.health_check]
//! type = "command"
//! endpoint = "./scripts/health-check.sh"
//! timeout_secs = 10
//! retries = 3
//!
//! [upgrade.hooks]
//! pre_upgrade = "scripts/pre-upgrade.sh"
//! post_upgrade = "scripts/post-upgrade.sh"
//!
//! [[upgrade.migrations]]
//! from = "1.*"
//! to = "2.0.0"
//! script = "migrations/v1_to_v2.sh"
//! description = "Migrate config schema"
//! required = true
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level upgrade section
// ---------------------------------------------------------------------------

/// The `[upgrade]` section of a ProviderManifest.toml.
///
/// All fields are optional — components without an `[upgrade]` section
/// still get the agent's default upgrade behavior (backup → replace → register).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpgradeSection {
    /// Minimum agent version required for this component version.
    /// If the running agent is older, the upgrade won't be attempted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_agent_version: Option<String>,

    /// Whether this version introduces breaking changes.
    /// When `true`, the platform uses canary rollout (gradual fleet deployment).
    #[serde(default)]
    pub breaking: bool,

    /// Human-readable release notes for this version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Agent features required by this version (e.g., `["torch", "onnx"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_agent_features: Vec<String>,

    /// Directories (relative to install dir) that should be preserved across
    /// upgrades. The agent will NOT delete these during the backup-and-replace
    /// cycle.
    ///
    /// Use for user-generated data, databases, caches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserve_data: Vec<String>,

    /// Health check definition for post-upgrade verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_check: Option<UpgradeHealthCheck>,

    /// Lifecycle hook scripts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<UpgradeHookPaths>,

    /// Version-specific migration rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migrations: Vec<MigrationEntry>,
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// Developer-defined health check run after an upgrade.
///
/// If the check fails after all retries, the agent rolls back automatically.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpgradeHealthCheck {
    /// Check type: `"http"`, `"command"`, `"tcp"`, or `"none"`.
    #[serde(rename = "type")]
    pub check_type: String,

    /// For HTTP: endpoint path (e.g., `"/health"`).
    /// For command: script path relative to install dir.
    /// For TCP: port number as string (e.g., `"8080"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Timeout per attempt in seconds. Default: 10.
    #[serde(default = "default_10")]
    pub timeout_secs: u64,

    /// Number of retries before declaring failure. Default: 3.
    #[serde(default = "default_3")]
    pub retries: u32,

    /// Delay between retries in seconds. Default: 2.
    #[serde(default = "default_2")]
    pub retry_delay_secs: u64,

    /// For HTTP: status codes that indicate success. Default: [200].
    #[serde(default = "default_success_codes", skip_serializing_if = "Vec::is_empty")]
    pub success_codes: Vec<u16>,
}

fn default_10() -> u64 { 10 }
fn default_3() -> u32 { 3 }
fn default_2() -> u64 { 2 }
fn default_success_codes() -> Vec<u16> { vec![200] }

// ---------------------------------------------------------------------------
// Lifecycle hooks
// ---------------------------------------------------------------------------

/// Paths to lifecycle hook scripts (relative to install directory).
///
/// The agent runs these at specific points during the upgrade pipeline.
/// Scripts receive environment variables:
/// - `LIFESAVOR_HOOK` — the hook name
/// - `LIFESAVOR_INSTALL_DIR` — the component's install directory
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpgradeHookPaths {
    /// Runs before backup. Use to flush state, stop workers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_upgrade: Option<String>,

    /// Runs after install. Use to migrate data, warm caches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_upgrade: Option<String>,

    /// Runs before restoring backup on failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_rollback: Option<String>,

    /// Runs after backup is restored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_rollback: Option<String>,
}

// ---------------------------------------------------------------------------
// Migration rules
// ---------------------------------------------------------------------------

/// A version-specific migration rule.
///
/// The `from` field supports glob patterns:
/// - `"1.2.3"` — exact version match
/// - `"1.2.*"` — any patch in 1.2.x
/// - `"1.*"` — any version in 1.x.x
/// - `"*"` — any source version
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationEntry {
    /// Source version pattern (glob).
    pub from: String,

    /// Target version (exact).
    pub to: String,

    /// Path to migration script (relative to install dir).
    pub script: String,

    /// Human-readable description (for logging).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// If `true` (default), script failure triggers rollback.
    /// If `false`, failure is logged but upgrade continues.
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool { true }

// ---------------------------------------------------------------------------
// Adapter dependency (for skill manifests)
// ---------------------------------------------------------------------------

/// Declares that a skill depends on a fine-tuned adapter.
///
/// Added to the skill manifest's `[[adapter_dependencies]]` array.
/// The agent handles all download/install/upgrade logic automatically.
///
/// # Example
///
/// ```toml
/// [[adapter_dependencies]]
/// name = "medical-ner-lora"
/// adapter_type = "lora"
/// base_model_architecture = "llama3"
/// min_base_params = 7000
/// source_url = "https://cdn.lifesavor.ai/adapters/medical-ner/1.2.0.tar.gz"
/// version = "1.2.0"
/// version_constraint = ">=1.0.0, <2.0.0"
/// checksum_sha256 = "abc123def456..."
/// size_bytes = 52428800
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterDependencyDeclaration {
    /// Adapter name (unique within the skill).
    pub name: String,

    /// Adapter type: `"lora"` or `"qlora"`.
    pub adapter_type: String,

    /// Required base model architecture (e.g., `"llama3"`, `"mistral"`).
    pub base_model_architecture: String,

    /// Minimum base model size in millions of parameters.
    #[serde(default)]
    pub min_base_params: u64,

    /// CDN URL for downloading the adapter artifact.
    pub source_url: String,

    /// Specific adapter version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// SemVer version constraint (e.g., `">=1.0.0, <2.0.0"`).
    /// Used to determine if a new adapter version satisfies the skill's needs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_constraint: Option<String>,

    /// SHA-256 checksum of the adapter artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256: Option<String>,

    /// Size of the adapter artifact in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate an UpgradeSection for common errors.
///
/// Returns a list of warnings/errors for the developer.
pub fn validate_upgrade_section(section: &UpgradeSection) -> Vec<String> {
    let mut issues = Vec::new();

    // Validate min_agent_version is valid semver
    if let Some(ref v) = section.min_agent_version {
        if semver::Version::parse(v).is_err() {
            issues.push(format!(
                "upgrade.min_agent_version '{}' is not valid semver",
                v
            ));
        }
    }

    // Validate health check type
    if let Some(ref hc) = section.health_check {
        match hc.check_type.as_str() {
            "http" | "command" | "tcp" | "none" => {}
            other => {
                issues.push(format!(
                    "upgrade.health_check.type '{}' is not valid (use: http, command, tcp, none)",
                    other
                ));
            }
        }

        if hc.check_type == "http" && hc.endpoint.is_none() {
            issues.push(
                "upgrade.health_check: HTTP type requires an 'endpoint' field".to_string(),
            );
        }
    }

    // Validate migration scripts reference valid paths (just format checks)
    for (i, migration) in section.migrations.iter().enumerate() {
        if migration.from.is_empty() {
            issues.push(format!("upgrade.migrations[{}].from must not be empty", i));
        }
        if migration.to.is_empty() {
            issues.push(format!("upgrade.migrations[{}].to must not be empty", i));
        }
        if migration.script.is_empty() {
            issues.push(format!("upgrade.migrations[{}].script must not be empty", i));
        }
        // Validate 'to' is valid semver
        if !migration.to.is_empty() && semver::Version::parse(&migration.to).is_err() {
            issues.push(format!(
                "upgrade.migrations[{}].to '{}' is not valid semver",
                i, migration.to
            ));
        }
    }

    // Validate preserve_data paths don't escape the install directory
    for path in &section.preserve_data {
        if path.contains("..") {
            issues.push(format!(
                "upgrade.preserve_data '{}' must not contain '..'",
                path
            ));
        }
        if path.starts_with('/') {
            issues.push(format!(
                "upgrade.preserve_data '{}' must be relative (no leading '/')",
                path
            ));
        }
    }

    issues
}

/// Validate an AdapterDependencyDeclaration.
pub fn validate_adapter_dependency(dep: &AdapterDependencyDeclaration) -> Vec<String> {
    let mut issues = Vec::new();

    if dep.name.is_empty() {
        issues.push("adapter_dependencies: 'name' must not be empty".to_string());
    }

    match dep.adapter_type.as_str() {
        "lora" | "qlora" => {}
        other => {
            issues.push(format!(
                "adapter_dependencies: adapter_type '{}' is not valid (use: lora, qlora)",
                other
            ));
        }
    }

    if dep.base_model_architecture.is_empty() {
        issues.push(
            "adapter_dependencies: 'base_model_architecture' must not be empty".to_string(),
        );
    }

    if dep.source_url.is_empty() {
        issues.push("adapter_dependencies: 'source_url' must not be empty".to_string());
    } else if !dep.source_url.starts_with("https://") {
        issues.push(
            "adapter_dependencies: 'source_url' must use HTTPS".to_string(),
        );
    }

    if let Some(ref constraint) = dep.version_constraint {
        if semver::VersionReq::parse(constraint).is_err() {
            issues.push(format!(
                "adapter_dependencies: version_constraint '{}' is not valid semver range",
                constraint
            ));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_upgrade_section() {
        let toml = r#"
[upgrade]
breaking = false
"#;
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(default)]
            upgrade: UpgradeSection,
        }
        let w: Wrapper = toml::from_str(toml).unwrap();
        assert!(!w.upgrade.breaking);
        assert!(w.upgrade.min_agent_version.is_none());
        assert!(w.upgrade.migrations.is_empty());
    }

    #[test]
    fn test_parse_full_upgrade_section() {
        let toml = r#"
[upgrade]
min_agent_version = "2.5.0"
breaking = true
notes = "Major rewrite"
required_agent_features = ["torch"]
preserve_data = ["data/", "cache/"]

[upgrade.health_check]
type = "http"
endpoint = "/health"
timeout_secs = 15
retries = 5
retry_delay_secs = 3
success_codes = [200, 204]

[upgrade.hooks]
pre_upgrade = "scripts/pre.sh"
post_upgrade = "scripts/post.sh"

[[upgrade.migrations]]
from = "1.*"
to = "2.0.0"
script = "migrations/v1_to_v2.sh"
description = "Schema migration"
required = true
"#;
        #[derive(Deserialize)]
        struct Wrapper {
            upgrade: UpgradeSection,
        }
        let w: Wrapper = toml::from_str(toml).unwrap();
        let u = w.upgrade;

        assert_eq!(u.min_agent_version.as_deref(), Some("2.5.0"));
        assert!(u.breaking);
        assert_eq!(u.notes.as_deref(), Some("Major rewrite"));
        assert_eq!(u.required_agent_features, vec!["torch"]);
        assert_eq!(u.preserve_data, vec!["data/", "cache/"]);

        let hc = u.health_check.unwrap();
        assert_eq!(hc.check_type, "http");
        assert_eq!(hc.endpoint.as_deref(), Some("/health"));
        assert_eq!(hc.retries, 5);
        assert_eq!(hc.success_codes, vec![200, 204]);

        let hooks = u.hooks.unwrap();
        assert_eq!(hooks.pre_upgrade.as_deref(), Some("scripts/pre.sh"));

        assert_eq!(u.migrations.len(), 1);
        assert_eq!(u.migrations[0].from, "1.*");
        assert_eq!(u.migrations[0].to, "2.0.0");
    }

    #[test]
    fn test_default_upgrade_section_is_empty() {
        let section = UpgradeSection::default();
        assert!(!section.breaking);
        assert!(section.min_agent_version.is_none());
        assert!(section.health_check.is_none());
        assert!(section.hooks.is_none());
        assert!(section.migrations.is_empty());
        assert!(section.preserve_data.is_empty());
    }

    #[test]
    fn test_validate_good_section() {
        let section = UpgradeSection {
            min_agent_version: Some("2.5.0".to_string()),
            breaking: false,
            notes: Some("Fix".to_string()),
            preserve_data: vec!["data/".to_string()],
            health_check: Some(UpgradeHealthCheck {
                check_type: "command".to_string(),
                endpoint: Some("./check.sh".to_string()),
                timeout_secs: 10,
                retries: 3,
                retry_delay_secs: 2,
                success_codes: vec![],
            }),
            hooks: None,
            migrations: vec![MigrationEntry {
                from: "1.*".to_string(),
                to: "2.0.0".to_string(),
                script: "migrate.sh".to_string(),
                description: None,
                required: true,
            }],
            required_agent_features: vec![],
        };

        let issues = validate_upgrade_section(&section);
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
    }

    #[test]
    fn test_validate_bad_version() {
        let section = UpgradeSection {
            min_agent_version: Some("not-semver".to_string()),
            ..Default::default()
        };
        let issues = validate_upgrade_section(&section);
        assert!(issues.iter().any(|i| i.contains("not valid semver")));
    }

    #[test]
    fn test_validate_bad_health_check_type() {
        let section = UpgradeSection {
            health_check: Some(UpgradeHealthCheck {
                check_type: "magic".to_string(),
                endpoint: None,
                timeout_secs: 10,
                retries: 3,
                retry_delay_secs: 2,
                success_codes: vec![],
            }),
            ..Default::default()
        };
        let issues = validate_upgrade_section(&section);
        assert!(issues.iter().any(|i| i.contains("not valid")));
    }

    #[test]
    fn test_validate_preserve_data_path_traversal() {
        let section = UpgradeSection {
            preserve_data: vec!["../etc/passwd".to_string()],
            ..Default::default()
        };
        let issues = validate_upgrade_section(&section);
        assert!(issues.iter().any(|i| i.contains("..")));
    }

    #[test]
    fn test_validate_preserve_data_absolute_path() {
        let section = UpgradeSection {
            preserve_data: vec!["/var/secret".to_string()],
            ..Default::default()
        };
        let issues = validate_upgrade_section(&section);
        assert!(issues.iter().any(|i| i.contains("relative")));
    }

    #[test]
    fn test_validate_adapter_dependency_good() {
        let dep = AdapterDependencyDeclaration {
            name: "medical-ner".to_string(),
            adapter_type: "lora".to_string(),
            base_model_architecture: "llama3".to_string(),
            min_base_params: 7000,
            source_url: "https://cdn.lifesavor.ai/adapter.tar.gz".to_string(),
            version: Some("1.0.0".to_string()),
            version_constraint: Some(">=1.0.0".to_string()),
            checksum_sha256: Some("abc123".to_string()),
            size_bytes: Some(50_000_000),
        };
        let issues = validate_adapter_dependency(&dep);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_adapter_dependency_bad() {
        let dep = AdapterDependencyDeclaration {
            name: "".to_string(),
            adapter_type: "invalid".to_string(),
            base_model_architecture: "".to_string(),
            min_base_params: 0,
            source_url: "http://insecure.com/adapter.tar.gz".to_string(),
            version: None,
            version_constraint: Some("not-valid".to_string()),
            checksum_sha256: None,
            size_bytes: None,
        };
        let issues = validate_adapter_dependency(&dep);
        assert!(issues.len() >= 4); // name, type, architecture, url, constraint
    }

    #[test]
    fn test_round_trip_serialization() {
        let section = UpgradeSection {
            min_agent_version: Some("1.0.0".to_string()),
            breaking: true,
            notes: Some("test".to_string()),
            required_agent_features: vec!["torch".to_string()],
            preserve_data: vec!["data/".to_string()],
            health_check: Some(UpgradeHealthCheck {
                check_type: "tcp".to_string(),
                endpoint: Some("9090".to_string()),
                timeout_secs: 5,
                retries: 2,
                retry_delay_secs: 1,
                success_codes: vec![],
            }),
            hooks: Some(UpgradeHookPaths {
                pre_upgrade: Some("pre.sh".to_string()),
                post_upgrade: None,
                pre_rollback: None,
                post_rollback: None,
            }),
            migrations: vec![MigrationEntry {
                from: "*".to_string(),
                to: "1.0.0".to_string(),
                script: "init.sh".to_string(),
                description: Some("Initial".to_string()),
                required: false,
            }],
        };

        let toml_str = toml::to_string_pretty(&section).unwrap();
        let parsed: UpgradeSection = toml::from_str(&toml_str).unwrap();
        assert_eq!(section, parsed);
    }
}

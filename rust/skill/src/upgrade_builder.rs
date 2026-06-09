//! Fluent builders for the `[upgrade]` manifest section and adapter dependencies.
//!
//! Provides compile-time safe, ergonomic APIs for declaring upgrade behavior
//! and adapter dependencies in skill manifests.
//!
//! # Example
//!
//! ```rust,ignore
//! use lifesavor_skill_sdk::upgrade_builder::{UpgradeSectionBuilder, AdapterDependencyBuilder};
//!
//! let upgrade = UpgradeSectionBuilder::new()
//!     .min_agent_version("2.5.0")
//!     .preserve_data("data/")
//!     .preserve_data("cache/")
//!     .health_check_command("./scripts/health-check.sh")
//!     .pre_upgrade_hook("scripts/pre-upgrade.sh")
//!     .post_upgrade_hook("scripts/post-upgrade.sh")
//!     .migration("1.*", "2.0.0", "migrations/v1_to_v2.sh")
//!         .description("Migrate config schema")
//!         .required()
//!     .build()?;
//!
//! let adapter = AdapterDependencyBuilder::new("medical-ner-lora")
//!     .lora()
//!     .base_model("llama3", 7000)
//!     .source("https://cdn.lifesavor.ai/adapters/medical-ner/1.2.0.tar.gz")
//!     .version("1.2.0")
//!     .version_constraint(">=1.0.0, <2.0.0")
//!     .checksum("abc123def456...")
//!     .size_bytes(52_428_800)
//!     .build()?;
//! ```

use lifesavor_agent_types::upgrade_manifest::{
    AdapterDependencyDeclaration, MigrationEntry, UpgradeHealthCheck, UpgradeHookPaths,
    UpgradeSection, validate_adapter_dependency, validate_upgrade_section,
};

use crate::error::SkillSdkError;

// ---------------------------------------------------------------------------
// UpgradeSectionBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for constructing an [`UpgradeSection`].
pub struct UpgradeSectionBuilder {
    section: UpgradeSection,
}

impl UpgradeSectionBuilder {
    /// Create a new builder with defaults (no restrictions, no hooks).
    pub fn new() -> Self {
        Self {
            section: UpgradeSection::default(),
        }
    }

    /// Set the minimum agent version required for this component version.
    ///
    /// Agents older than this will skip the upgrade entirely.
    pub fn min_agent_version(mut self, version: &str) -> Self {
        self.section.min_agent_version = Some(version.to_string());
        self
    }

    /// Mark this version as breaking.
    ///
    /// The platform will use canary rollout (gradual deployment) instead of
    /// immediate fleet-wide upgrade.
    pub fn breaking(mut self) -> Self {
        self.section.breaking = true;
        self
    }

    /// Set release notes for this version.
    pub fn notes(mut self, notes: &str) -> Self {
        self.section.notes = Some(notes.to_string());
        self
    }

    /// Add an agent feature requirement (e.g., `"torch"`, `"onnx"`).
    pub fn require_feature(mut self, feature: &str) -> Self {
        self.section.required_agent_features.push(feature.to_string());
        self
    }

    /// Declare a directory to preserve across upgrades.
    ///
    /// Path must be relative to the install directory (no `..` or leading `/`).
    /// Use for databases, user config, caches.
    pub fn preserve_data(mut self, path: &str) -> Self {
        self.section.preserve_data.push(path.to_string());
        self
    }

    /// Set an HTTP health check.
    ///
    /// After upgrade, the agent GETs this endpoint and checks for a 200 status.
    /// On failure (after retries), the upgrade is rolled back.
    pub fn health_check_http(mut self, endpoint: &str) -> Self {
        self.section.health_check = Some(UpgradeHealthCheck {
            check_type: "http".to_string(),
            endpoint: Some(endpoint.to_string()),
            timeout_secs: 10,
            retries: 3,
            retry_delay_secs: 2,
            success_codes: vec![200],
        });
        self
    }

    /// Set a command-based health check.
    ///
    /// After upgrade, the agent runs this script. Exit 0 = healthy, non-zero = rollback.
    pub fn health_check_command(mut self, script_path: &str) -> Self {
        self.section.health_check = Some(UpgradeHealthCheck {
            check_type: "command".to_string(),
            endpoint: Some(script_path.to_string()),
            timeout_secs: 10,
            retries: 3,
            retry_delay_secs: 2,
            success_codes: vec![],
        });
        self
    }

    /// Set a TCP health check.
    ///
    /// After upgrade, the agent tries connecting to this port. Success = healthy.
    pub fn health_check_tcp(mut self, port: u16) -> Self {
        self.section.health_check = Some(UpgradeHealthCheck {
            check_type: "tcp".to_string(),
            endpoint: Some(port.to_string()),
            timeout_secs: 10,
            retries: 3,
            retry_delay_secs: 2,
            success_codes: vec![],
        });
        self
    }

    /// Customize health check retries (default: 3 retries, 2s delay, 10s timeout).
    pub fn health_check_retries(mut self, retries: u32, delay_secs: u64, timeout_secs: u64) -> Self {
        if let Some(ref mut hc) = self.section.health_check {
            hc.retries = retries;
            hc.retry_delay_secs = delay_secs;
            hc.timeout_secs = timeout_secs;
        }
        self
    }

    /// Set the pre-upgrade hook script path.
    ///
    /// Runs before the backup is taken. Use to flush caches, stop background workers.
    pub fn pre_upgrade_hook(mut self, path: &str) -> Self {
        let hooks = self.section.hooks.get_or_insert_with(UpgradeHookPaths::default);
        hooks.pre_upgrade = Some(path.to_string());
        self
    }

    /// Set the post-upgrade hook script path.
    ///
    /// Runs after the new version is installed. Use to warm caches, run migrations.
    pub fn post_upgrade_hook(mut self, path: &str) -> Self {
        let hooks = self.section.hooks.get_or_insert_with(UpgradeHookPaths::default);
        hooks.post_upgrade = Some(path.to_string());
        self
    }

    /// Set the pre-rollback hook script path.
    ///
    /// Runs before restoring the backup on failure. Use to clean up state.
    pub fn pre_rollback_hook(mut self, path: &str) -> Self {
        let hooks = self.section.hooks.get_or_insert_with(UpgradeHookPaths::default);
        hooks.pre_rollback = Some(path.to_string());
        self
    }

    /// Set the post-rollback hook script path.
    ///
    /// Runs after the backup is restored. Use to restart old-version workers.
    pub fn post_rollback_hook(mut self, path: &str) -> Self {
        let hooks = self.section.hooks.get_or_insert_with(UpgradeHookPaths::default);
        hooks.post_rollback = Some(path.to_string());
        self
    }

    /// Add a migration rule (required by default).
    ///
    /// `from` supports glob patterns: `"1.*"`, `"1.2.*"`, `"*"`.
    pub fn migration(mut self, from: &str, to: &str, script: &str) -> MigrationBuilder {
        MigrationBuilder {
            parent: self,
            entry: MigrationEntry {
                from: from.to_string(),
                to: to.to_string(),
                script: script.to_string(),
                description: None,
                required: true,
            },
        }
    }

    /// Add a migration rule directly (without the sub-builder).
    pub fn add_migration(mut self, entry: MigrationEntry) -> Self {
        self.section.migrations.push(entry);
        self
    }

    /// Validate and build the [`UpgradeSection`].
    ///
    /// # Errors
    ///
    /// Returns [`SkillSdkError::ConfigBuilder`] if validation fails
    /// (invalid semver, path traversal in preserve_data, bad health check type).
    pub fn build(self) -> Result<UpgradeSection, SkillSdkError> {
        let issues = validate_upgrade_section(&self.section);
        if !issues.is_empty() {
            return Err(SkillSdkError::ConfigBuilder(
                format!("Upgrade section validation failed:\n  - {}", issues.join("\n  - ")),
            ));
        }
        Ok(self.section)
    }

    /// Build without validation (for testing or generated manifests).
    pub fn build_unchecked(self) -> UpgradeSection {
        self.section
    }

    /// Serialize the upgrade section to a TOML string.
    ///
    /// Useful for generating `component-manifest.toml` files programmatically.
    pub fn to_toml(&self) -> Result<String, SkillSdkError> {
        let issues = validate_upgrade_section(&self.section);
        if !issues.is_empty() {
            return Err(SkillSdkError::ConfigBuilder(
                format!("Upgrade section validation failed:\n  - {}", issues.join("\n  - ")),
            ));
        }

        toml::to_string_pretty(&self.section).map_err(|e| {
            SkillSdkError::ConfigBuilder(format!("TOML serialization failed: {}", e))
        })
    }
}

impl Default for UpgradeSectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MigrationBuilder (sub-builder for migration entries)
// ---------------------------------------------------------------------------

/// Sub-builder for configuring a single migration rule.
///
/// Call methods to customize, then use implicit conversion back to
/// [`UpgradeSectionBuilder`] via the parent reference.
pub struct MigrationBuilder {
    parent: UpgradeSectionBuilder,
    entry: MigrationEntry,
}

impl MigrationBuilder {
    /// Set a description for this migration (logged during execution).
    pub fn description(mut self, desc: &str) -> Self {
        self.entry.description = Some(desc.to_string());
        self
    }

    /// Mark this migration as required (default). Failure triggers rollback.
    pub fn required(mut self) -> Self {
        self.entry.required = true;
        self
    }

    /// Mark this migration as optional. Failure is logged but doesn't rollback.
    pub fn optional(mut self) -> Self {
        self.entry.required = false;
        self
    }

    /// Finalize this migration and return to the parent builder.
    pub fn done(mut self) -> UpgradeSectionBuilder {
        self.parent.section.migrations.push(self.entry);
        self.parent
    }

    /// Shortcut: finalize and immediately build the parent.
    pub fn build(self) -> Result<UpgradeSection, SkillSdkError> {
        self.done().build()
    }
}

// ---------------------------------------------------------------------------
// AdapterDependencyBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for constructing an [`AdapterDependencyDeclaration`].
pub struct AdapterDependencyBuilder {
    dep: AdapterDependencyDeclaration,
}

impl AdapterDependencyBuilder {
    /// Create a new adapter dependency builder with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            dep: AdapterDependencyDeclaration {
                name: name.to_string(),
                adapter_type: "lora".to_string(),
                base_model_architecture: String::new(),
                min_base_params: 0,
                source_url: String::new(),
                version: None,
                version_constraint: None,
                checksum_sha256: None,
                size_bytes: None,
            },
        }
    }

    /// Set adapter type to LoRA (default).
    pub fn lora(mut self) -> Self {
        self.dep.adapter_type = "lora".to_string();
        self
    }

    /// Set adapter type to QLoRA (quantized LoRA).
    pub fn qlora(mut self) -> Self {
        self.dep.adapter_type = "qlora".to_string();
        self
    }

    /// Set the required base model architecture and minimum parameter count.
    ///
    /// `min_params` is in millions (e.g., 7000 = 7B parameters).
    pub fn base_model(mut self, architecture: &str, min_params: u64) -> Self {
        self.dep.base_model_architecture = architecture.to_string();
        self.dep.min_base_params = min_params;
        self
    }

    /// Set the download URL for the adapter artifact.
    ///
    /// Must be HTTPS.
    pub fn source(mut self, url: &str) -> Self {
        self.dep.source_url = url.to_string();
        self
    }

    /// Pin to a specific adapter version.
    pub fn version(mut self, version: &str) -> Self {
        self.dep.version = Some(version.to_string());
        self
    }

    /// Set a SemVer version constraint (e.g., `">=1.0.0, <2.0.0"`).
    ///
    /// The agent uses this to decide if an installed adapter satisfies
    /// the dependency or needs upgrading.
    pub fn version_constraint(mut self, constraint: &str) -> Self {
        self.dep.version_constraint = Some(constraint.to_string());
        self
    }

    /// Set the expected SHA-256 checksum of the adapter artifact.
    pub fn checksum(mut self, sha256: &str) -> Self {
        self.dep.checksum_sha256 = Some(sha256.to_string());
        self
    }

    /// Set the artifact size in bytes.
    pub fn size_bytes(mut self, bytes: u64) -> Self {
        self.dep.size_bytes = Some(bytes);
        self
    }

    /// Validate and build the [`AdapterDependencyDeclaration`].
    ///
    /// # Errors
    ///
    /// Returns [`SkillSdkError::ConfigBuilder`] if the declaration is invalid
    /// (empty name, bad adapter type, non-HTTPS URL, invalid version constraint).
    pub fn build(self) -> Result<AdapterDependencyDeclaration, SkillSdkError> {
        let issues = validate_adapter_dependency(&self.dep);
        if !issues.is_empty() {
            return Err(SkillSdkError::ConfigBuilder(
                format!("Adapter dependency validation failed:\n  - {}", issues.join("\n  - ")),
            ));
        }
        Ok(self.dep)
    }

    /// Build without validation (for testing).
    pub fn build_unchecked(self) -> AdapterDependencyDeclaration {
        self.dep
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_upgrade_section() {
        let section = UpgradeSectionBuilder::new().build().unwrap();
        assert!(!section.breaking);
        assert!(section.min_agent_version.is_none());
        assert!(section.migrations.is_empty());
    }

    #[test]
    fn test_full_upgrade_section() {
        let section = UpgradeSectionBuilder::new()
            .min_agent_version("2.5.0")
            .breaking()
            .notes("Big update")
            .require_feature("torch")
            .preserve_data("data/")
            .preserve_data("cache/")
            .health_check_command("./check.sh")
            .health_check_retries(5, 3, 15)
            .pre_upgrade_hook("scripts/pre.sh")
            .post_upgrade_hook("scripts/post.sh")
            .migration("1.*", "2.0.0", "migrations/v1_to_v2.sh")
                .description("Schema migration")
                .required()
                .done()
            .migration("*", "2.0.0", "migrations/catch_all.sh")
                .optional()
                .done()
            .build()
            .unwrap();

        assert!(section.breaking);
        assert_eq!(section.min_agent_version.as_deref(), Some("2.5.0"));
        assert_eq!(section.notes.as_deref(), Some("Big update"));
        assert_eq!(section.required_agent_features, vec!["torch"]);
        assert_eq!(section.preserve_data, vec!["data/", "cache/"]);

        let hc = section.health_check.unwrap();
        assert_eq!(hc.check_type, "command");
        assert_eq!(hc.retries, 5);
        assert_eq!(hc.retry_delay_secs, 3);
        assert_eq!(hc.timeout_secs, 15);

        let hooks = section.hooks.unwrap();
        assert_eq!(hooks.pre_upgrade.as_deref(), Some("scripts/pre.sh"));
        assert_eq!(hooks.post_upgrade.as_deref(), Some("scripts/post.sh"));

        assert_eq!(section.migrations.len(), 2);
        assert!(section.migrations[0].required);
        assert!(!section.migrations[1].required);
    }

    #[test]
    fn test_upgrade_section_bad_version_rejected() {
        let result = UpgradeSectionBuilder::new()
            .min_agent_version("not-a-version")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not valid semver"));
    }

    #[test]
    fn test_upgrade_section_path_traversal_rejected() {
        let result = UpgradeSectionBuilder::new()
            .preserve_data("../etc/passwd")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".."));
    }

    #[test]
    fn test_adapter_dependency_basic() {
        let dep = AdapterDependencyBuilder::new("medical-ner")
            .lora()
            .base_model("llama3", 7000)
            .source("https://cdn.lifesavor.ai/adapters/medical-ner.tar.gz")
            .version("1.0.0")
            .checksum("abc123")
            .size_bytes(50_000_000)
            .build()
            .unwrap();

        assert_eq!(dep.name, "medical-ner");
        assert_eq!(dep.adapter_type, "lora");
        assert_eq!(dep.base_model_architecture, "llama3");
        assert_eq!(dep.min_base_params, 7000);
        assert_eq!(dep.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn test_adapter_dependency_qlora() {
        let dep = AdapterDependencyBuilder::new("legal-qlora")
            .qlora()
            .base_model("mistral", 7000)
            .source("https://cdn.lifesavor.ai/adapters/legal.tar.gz")
            .version_constraint(">=1.0.0, <2.0.0")
            .build()
            .unwrap();

        assert_eq!(dep.adapter_type, "qlora");
        assert_eq!(dep.version_constraint.as_deref(), Some(">=1.0.0, <2.0.0"));
    }

    #[test]
    fn test_adapter_dependency_empty_name_rejected() {
        let result = AdapterDependencyBuilder::new("")
            .lora()
            .base_model("llama3", 7000)
            .source("https://cdn.lifesavor.ai/x.tar.gz")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_dependency_non_https_rejected() {
        let result = AdapterDependencyBuilder::new("test")
            .lora()
            .base_model("llama3", 7000)
            .source("http://insecure.example.com/adapter.tar.gz")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTPS"));
    }

    #[test]
    fn test_adapter_dependency_bad_constraint_rejected() {
        let result = AdapterDependencyBuilder::new("test")
            .lora()
            .base_model("llama3", 7000)
            .source("https://cdn.lifesavor.ai/x.tar.gz")
            .version_constraint("not valid semver range")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_to_toml_output() {
        let builder = UpgradeSectionBuilder::new()
            .min_agent_version("1.0.0")
            .preserve_data("data/")
            .health_check_http("/health");

        let toml_str = builder.to_toml().unwrap();
        assert!(toml_str.contains("min_agent_version"));
        assert!(toml_str.contains("1.0.0"));
        assert!(toml_str.contains("data/"));
        assert!(toml_str.contains("/health"));
    }

    #[test]
    fn test_migration_shortcut_build() {
        let section = UpgradeSectionBuilder::new()
            .migration("1.*", "2.0.0", "migrate.sh")
                .description("Test")
                .build()
                .unwrap();

        assert_eq!(section.migrations.len(), 1);
        assert_eq!(section.migrations[0].description.as_deref(), Some("Test"));
    }
}

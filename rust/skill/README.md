# lifesavor-skill-sdk

Build skill providers for the Life Savor agent using the `SkillProvider` trait.

Skills are sandboxed extensions that expose tools to the agent via JSON stdin/stdout or MCP protocols. They run as child processes with restricted filesystem access, environment variables, and output size limits. The SDK provides builders for provider scaffolds, tool schemas, and sandbox compliance checking.

## Target Trait

[`SkillProvider`](https://docs.rs/lifesavor-agent/latest/lifesavor_agent/providers/skill_provider/trait.SkillProvider.html) — defines `invoke`, `list_tools`, `health_check`, and `capability_descriptor` methods.

## Prerequisites

- Rust toolchain **1.75+** (edition 2021)
- Access to the `lifesavor-agent` crate (path dependency or published version)
- Familiarity with `async-trait` and `tokio`

## Quickstart

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
lifesavor-skill-sdk = { path = "../SDK/skill" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

Build a minimal skill with a tool schema:

```rust
use lifesavor_skill_sdk::prelude::*;
use lifesavor_skill_sdk::builder::{SkillProviderBuilder, ToolSchemaBuilder};

fn main() -> lifesavor_skill_sdk::error::Result<()> {
    let tool = ToolSchemaBuilder::new()
        .name("greet")
        .description("Returns a greeting message")
        .input_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        }))
        .build()?;

    let manifest = parse_manifest(r#"
        provider_type = "skill"
        instance_name = "my-greeter"
        sdk_version = "0.1.0"

        [connection]
        command = "./target/release/my-greeter"
        transport = "json_stdio"

        [auth]
        strategy = "none"

        [health_check]
        method = "process_alive"
        interval_seconds = 30
        timeout_seconds = 5
    "#)?;

    let provider = SkillProviderBuilder::new(manifest)?
        .tool(tool)
        .build();

    println!("Skill provider scaffold created");
    Ok(())
}
```

## Feature Flags

| Flag | Description |
|------|-------------|
| `mcp` | MCP transport types and protocol support |
| `analytics` | Developer Portal analytics reporting |
| `runtime-api` | Adapter loading and host channel communication (requires tokio) |

All features are disabled by default. The core JSON stdin/stdout types are always available.

## Upgrade & Adapter Support

Declare how your component behaves during upgrades and what fine-tuned adapters it depends on:

```rust
use lifesavor_skill_sdk::prelude::*;

// Declare upgrade behavior
let upgrade = UpgradeSectionBuilder::new()
    .min_agent_version("2.5.0")
    .preserve_data("data/")
    .health_check_command("./scripts/health-check.sh")
    .pre_upgrade_hook("scripts/pre-upgrade.sh")
    .post_upgrade_hook("scripts/post-upgrade.sh")
    .migration("1.*", "2.0.0", "migrations/v1_to_v2.sh")
        .description("Migrate config schema")
        .required()
    .build()
    .unwrap();

// Declare adapter dependencies
let adapter = AdapterDependencyBuilder::new("medical-ner-lora")
    .lora()
    .base_model("llama3", 7000)
    .source("https://cdn.lifesavor.ai/adapters/medical-ner/1.2.0.tar.gz")
    .version("1.2.0")
    .version_constraint(">=1.0.0, <2.0.0")
    .checksum("abc123def456...")
    .size_bytes(52_428_800)
    .build()
    .unwrap();
```

The agent handles all upgrade orchestration (backup, download, verify, install, rollback) automatically. See the [Component Upgrades docs](https://docs.lifesavor.ai/developer-tools/component-upgrades) for the full manifest reference.

## Binaries

- `sandbox-runner` — Spawns a skill as a child process with `ProcessSandbox` restrictions for local testing without a running agent. Use `cargo run --bin sandbox-runner -- --manifest path/to/manifest.toml`.

## Examples

- [`examples/json_stdio_skill/`](examples/json_stdio_skill/) — Minimal JSON stdin/stdout skill
- [`examples/mcp_skill/`](examples/mcp_skill/) — Minimal MCP server skill
- [`examples/bridge_access/`](examples/bridge_access/) — Access system components via `SystemComponentBridge`
- [`examples/sandbox_compliance/`](examples/sandbox_compliance/) — Sandbox constraint demonstration

## Documentation

- [Getting Started](docs/GETTING_STARTED.md) — Build a minimal skill from scratch
- [Fine-Tuned Adapters](docs/ADAPTERS.md) — Declare, upload, load, and manage LoRA/QLoRA adapters
- [Deployment Guide](docs/DEPLOYMENT.md) — Compile, deploy, and verify your skill
- [Compatibility](COMPATIBILITY.md) — SDK ↔ agent version mapping
- [Changelog](CHANGELOG.md) — Release history

## Architecture

This SDK is a thin re-export layer over the `lifesavor-agent` crate. Types like `ProviderManifest`, `ErrorChain`, and `ToolSchema` are the identical Rust types from the agent — no duplication, no drift.

See the [pluggable integration architecture spec](../../.kiro/specs/agent-pluggable-integrations/) for detailed design context.

## License

[MIT](LICENSE)

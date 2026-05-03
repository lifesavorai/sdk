# Life Savor Developer SDK

The Life Savor Developer SDK provides tools and libraries for building components (Models, Assistants, Skills, and System components) for the Life Savor platform.

## Quick Links

- [Getting Started](./GETTING_STARTED.md) — Create, configure, and publish your first component
- [Architecture](./ARCHITECTURE.md) — Platform architecture overview
- [Build Configuration](./BUILD_CONFIG.md) — `lifesavor-build.yml` reference
- [Deploy Keys](./DEPLOY_KEYS.md) — SSH deploy key setup for system components
- [Security Scanning](./SECURITY_SCANNING.md) — Build security scanning details
- [Troubleshooting](./TROUBLESHOOTING.md) — Common issues and solutions
- [Migration Guide](./MIGRATION.md) — Migrating from previous SDK versions

## Installation

### CLI Tool

```bash
# macOS / Linux
brew install lifesavor/tap/lsai-cli

# From source
cd developer/cli && cargo install --path .

# First-time setup
lsai-cli setup
lsai-cli whoami
```

### Rust SDK

```toml
[dependencies]
# For system components:
lifesavor-system-sdk = "0.5.0"

# For model components:
lifesavor-model-sdk = "0.5.0"
```

## Quick Start

```bash
# 1. Create a component
lsai-cli components create --name my-skill --type skill --language rust

# 2. Set required metadata (description, category, tags)
lsai-cli components update <id> \
  --description "My awesome skill" \
  --category General \
  --tags automation,utility

# 3. Connect your repo
lsai-cli components connect --component <id> --repo-url https://github.com/you/my-skill

# 4. Submit for review
lsai-cli components submit <id>

# 5. Build
lsai-cli builds submit --component <id> --all-platforms

# 6. Publish (after QA approval)
lsai-cli components publish --component <id> --version 1.0.0 --notes CHANGELOG.md
```

See [Getting Started](./GETTING_STARTED.md) for the full walkthrough.

## Component Types

| Type      | Description                      | Language |
| --------- | -------------------------------- | -------- |
| Model     | AI/ML models for inference       | Rust     |
| Assistant | Conversational AI assistants     | Python   |
| Skill     | Reusable capabilities and tools  | Any      |
| System    | Platform-level system components | Rust     |
| Agent     | Autonomous agent configurations  | Any      |

## Authentication

```bash
lsai-cli setup         # First-time setup with API key
lsai-cli whoami         # Verify identity
lsai-cli auth status    # Check auth state
```

API keys use the `lsk_` prefix and are managed at [developer.lifesavor.ai/api-keys](https://developer.lifesavor.ai/api-keys).

## Support

- [Developer Portal](https://developer.lifesavor.ai)
- [SDK Documentation](https://developer.lifesavor.ai/documentation)
- [Support Cases](https://developer.lifesavor.ai/support)

# Getting Started

## Prerequisites

- Node.js 20+ or Rust 1.75+ (depending on component type)
- Git
- `lsai-cli` installed ([installation guide](./README.md#installation))

## 1. Create a Developer Account

Visit [developer.lifesavor.ai](https://developer.lifesavor.ai) and sign up with Google OAuth or email. Complete the developer agreement and profile setup.

## 2. Install and Configure the CLI

```bash
# macOS
brew install lifesavor/tap/lsai-cli

# From source
cd developer/cli && cargo install --path .

# First-time setup — paste your API key (lsk_ prefix) from the portal
lsai-cli setup

# Verify
lsai-cli whoami
```

## 3. Create a Component

```bash
lsai-cli components create \
  --name "my-skill" \
  --type skill \
  --language rust
```

This creates a draft component and returns a component ID. Save it — you'll need it for the next steps.

## 4. Set Required Metadata

You **must** set a description, category, and tags before submitting for review:

```bash
lsai-cli components update <component-id> \
  --description "A useful skill for Life Savor agents" \
  --category General \
  --tags automation,utility,productivity
```

Valid categories: `General`, `Code`, `Embedding`, `Vision`, `Specialized`

## 5. Set Up Build Configuration

Create `lifesavor-build.yml` in your repository root:

```yaml
version: 1
component:
  type: skill
  name: my-skill
build:
  language: rust
  command: cargo build --release
  artifact: target/release/my-skill
```

See [Build Configuration Reference](./BUILD_CONFIG.md) for the full schema.

## 6. Connect GitHub Repository

```bash
lsai-cli components connect \
  --component <component-id> \
  --repo-url https://github.com/you/my-skill
```

## 7. Submit for Review

Submission puts your component in the QA review queue. Builds happen **after** submission, not before.

```bash
lsai-cli components submit <component-id>
```

If this fails, check the error message — it will tell you exactly what's missing (description, category, or tags).

## 8. Trigger Builds

```bash
# Build for all configured platforms
lsai-cli builds submit --component <component-id> --all-platforms

# Or build for a specific platform
lsai-cli builds submit --component <component-id> --platform linux-x86_64
```

## 9. Publish (After QA Approval)

Once QA approves your component and you have at least one successful build:

```bash
lsai-cli components publish \
  --component <component-id> \
  --version 1.0.0 \
  --notes CHANGELOG.md
```

## Summary: The Full Flow

```
create → set metadata → connect repo → submit → build → QA approval → publish
```

| Step | Command | Required Before |
|------|---------|-----------------|
| Create draft | `components create` | — |
| Set metadata | `components update --description --category --tags` | Submit |
| Connect repo | `components connect` | Build |
| Submit for review | `components submit` | Build |
| Trigger build | `builds submit` | Publish |
| Publish version | `components publish` | — |

## Local Development

```bash
# Install a local component for testing
lsai-cli component install ./my-component

# Validate build config
lsai-cli config validate

# Check component status
lsai-cli components status <component-id>
```

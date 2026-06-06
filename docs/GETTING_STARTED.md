# Getting Started

## Prerequisites

- Node.js 20+ or Rust 1.75+ (depending on component type)
- Git
- `lsai-cli` installed ([installation guide](./README.md#installation) or [Downloads page](https://developer.lifesavor.ai/downloads))

## 1. Create a Developer Account

Visit [developer.lifesavor.ai](https://developer.lifesavor.ai) and sign up with Google OAuth or email. Complete the developer agreement and profile setup.

## 2. Install and Configure the CLI

Install `lsai-cli` for your platform. For all platforms and package formats, visit the [Downloads page](https://developer.lifesavor.ai/downloads).

```bash
# macOS (Homebrew)
brew tap Life-Savor-AI/tap && brew install lsai-cli

# Linux — Debian/Ubuntu (APT repository)
curl -fsSL https://download.lifesavor.ai/lsai-cli/apt/setup.sh | sudo bash
sudo apt install lsai-cli

# Linux — Debian/Ubuntu (dpkg direct install, x86_64)
curl -LO https://download.lifesavor.ai/lsai-cli/latest/x86_64-unknown-linux-gnu/lsai-cli_amd64.deb
sudo dpkg -i lsai-cli_amd64.deb

# Linux — Fedora/RHEL/Amazon Linux (DNF/YUM repository)
curl -fsSL https://download.lifesavor.ai/lsai-cli/yum/setup.sh | sudo bash
sudo dnf install lsai-cli

# Linux — Fedora/RHEL/Amazon Linux (rpm direct install, x86_64)
curl -LO https://download.lifesavor.ai/lsai-cli/latest/x86_64-unknown-linux-gnu/lsai-cli.x86_64.rpm
sudo rpm -i lsai-cli.x86_64.rpm

# Windows (winget)
winget install LifeSavorAI.lsai-cli

# From source (any platform with Rust 1.75+)
cd developer/cli && cargo install --path .

# First-time setup — paste your API key (lsk_ prefix) from the portal
lsai-cli setup

# Verify
lsai-cli whoami
```

> **Note:** Linux arm64 packages are also available. See the [Downloads page](https://developer.lifesavor.ai/downloads) for arm64 DEB, RPM, and tarball links.

## 3. Create a Component

```bash
lsai-cli components create \
  --name "my-skill" \
  --type skill \
  --language rust
```

This creates a draft component and returns a component ID. Save it — you'll need it for the next steps.

## 4. Set Required Metadata

You **must** set a description, category, tags, compatibility, and license before submitting for review:

```bash
lsai-cli components update <component-id> \
  --description "A useful skill for Life Savor agents" \
  --category Productivity \
  --tags automation,utility,productivity \
  --compatibility linux,macos,windows \
  --license MIT
```

Valid categories depend on your component type (the portal shows the right ones automatically).

Valid compatibility platforms: `linux`, `macos`, `windows`, `ios`, `android`, `tvos`

Valid licenses: `MIT`, `Apache-2.0`, `GPL-3.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `MPL-2.0`, `LGPL-3.0`, `Proprietary`, `Custom`

You can also upload an icon:

```bash
lsai-cli components update <component-id> --icon logo.png
```

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

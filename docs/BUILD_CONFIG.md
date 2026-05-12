# Build Configuration Reference

## `lifesavor-build.yml`

The build configuration file must be placed at the root of your repository.

### Schema

```yaml
version: 1 # Required: schema version
component:
  type: skill # Required: model | assistant | skill | system
  name: my-component # Required: component name
build:
  language: rust # Required: rust | go | python | node | cpp
  command: cargo build --release # Required: build command
  artifact: target/release/my-component # Required: path to the built artifact
  targets: # Optional: multi-platform targets
    - platform: linux
      arch: x86_64
    - platform: macos
      arch: aarch64
security:
  skip_scan: false # Optional: skip security scan (global-admin only)
```

### Rust Components

Rust components are compiled as shared libraries (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows) that the agent loads at runtime.

Your `Cargo.toml` **must** include:

```toml
[lib]
crate-type = ["cdylib"]
```

Without this, `cargo build --release` produces a `.rlib` (Rust-only static library) instead of the shared library the platform expects.

The `build.artifact` field should point to the shared library output:

```yaml
build:
  language: rust
  command: cargo build --release
  artifact: target/release/libmy_component.so
```

Note: Rust converts hyphens to underscores in library names. A crate named `my-component` produces `libmy_component.so`.

### Supported Languages

| Language | Build Tool | Dependency Audit |
| -------- | ---------- | ---------------- |
| Rust     | cargo      | `cargo audit`    |
| Go       | go build   | `govulncheck`    |
| Python   | pip/poetry | `pip-audit`      |
| Node.js  | npm/yarn   | `npm audit`      |
| C/C++    | make/cmake | (manual)         |

### Artifact Size Limits

| Component Type | Max Size |
| -------------- | -------- |
| System         | 500 MB   |
| Model          | 200 MB   |
| Assistant      | 100 MB   |
| Skill          | 100 MB   |

### Build Secrets

Set build secrets via the CLI or portal. They are injected as environment variables during build:

```bash
lsai-cli secrets set --component-id <id> --key API_KEY --value <value>
```

### Scheduled Builds

Configure scheduled builds in the portal: daily, weekly, or custom cron expression (5-field format).

### Validation

```bash
lsai-cli config validate              # Validate local config
lsai-cli config validate path/to/file # Validate specific file
```

# Troubleshooting

## Submission Errors

### "Component does not meet submission requirements"

The most common error. The API returns the specific missing requirements. Check:

```bash
# Set description
lsai-cli components update <id> --description "Your component description"

# Set category (General, Code, Embedding, Vision, Specialized)
lsai-cli components update <id> --category General

# Set tags (comma-separated)
lsai-cli components update <id> --tags tag1,tag2,tag3
```

All three (description, category, tags) are required before `lsai-cli components submit` will succeed.

### "Invalid status transition"

You're trying to perform an action that isn't valid for the component's current status. Check the current state:

```bash
lsai-cli components status <id>
```

Common causes:
- Trying to submit a component that's already in review
- Trying to publish without QA approval
- Trying to build a deleted component

## Build Errors

### Build Fails with "Config Invalid"

Ensure your `lifesavor-build.yml` matches the schema. Validate locally:

```bash
lsai-cli config validate
```

### Build Timeout (30 minutes)

Builds are automatically terminated after 30 minutes. Optimize your build:

- Use build caching
- Reduce dependencies
- Use pre-built base images

### Security Scan Failures

Check the security scan report in the build details:

```bash
lsai-cli builds logs <build-id>
```

Fix critical/high findings before re-triggering.

## Authentication Errors

### "Request failed" or Empty Error Messages

If you see generic "Request failed" errors, update your CLI — older versions don't parse error details correctly. The latest version shows the full error message and missing requirements.

```bash
# Update CLI
brew upgrade lsai-cli
# or rebuild from source
cd developer/cli && cargo install --path .
```

### Authentication Expired

```bash
lsai-cli whoami        # Check if authenticated
lsai-cli setup         # Re-configure with a new API key
```

## Rate Limiting

### Rate Limited (429)

Wait for the `Retry-After` period. Check your rate limit dashboard in the portal at [developer.lifesavor.ai/settings](https://developer.lifesavor.ai/settings).

## Deploy Key Issues

- Verify the key is added to your GitHub repository
- Ensure SSH format URL (`git@github.com:org/repo.git`)
- Check key permissions (read access required)

See [Deploy Keys](./DEPLOY_KEYS.md) for setup instructions.

## Diagnostics

Run the built-in diagnostics command:

```bash
lsai-cli diagnostics
lsai-cli diagnostics --json  # Machine-readable output
```

## Getting Help

- [Developer Portal Support](https://developer.lifesavor.ai/support)
- [SDK Documentation](https://developer.lifesavor.ai/documentation)

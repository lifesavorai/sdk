# Life Savor JavaScript SDK

JavaScript SDK modules for building Life Savor skills and configuration interfaces.

## Modules

| Module | Status | Description |
|--------|--------|-------------|
| `skill-config-sdk.js` | **Active** | Configuration schema, setup steps, validation helpers (API-routed) |
| `config-sdk.js` | **Deprecated** | Legacy iframe + WebSocket config page client |

## skill-config-sdk.js

Pure utility module for defining configuration schemas, setup steps, and
validation command handlers. No WebSocket or iframe dependency — all
configuration flows through the platform's REST API endpoints.

### Quick Start

```javascript
const {
  createConfigSchema,
  createSetupStep,
  validateConfigValues,
  handleValidation,
  createValidationResponse,
  invalidCredentials,
} = require('./skill-config-sdk');

// Define a configuration schema
const schema = createConfigSchema({
  properties: {
    api_key: {
      type: 'string',
      title: 'API Key',
      description: 'Your service API key',
      'x-secret': true,
    },
    region: {
      type: 'string',
      title: 'Region',
      description: 'Deployment region',
      enum: ['us-east-1', 'eu-west-1'],
      default: 'us-east-1',
    },
    max_retries: {
      type: 'integer',
      title: 'Max Retries',
      description: 'Maximum number of retry attempts',
      default: 3,
    },
  },
  required: ['api_key'],
});

// Define setup steps
const step1 = createSetupStep({
  stepId: 'credentials',
  title: 'API Credentials',
  description: 'Enter your API key to connect the service',
  fields: ['api_key'],
  validationCommand: 'validate_credentials',
});

const step2 = createSetupStep({
  stepId: 'preferences',
  title: 'Preferences',
  description: 'Configure your deployment preferences',
  fields: ['region', 'max_retries'],
});

// Validate user-provided values
const result = validateConfigValues(schema, {
  api_key: 'sk-abc123',
  region: 'us-east-1',
  max_retries: 3,
});
console.log(result.valid);  // true
console.log(result.errors); // []
```

### API Reference

#### `CONFIG_FIELD_TYPES`

Array of supported field types: `'string'`, `'number'`, `'boolean'`, `'integer'`, `'array'`.

#### `createConfigSchema(definition)`

Creates a validated ConfigSchema object. Throws if any property is missing
`type`, `title`, or `description`.

**Parameters:**
- `definition.properties` — Map of field name to field definition
- `definition.required` — Optional array of required field names

**Returns:** A ConfigSchema object with `$schema`, `type`, `properties`, and `required` fields.

#### `createSetupStep(stepDef)`

Creates a validated SetupStep object with snake_case keys matching the Rust types.

**Parameters:**
- `stepDef.stepId` — Unique step identifier
- `stepDef.title` — Display title (3–100 characters)
- `stepDef.description` — Description text (10–500 characters)
- `stepDef.fields` — Array of config field names for this step
- `stepDef.validationCommand` — Optional validation operation name

**Returns:** A SetupStep object with `step_id`, `title`, `description`, `fields`, and optional `validation_command`.

#### `validateConfigValues(schema, values)`

Validates a values object against a ConfigSchema.

**Returns:** `{ valid: boolean, errors: Array<{ field: string, message: string }> }`

#### `parseValidationRequest(input)`

Parses a JSON-RPC validation request from stdin.

**Returns:** `{ stepId, values, context: { skillId, isReconfigure } }`

#### `createValidationResponse(status, message?, data?)`

Creates a formatted validation response object.

#### `handleValidation(handlerFn)`

Wraps a developer's validation handler for stdin/stdout JSON-RPC. Reads stdin,
parses the request, calls the handler, and writes the response to stdout.

```javascript
// validate_credentials.js — invoked by the agent as a validation_command
const { handleValidation, createValidationResponse, invalidCredentials } = require('./skill-config-sdk');

handleValidation(async (request) => {
  const { values } = request;
  const apiKey = values.api_key;

  // Test the API key against the external service
  try {
    const response = await fetch('https://api.example.com/verify', {
      headers: { Authorization: `Bearer ${apiKey}` },
    });
    if (!response.ok) {
      return invalidCredentials('API key is invalid or expired');
    }
    return createValidationResponse('success', 'Credentials verified');
  } catch (err) {
    return invalidCredentials('Could not verify credentials: ' + err.message);
  }
});
```

#### Error Helpers

- `invalidCredentials(msg?)` — Pre-formatted failure for bad credentials
- `connectionFailed(msg?)` — Pre-formatted failure for connection issues
- `validationTimeout(msg?)` — Pre-formatted failure for timeouts

### Configuration Schema Guide

This section covers defining schemas with all supported field types, handling
validation errors, and structuring multi-step workflows.

#### Supported Field Types

| Type | JS type check | UI control |
|------|--------------|------------|
| `string` | `typeof val === 'string'` | Text input (password for `x-secret`) |
| `number` | `typeof val === 'number'` | Number input (decimals allowed) |
| `integer` | `Number.isInteger(val)` | Number input (whole numbers) |
| `boolean` | `typeof val === 'boolean'` | Toggle switch |
| `array` | `Array.isArray(val)` | Tag input / multi-select |

#### All Field Types Example

```javascript
const { createConfigSchema } = require('./skill-config-sdk');

const schema = createConfigSchema({
  properties: {
    api_key: {
      type: 'string',
      title: 'API Key',
      description: 'Your service API key',
      'x-secret': true,
    },
    webhook_url: {
      type: 'string',
      title: 'Webhook URL',
      description: 'URL to receive event notifications',
    },
    confidence_threshold: {
      type: 'number',
      title: 'Confidence Threshold',
      description: 'Minimum confidence score (0.0 to 1.0)',
      default: 0.8,
    },
    max_retries: {
      type: 'integer',
      title: 'Max Retries',
      description: 'Maximum number of retry attempts',
      default: 3,
    },
    enabled: {
      type: 'boolean',
      title: 'Enabled',
      description: 'Whether the integration is active',
      default: true,
    },
    tags: {
      type: 'array',
      title: 'Tags',
      description: 'Labels for categorizing events',
      items: { type: 'string' },
      default: ['production'],
    },
  },
  required: ['api_key', 'webhook_url'],
});
```

#### Handling Validation Errors

`validateConfigValues` checks required fields, type correctness, and enum
constraints. The `errors` array contains one entry per invalid field:

```javascript
const { validateConfigValues } = require('./skill-config-sdk');

const result = validateConfigValues(schema, {
  // api_key missing (required)
  webhook_url: 12345,       // wrong type — should be string
  max_retries: 3.5,         // not an integer
  enabled: 'yes',           // wrong type — should be boolean
});

console.log(result.valid);  // false
console.log(result.errors);
// [
//   { field: 'api_key', message: 'Required field "api_key" is missing' },
//   { field: 'webhook_url', message: 'Expected type "string" ...' },
//   { field: 'max_retries', message: 'Expected type "integer" ...' },
//   { field: 'enabled', message: 'Expected type "boolean" ...' },
// ]
```

#### Multi-Step Workflow Pattern

Group related fields into steps so users see a focused wizard instead of a
single long form:

```javascript
const { createSetupStep } = require('./skill-config-sdk');

// Step 1: Credentials (with server-side validation)
const credentialsStep = createSetupStep({
  stepId: 'credentials',
  title: 'API Credentials',
  description: 'Enter your API key to connect the service',
  fields: ['api_key'],
  validationCommand: 'validate_credentials',
});

// Step 2: Connection settings
const connectionStep = createSetupStep({
  stepId: 'connection',
  title: 'Connection Settings',
  description: 'Configure the webhook and retry behavior',
  fields: ['webhook_url', 'max_retries'],
});

// Step 3: Preferences (no validation needed)
const preferencesStep = createSetupStep({
  stepId: 'preferences',
  title: 'Preferences',
  description: 'Set your notification and tagging preferences',
  fields: ['confidence_threshold', 'enabled', 'tags'],
});
```

Each field should appear in exactly one step. The platform renders the steps
in order and advances automatically on successful submission.

---

## Migration Guide: config-sdk.js → skill-config-sdk.js

The legacy `config-sdk.js` module used iframe-hosted config pages that
communicated with the agent via WebSocket through the connect service. This
approach is **deprecated** in favor of the API-routed setup wizard powered by
`skill-config-sdk.js`.

### Why Migrate?

| | Legacy (`config-sdk.js`) | New (`skill-config-sdk.js`) |
|---|---|---|
| Communication | WebSocket via iframe | REST API (platform-managed) |
| UI | Custom HTML in iframe | Platform-rendered setup wizard |
| Validation | Client-side only | Server-side via `validation_command` |
| Secret handling | Plaintext in WebSocket messages | Encrypted via Vault |
| Multi-step | Manual implementation | Declarative `setup_steps` |
| Mobile support | None (iframe) | Native iOS + web |

### Step-by-Step Migration

#### 1. Replace iframe config page with a schema definition

**Before (config-sdk.js):**
```javascript
const sdk = new LifeSavorConfigSDK({ wsUrl, componentId });
await sdk.connect();
const config = await sdk.getConfig();
// ... render custom HTML form ...
await sdk.updateConfig(newValues);
```

**After (skill-config-sdk.js):**
```javascript
const { createConfigSchema } = require('./skill-config-sdk');

// Declare your schema in skill.json — no runtime code needed for the UI
const schema = createConfigSchema({
  properties: {
    api_key: {
      type: 'string',
      title: 'API Key',
      description: 'Your API key',
      'x-secret': true,
    },
  },
  required: ['api_key'],
});
```

The platform renders the setup wizard automatically from your `config_schema`
in `skill.json`. You no longer need to build a custom config page.

#### 2. Add setup steps to your skill manifest

```json
{
  "config_schema": { ... },
  "setup_steps": [
    {
      "step_id": "credentials",
      "title": "API Credentials",
      "description": "Enter your API key to connect the service",
      "fields": ["api_key"],
      "validation_command": "validate_credentials"
    }
  ]
}
```

#### 3. Implement validation commands (optional)

If your config page had client-side validation, move it to a
`validation_command` script:

```javascript
const { handleValidation, createValidationResponse, invalidCredentials } = require('./skill-config-sdk');

handleValidation(async (request) => {
  // Validate the API key server-side
  const isValid = await checkApiKey(request.values.api_key);
  if (!isValid) return invalidCredentials();
  return createValidationResponse('success');
});
```

#### 4. Remove iframe references

- Delete your `config.html` file (or equivalent iframe page)
- Remove the `config_page_url` field from your `skill.json`
- Remove the `config-sdk.js` import from your skill

#### 5. Test with the Developer CLI

```bash
lsai-cli skill config validate
lsai-cli skill config preview
lsai-cli skill config test-validation --step credentials --values '{"api_key":"sk-test"}'
```

### JSON Structure Compatibility

The `skill-config-sdk.js` module produces JSON structures identical to the
Rust SDK types in `developer/sdk/rust/agent/src/skill_config.rs`. This ensures
cross-SDK consistency — skill manifests are interchangeable regardless of
which SDK was used to create them.

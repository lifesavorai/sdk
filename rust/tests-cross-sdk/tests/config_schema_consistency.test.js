/**
 * Cross-SDK consistency tests for Life Savor config schema types.
 *
 * Feature: skill-setup-workflow, Property 14: Cross-SDK JSON structural equality
 *
 * Verifies that the JS SDK (`createConfigSchema`, `createSetupStep`) produces
 * JSON structures identical to what the Rust SDK (`ConfigSchema`, `SetupStep`)
 * serializes via serde_json. Since shelling out to a Rust binary is not
 * practical in every test environment, we define reference JSON structures
 * that match the Rust serde output (field names, nesting, omission rules)
 * and compare the JS SDK output against them.
 *
 * The Rust canonical types live in `developer/sdk/rust/agent/src/skill_config.rs`.
 * The JS SDK lives in `developer/sdk/js/skill-config-sdk.js`.
 *
 * **Validates: Requirements 9.1, 9.2, 9.3, 9.4**
 */

import { describe, test, expect } from 'vitest';
import * as fc from 'fast-check';

const {
  CONFIG_FIELD_TYPES,
  createConfigSchema,
  createSetupStep,
} = require('../../../js/skill-config-sdk');

// ---------------------------------------------------------------------------
// Helpers: Build a Rust-equivalent reference JSON from a schema definition
// ---------------------------------------------------------------------------

/**
 * Build the reference JSON that the Rust `ConfigSchema` would produce via
 * `serde_json::to_value`. This encodes the exact serialization rules from
 * the Rust struct:
 *
 * - `$schema` always present (default: draft 2020-12)
 * - `type` always "object"
 * - `properties` is a BTreeMap → keys sorted alphabetically
 * - `required` is a Vec<String> (empty array when no required fields)
 * - Each field has: `type`, `title`, `description`
 * - `default` omitted when None (skip_serializing_if = "Option::is_none")
 * - `x-secret` omitted when false (skip_serializing_if = "is_false")
 * - `items` omitted when None
 * - `enum` omitted when None
 */
function buildRustReferenceSchema(definition) {
  const sortedKeys = Object.keys(definition.properties).sort();
  const properties = {};

  for (const key of sortedKeys) {
    const prop = definition.properties[key];
    const field = {
      type: prop.type,
      title: prop.title,
      description: prop.description,
    };

    // Rust: #[serde(default, skip_serializing_if = "Option::is_none")]
    if (prop.default !== undefined) {
      field.default = prop.default;
    }

    // Rust: #[serde(rename = "x-secret", default, skip_serializing_if = "is_false")]
    if (prop['x-secret'] === true) {
      field['x-secret'] = true;
    }

    // Rust: #[serde(default, skip_serializing_if = "Option::is_none")]
    if (prop.items !== undefined && prop.items !== null) {
      field.items = prop.items;
    }

    // Rust: #[serde(rename = "enum", default, skip_serializing_if = "Option::is_none")]
    if (prop.enum !== undefined && prop.enum !== null) {
      field.enum = prop.enum;
    }

    properties[key] = field;
  }

  return {
    $schema: 'https://json-schema.org/draft/2020-12/schema',
    type: 'object',
    properties,
    required: Array.isArray(definition.required) ? definition.required.slice() : [],
  };
}

/**
 * Build the reference JSON that the Rust `SetupStep` would produce via
 * `serde_json::to_value`. Encodes the exact serialization rules:
 *
 * - `step_id` (snake_case, not camelCase)
 * - `title`, `description`, `fields` always present
 * - `validation_command` omitted when None (skip_serializing_if = "Option::is_none")
 */
function buildRustReferenceStep(stepDef) {
  const step = {
    step_id: stepDef.stepId,
    title: stepDef.title,
    description: stepDef.description,
    fields: stepDef.fields.slice(),
  };

  if (stepDef.validationCommand && typeof stepDef.validationCommand === 'string') {
    step.validation_command = stepDef.validationCommand;
  }

  return step;
}

/**
 * Normalize a JSON object for structural comparison: sort object keys
 * recursively so that key ordering differences don't cause false negatives.
 */
function sortKeysDeep(obj) {
  if (Array.isArray(obj)) {
    return obj.map(sortKeysDeep);
  }
  if (obj !== null && typeof obj === 'object') {
    const sorted = {};
    for (const key of Object.keys(obj).sort()) {
      sorted[key] = sortKeysDeep(obj[key]);
    }
    return sorted;
  }
  return obj;
}

// ---------------------------------------------------------------------------
// Arbitraries — generators for valid schema definitions
// ---------------------------------------------------------------------------

/** Non-empty alphanumeric identifier for field names / step IDs. */
const arbFieldName = fc.stringMatching(/^[a-z][a-z0-9_]{0,19}$/);

/** Field type from the supported set. */
const arbFieldType = fc.constantFrom(...CONFIG_FIELD_TYPES);

/** Non-empty title (1-80 chars). */
const arbTitle = fc
  .string({ minLength: 1, maxLength: 80 })
  .filter((s) => s.trim().length > 0);

/** Non-empty description (1-200 chars). */
const arbDescription = fc
  .string({ minLength: 1, maxLength: 200 })
  .filter((s) => s.trim().length > 0);

/** Generate an optional default value appropriate for a given type. */
function arbDefaultForType(fieldType) {
  switch (fieldType) {
    case 'string':
      return fc.option(fc.string({ maxLength: 50 }), { nil: undefined });
    case 'number':
      return fc.option(
        fc.double({ min: -1e6, max: 1e6, noNaN: true, noDefaultInfinity: true }),
        { nil: undefined }
      );
    case 'integer':
      return fc.option(fc.integer({ min: -1000000, max: 1000000 }), { nil: undefined });
    case 'boolean':
      return fc.option(fc.boolean(), { nil: undefined });
    case 'array':
      return fc.option(fc.array(fc.string({ maxLength: 20 }), { maxLength: 5 }), {
        nil: undefined,
      });
    default:
      return fc.constant(undefined);
  }
}

/** Generate a single field definition object. */
const arbFieldDefinition = arbFieldType.chain((fieldType) =>
  fc
    .record({
      type: fc.constant(fieldType),
      title: arbTitle,
      description: arbDescription,
      defaultVal: arbDefaultForType(fieldType),
      secret: fc.boolean(),
    })
    .map(({ type: fType, title, description, defaultVal, secret }) => {
      const def = { type: fType, title, description };
      if (defaultVal !== undefined) {
        def.default = defaultVal;
      }
      if (secret) {
        def['x-secret'] = true;
      }
      // For array fields, add items schema (matching Rust's Box<ConfigFieldDefinition>)
      if (fType === 'array') {
        def.items = { type: 'string', title: '', description: '' };
      }
      return def;
    })
);

/** Generate a valid schema definition with 1-6 properties. */
const arbSchemaDefinition = fc
  .array(fc.tuple(arbFieldName, arbFieldDefinition), { minLength: 1, maxLength: 6 })
  .chain((entries) => {
    const seen = new Set();
    const unique = entries.filter(([name]) => {
      if (seen.has(name)) return false;
      seen.add(name);
      return true;
    });
    if (unique.length === 0) return fc.constant(null);

    const properties = {};
    const allNames = [];
    for (const [name, def] of unique) {
      properties[name] = def;
      allNames.push(name);
    }

    return fc
      .subarray(allNames, { minLength: 0, maxLength: allNames.length })
      .map((required) => ({ properties, required }));
  })
  .filter((def) => def !== null);

/** Step title: 3-100 chars. */
const arbStepTitle = fc
  .string({ minLength: 3, maxLength: 60 })
  .filter((s) => s.trim().length >= 3);

/** Step description: 10-500 chars. */
const arbStepDescription = fc
  .string({ minLength: 10, maxLength: 100 })
  .filter((s) => s.trim().length >= 10);

/** Optional validation command. */
const arbValidationCommand = fc.option(
  fc.stringMatching(/^[a-z][a-z0-9_]{2,29}$/),
  { nil: undefined }
);

// ---------------------------------------------------------------------------
// Property 14: Cross-SDK JSON structural equality — ConfigSchema
// ---------------------------------------------------------------------------

describe('Property 14: Cross-SDK JSON structural equality', () => {
  // Feature: skill-setup-workflow, Property 14: Cross-SDK JSON structural equality
  // **Validates: Requirements 9.1, 9.2, 9.3, 9.4**

  test('JS createConfigSchema output matches Rust serde reference for arbitrary schemas', () => {
    fc.assert(
      fc.property(arbSchemaDefinition, (definition) => {
        const jsOutput = createConfigSchema(definition);
        const rustReference = buildRustReferenceSchema(definition);

        // Normalize key ordering for comparison
        const jsNormalized = sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)));
        const rustNormalized = sortKeysDeep(rustReference);

        expect(jsNormalized).toEqual(rustNormalized);
      }),
      { numRuns: 100 }
    );
  });

  // -------------------------------------------------------------------------
  // Example-based: all field types
  // -------------------------------------------------------------------------

  test('schema with all field types matches Rust reference', () => {
    const definition = {
      properties: {
        api_key: {
          type: 'string',
          title: 'API Key',
          description: 'Your service API key',
          'x-secret': true,
        },
        score: {
          type: 'number',
          title: 'Score',
          description: 'A numeric score',
          default: 0.0,
        },
        active: {
          type: 'boolean',
          title: 'Active',
          description: 'Is active',
          default: true,
        },
        count: {
          type: 'integer',
          title: 'Count',
          description: 'Item count',
          default: 42,
        },
        tags: {
          type: 'array',
          title: 'Tags',
          description: 'Tag list',
          items: { type: 'string', title: '', description: '' },
          default: ['a', 'b'],
        },
      },
      required: ['api_key'],
    };

    const jsOutput = createConfigSchema(definition);
    const rustReference = buildRustReferenceSchema(definition);

    const jsNormalized = sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)));
    const rustNormalized = sortKeysDeep(rustReference);

    expect(jsNormalized).toEqual(rustNormalized);

    // Verify specific structural details
    expect(jsOutput.$schema).toBe('https://json-schema.org/draft/2020-12/schema');
    expect(jsOutput.type).toBe('object');
    expect(jsOutput.required).toEqual(['api_key']);
    expect(jsOutput.properties.api_key['x-secret']).toBe(true);
    expect(jsOutput.properties.score.default).toBe(0.0);
    expect(jsOutput.properties.active.default).toBe(true);
    expect(jsOutput.properties.count.default).toBe(42);
    expect(jsOutput.properties.tags.items).toEqual({ type: 'string', title: '', description: '' });
  });

  // -------------------------------------------------------------------------
  // Example-based: x-secret annotation
  // -------------------------------------------------------------------------

  test('x-secret annotation present only when true', () => {
    const definition = {
      properties: {
        secret_field: {
          type: 'string',
          title: 'Secret',
          description: 'A secret value',
          'x-secret': true,
        },
        normal_field: {
          type: 'string',
          title: 'Normal',
          description: 'A normal value',
        },
      },
    };

    const jsOutput = createConfigSchema(definition);

    // x-secret present on secret field
    expect(jsOutput.properties.secret_field['x-secret']).toBe(true);

    // x-secret NOT present on normal field (Rust skips when false)
    expect(jsOutput.properties.normal_field['x-secret']).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // Example-based: enum values
  // -------------------------------------------------------------------------

  test('enum values preserved in schema', () => {
    const definition = {
      properties: {
        units: {
          type: 'string',
          title: 'Units',
          description: 'Temperature units',
          enum: ['metric', 'imperial', 'kelvin'],
        },
      },
    };

    const jsOutput = createConfigSchema(definition);
    const rustReference = buildRustReferenceSchema(definition);

    expect(sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)))).toEqual(
      sortKeysDeep(rustReference)
    );
    expect(jsOutput.properties.units.enum).toEqual(['metric', 'imperial', 'kelvin']);
  });

  // -------------------------------------------------------------------------
  // Example-based: required fields
  // -------------------------------------------------------------------------

  test('required fields array matches Rust reference', () => {
    const definition = {
      properties: {
        api_key: { type: 'string', title: 'API Key', description: 'Key', 'x-secret': true },
        location: { type: 'string', title: 'Location', description: 'City' },
        interval: { type: 'integer', title: 'Interval', description: 'Minutes' },
      },
      required: ['api_key', 'location'],
    };

    const jsOutput = createConfigSchema(definition);
    const rustReference = buildRustReferenceSchema(definition);

    expect(jsOutput.required).toEqual(['api_key', 'location']);
    expect(sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)))).toEqual(
      sortKeysDeep(rustReference)
    );
  });

  // -------------------------------------------------------------------------
  // Example-based: default values
  // -------------------------------------------------------------------------

  test('default values preserved for all types', () => {
    const definition = {
      properties: {
        name: { type: 'string', title: 'Name', description: 'User name', default: 'anonymous' },
        retries: { type: 'integer', title: 'Retries', description: 'Retry count', default: 3 },
        rate: { type: 'number', title: 'Rate', description: 'Rate value', default: 1.5 },
        verbose: { type: 'boolean', title: 'Verbose', description: 'Verbose mode', default: false },
        items: {
          type: 'array',
          title: 'Items',
          description: 'Item list',
          items: { type: 'string', title: '', description: '' },
          default: ['x'],
        },
      },
    };

    const jsOutput = createConfigSchema(definition);
    const rustReference = buildRustReferenceSchema(definition);

    expect(sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)))).toEqual(
      sortKeysDeep(rustReference)
    );

    expect(jsOutput.properties.name.default).toBe('anonymous');
    expect(jsOutput.properties.retries.default).toBe(3);
    expect(jsOutput.properties.rate.default).toBe(1.5);
    expect(jsOutput.properties.verbose.default).toBe(false);
    expect(jsOutput.properties.items.default).toEqual(['x']);
  });

  // -------------------------------------------------------------------------
  // Property 14: Cross-SDK JSON structural equality — SetupStep
  // -------------------------------------------------------------------------

  test('JS createSetupStep output matches Rust serde reference for arbitrary steps', () => {
    const arbStepDef = fc.record({
      stepId: arbFieldName,
      title: arbStepTitle,
      description: arbStepDescription,
      fields: fc.array(arbFieldName, { minLength: 0, maxLength: 5 }),
      validationCommand: arbValidationCommand,
    });

    fc.assert(
      fc.property(arbStepDef, (stepDef) => {
        const jsOutput = createSetupStep(stepDef);
        const rustReference = buildRustReferenceStep(stepDef);

        const jsNormalized = sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)));
        const rustNormalized = sortKeysDeep(rustReference);

        expect(jsNormalized).toEqual(rustNormalized);
      }),
      { numRuns: 100 }
    );
  });

  // -------------------------------------------------------------------------
  // Example-based: multi-step workflow with validation_command
  // -------------------------------------------------------------------------

  test('multi-step workflow with validation_command matches Rust reference', () => {
    const steps = [
      {
        stepId: 'credentials',
        title: 'API Credentials',
        description: 'Enter your API key to connect the weather service',
        fields: ['api_key'],
        validationCommand: 'validate_api_key',
      },
      {
        stepId: 'preferences',
        title: 'Alert Preferences',
        description: 'Configure your location and alert preferences',
        fields: ['location', 'units', 'alert_types', 'polling_interval', 'notifications_enabled'],
      },
    ];

    for (const stepDef of steps) {
      const jsOutput = createSetupStep(stepDef);
      const rustReference = buildRustReferenceStep(stepDef);

      const jsNormalized = sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)));
      const rustNormalized = sortKeysDeep(rustReference);

      expect(jsNormalized).toEqual(rustNormalized);
    }

    // Verify step with validation_command
    const credStep = createSetupStep(steps[0]);
    expect(credStep.step_id).toBe('credentials');
    expect(credStep.validation_command).toBe('validate_api_key');

    // Verify step without validation_command
    const prefStep = createSetupStep(steps[1]);
    expect(prefStep.step_id).toBe('preferences');
    expect(prefStep.validation_command).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // Full weather-alerts example from design doc
  // -------------------------------------------------------------------------

  test('full weather-alerts schema from design doc matches Rust reference', () => {
    const definition = {
      properties: {
        api_key: {
          type: 'string',
          title: 'API Key',
          description: 'Your OpenWeatherMap API key',
          'x-secret': true,
        },
        location: {
          type: 'string',
          title: 'Default Location',
          description: 'City name or coordinates for weather alerts',
        },
        units: {
          type: 'string',
          title: 'Temperature Units',
          description: 'Preferred temperature unit system',
          enum: ['metric', 'imperial'],
          default: 'metric',
        },
        alert_types: {
          type: 'array',
          title: 'Alert Types',
          description: 'Types of weather alerts to receive',
          items: { type: 'string', title: '', description: '' },
          default: ['severe', 'warning'],
        },
        polling_interval: {
          type: 'integer',
          title: 'Polling Interval',
          description: 'How often to check for new alerts (in minutes)',
          default: 15,
        },
        notifications_enabled: {
          type: 'boolean',
          title: 'Enable Notifications',
          description: 'Whether to send push notifications for new alerts',
          default: true,
        },
      },
      required: ['api_key', 'location'],
    };

    const jsOutput = createConfigSchema(definition);
    const rustReference = buildRustReferenceSchema(definition);

    const jsNormalized = sortKeysDeep(JSON.parse(JSON.stringify(jsOutput)));
    const rustNormalized = sortKeysDeep(rustReference);

    expect(jsNormalized).toEqual(rustNormalized);

    // Verify the full structure
    expect(jsOutput.$schema).toBe('https://json-schema.org/draft/2020-12/schema');
    expect(jsOutput.type).toBe('object');
    expect(Object.keys(jsOutput.properties)).toHaveLength(6);
    expect(jsOutput.required).toEqual(['api_key', 'location']);
    expect(jsOutput.properties.api_key['x-secret']).toBe(true);
    expect(jsOutput.properties.units.enum).toEqual(['metric', 'imperial']);
    expect(jsOutput.properties.alert_types.items.type).toBe('string');
  });

  // -------------------------------------------------------------------------
  // Structural key naming: snake_case consistency
  // -------------------------------------------------------------------------

  test('JS SDK uses snake_case keys matching Rust serde output', () => {
    const step = createSetupStep({
      stepId: 'test_step',
      title: 'Test Step Title',
      description: 'A description that is long enough for validation',
      fields: ['field_a', 'field_b'],
      validationCommand: 'validate_test',
    });

    // Rust serializes as snake_case via struct field names
    expect(step).toHaveProperty('step_id');
    expect(step).toHaveProperty('title');
    expect(step).toHaveProperty('description');
    expect(step).toHaveProperty('fields');
    expect(step).toHaveProperty('validation_command');

    // Must NOT have camelCase keys
    expect(step).not.toHaveProperty('stepId');
    expect(step).not.toHaveProperty('validationCommand');
  });

  test('ConfigSchema uses Rust serde key names ($schema, type, properties, required)', () => {
    const schema = createConfigSchema({
      properties: {
        test: { type: 'string', title: 'Test', description: 'Test field' },
      },
      required: ['test'],
    });

    expect(schema).toHaveProperty('$schema');
    expect(schema).toHaveProperty('type');
    expect(schema).toHaveProperty('properties');
    expect(schema).toHaveProperty('required');

    // Rust renames: schema_draft → $schema, schema_type → type
    expect(schema).not.toHaveProperty('schema_draft');
    expect(schema).not.toHaveProperty('schema_type');
  });

  // -------------------------------------------------------------------------
  // Omission rules: optional fields absent when not set
  // -------------------------------------------------------------------------

  test('optional fields omitted when not set (matching Rust skip_serializing_if)', () => {
    const definition = {
      properties: {
        simple: {
          type: 'string',
          title: 'Simple',
          description: 'A simple field with no extras',
        },
      },
    };

    const jsOutput = createConfigSchema(definition);
    const field = jsOutput.properties.simple;

    // These should be absent, not null/undefined, matching Rust's skip_serializing_if
    expect('default' in field).toBe(false);
    expect('x-secret' in field).toBe(false);
    expect('items' in field).toBe(false);
    expect('enum' in field).toBe(false);
  });

  test('SetupStep omits validation_command when not provided', () => {
    const step = createSetupStep({
      stepId: 'basic',
      title: 'Basic Step',
      description: 'A basic step without validation command',
      fields: ['field_a'],
    });

    // Rust: #[serde(default, skip_serializing_if = "Option::is_none")]
    expect('validation_command' in step).toBe(false);
  });
});

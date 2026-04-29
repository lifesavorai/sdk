/**
 * Property-based tests for the Life Savor Skill Configuration SDK.
 *
 * Feature: skill-setup-workflow, Property 2: JS SDK schema round-trip
 *
 * Uses fast-check + Vitest to generate arbitrary schema definitions and
 * assert that JSON.parse(JSON.stringify(createConfigSchema(def))) equals
 * the original output.
 *
 * **Validates: Requirements 7.6**
 */

import { describe, test, expect } from 'vitest';
import * as fc from 'fast-check';

const {
  CONFIG_FIELD_TYPES,
  createConfigSchema,
  createSetupStep,
  validateConfigValues,
  parseValidationRequest,
  createValidationResponse,
  invalidCredentials,
  connectionFailed,
  validationTimeout,
} = require('./skill-config-sdk');

// ---------------------------------------------------------------------------
// Arbitraries — generators for valid schema definitions
// ---------------------------------------------------------------------------

/** Generate a non-empty alphanumeric identifier (field name / step ID). */
const arbFieldName = fc.stringMatching(/^[a-z][a-z0-9_]{0,29}$/);

/** Generate a field type from the supported set. */
const arbFieldType = fc.constantFrom(...CONFIG_FIELD_TYPES);

/** Generate a non-empty title string (1-80 chars). */
const arbTitle = fc.string({ minLength: 1, maxLength: 80 }).filter((s) => s.trim().length > 0);

/** Generate a non-empty description string (1-200 chars). */
const arbDescription = fc
  .string({ minLength: 1, maxLength: 200 })
  .filter((s) => s.trim().length > 0);

/** Generate an optional default value appropriate for a given type. */
function arbDefaultForType(fieldType) {
  switch (fieldType) {
    case 'string':
      return fc.option(fc.string({ maxLength: 50 }), { nil: undefined });
    case 'number':
      return fc.option(fc.double({ min: -1e6, max: 1e6, noNaN: true, noDefaultInfinity: true }), {
        nil: undefined,
      });
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

/** Generate a single field definition. */
const arbFieldDefinition = arbFieldType.chain((fieldType) =>
  fc
    .record({
      type: fc.constant(fieldType),
      title: arbTitle,
      description: arbDescription,
      defaultVal: arbDefaultForType(fieldType),
      secret: fc.boolean(),
    })
    .map(({ type, title, description, defaultVal, secret }) => {
      const def = { type, title, description };
      if (defaultVal !== undefined) {
        def.default = defaultVal;
      }
      if (secret) {
        def['x-secret'] = true;
      }
      return def;
    })
);

/** Generate a valid schema definition with 1-5 properties. */
const arbSchemaDefinition = fc
  .array(
    fc.tuple(arbFieldName, arbFieldDefinition),
    { minLength: 1, maxLength: 5 }
  )
  .chain((entries) => {
    // Deduplicate field names
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

    // Pick a random subset as required
    return fc.subarray(allNames, { minLength: 0, maxLength: allNames.length }).map((required) => ({
      properties,
      required,
    }));
  })
  .filter((def) => def !== null);

// ---------------------------------------------------------------------------
// Property 2: JS SDK schema round-trip
// ---------------------------------------------------------------------------

describe('Property 2: JS SDK schema round-trip', () => {
  // Feature: skill-setup-workflow, Property 2: JS SDK schema round-trip
  // **Validates: Requirements 7.6**

  test('JSON.parse(JSON.stringify(createConfigSchema(def))) equals original', () => {
    fc.assert(
      fc.property(arbSchemaDefinition, (definition) => {
        const schema = createConfigSchema(definition);
        const roundTripped = JSON.parse(JSON.stringify(schema));
        expect(roundTripped).toEqual(schema);
      }),
      { numRuns: 100 }
    );
  });

  test('round-tripped schema preserves $schema field', () => {
    fc.assert(
      fc.property(arbSchemaDefinition, (definition) => {
        const schema = createConfigSchema(definition);
        const roundTripped = JSON.parse(JSON.stringify(schema));
        expect(roundTripped.$schema).toBe('https://json-schema.org/draft/2020-12/schema');
        expect(roundTripped.type).toBe('object');
      }),
      { numRuns: 100 }
    );
  });

  test('round-tripped schema preserves all property keys', () => {
    fc.assert(
      fc.property(arbSchemaDefinition, (definition) => {
        const schema = createConfigSchema(definition);
        const roundTripped = JSON.parse(JSON.stringify(schema));
        const originalKeys = Object.keys(schema.properties).sort();
        const rtKeys = Object.keys(roundTripped.properties).sort();
        expect(rtKeys).toEqual(originalKeys);
      }),
      { numRuns: 100 }
    );
  });
});

// ---------------------------------------------------------------------------
// Unit tests for createConfigSchema
// ---------------------------------------------------------------------------

describe('createConfigSchema', () => {
  test('creates a valid schema with all field types', () => {
    const schema = createConfigSchema({
      properties: {
        name: { type: 'string', title: 'Name', description: 'User name' },
        age: { type: 'integer', title: 'Age', description: 'User age' },
        score: { type: 'number', title: 'Score', description: 'User score' },
        active: { type: 'boolean', title: 'Active', description: 'Is active' },
        tags: { type: 'array', title: 'Tags', description: 'User tags' },
      },
      required: ['name'],
    });

    expect(schema.$schema).toBe('https://json-schema.org/draft/2020-12/schema');
    expect(schema.type).toBe('object');
    expect(Object.keys(schema.properties)).toHaveLength(5);
    expect(schema.required).toEqual(['name']);
  });

  test('preserves x-secret annotation', () => {
    const schema = createConfigSchema({
      properties: {
        api_key: {
          type: 'string',
          title: 'API Key',
          description: 'Secret key',
          'x-secret': true,
        },
      },
    });

    expect(schema.properties.api_key['x-secret']).toBe(true);
  });

  test('preserves enum values', () => {
    const schema = createConfigSchema({
      properties: {
        units: {
          type: 'string',
          title: 'Units',
          description: 'Temperature units',
          enum: ['metric', 'imperial'],
        },
      },
    });

    expect(schema.properties.units.enum).toEqual(['metric', 'imperial']);
  });

  test('throws when property missing type', () => {
    expect(() =>
      createConfigSchema({
        properties: { bad: { title: 'T', description: 'D' } },
      })
    ).toThrow("missing required attribute: type");
  });

  test('throws when property missing title', () => {
    expect(() =>
      createConfigSchema({
        properties: { bad: { type: 'string', description: 'D' } },
      })
    ).toThrow("missing required attribute: title");
  });

  test('throws when property missing description', () => {
    expect(() =>
      createConfigSchema({
        properties: { bad: { type: 'string', title: 'T' } },
      })
    ).toThrow("missing required attribute: description");
  });

  test('throws for unsupported type', () => {
    expect(() =>
      createConfigSchema({
        properties: { bad: { type: 'object', title: 'T', description: 'D' } },
      })
    ).toThrow('unsupported type');
  });

  test('throws when definition is not an object', () => {
    expect(() => createConfigSchema(null)).toThrow('must be an object');
    expect(() => createConfigSchema('string')).toThrow('must be an object');
  });
});

// ---------------------------------------------------------------------------
// Unit tests for createSetupStep
// ---------------------------------------------------------------------------

describe('createSetupStep', () => {
  test('creates a valid step with snake_case keys', () => {
    const step = createSetupStep({
      stepId: 'credentials',
      title: 'API Credentials',
      description: 'Enter your API key to connect the service',
      fields: ['api_key'],
      validationCommand: 'validate_api_key',
    });

    expect(step.step_id).toBe('credentials');
    expect(step.title).toBe('API Credentials');
    expect(step.fields).toEqual(['api_key']);
    expect(step.validation_command).toBe('validate_api_key');
  });

  test('omits validation_command when not provided', () => {
    const step = createSetupStep({
      stepId: 'prefs',
      title: 'Preferences',
      description: 'Configure your preferences for the skill',
      fields: ['location'],
    });

    expect(step.validation_command).toBeUndefined();
  });

  test('throws for title too short', () => {
    expect(() =>
      createSetupStep({
        stepId: 'x',
        title: 'AB',
        description: 'A valid description here',
        fields: [],
      })
    ).toThrow('title must be 3-100 characters');
  });

  test('throws for description too short', () => {
    expect(() =>
      createSetupStep({
        stepId: 'x',
        title: 'Valid Title',
        description: 'Too short',
        fields: [],
      })
    ).toThrow('description must be 10-500 characters');
  });

  test('throws when stepId is missing', () => {
    expect(() =>
      createSetupStep({
        title: 'Title',
        description: 'A valid description here',
        fields: [],
      })
    ).toThrow('stepId is required');
  });
});

// ---------------------------------------------------------------------------
// Unit tests for validateConfigValues
// ---------------------------------------------------------------------------

describe('validateConfigValues', () => {
  const schema = createConfigSchema({
    properties: {
      api_key: { type: 'string', title: 'API Key', description: 'Key', 'x-secret': true },
      count: { type: 'integer', title: 'Count', description: 'Item count' },
      enabled: { type: 'boolean', title: 'Enabled', description: 'Toggle' },
      score: { type: 'number', title: 'Score', description: 'Score value' },
      tags: { type: 'array', title: 'Tags', description: 'Tag list' },
    },
    required: ['api_key'],
  });

  test('valid values pass', () => {
    const result = validateConfigValues(schema, {
      api_key: 'sk-123',
      count: 5,
      enabled: true,
      score: 3.14,
      tags: ['a', 'b'],
    });
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  test('missing required field fails', () => {
    const result = validateConfigValues(schema, { count: 5 });
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'api_key')).toBe(true);
  });

  test('type mismatch fails', () => {
    const result = validateConfigValues(schema, { api_key: 123 });
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'api_key')).toBe(true);
  });

  test('unknown field fails', () => {
    const result = validateConfigValues(schema, { api_key: 'k', unknown: 'x' });
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'unknown')).toBe(true);
  });

  test('integer type rejects float', () => {
    const result = validateConfigValues(schema, { api_key: 'k', count: 3.5 });
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.field === 'count')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Unit tests for validation command helpers
// ---------------------------------------------------------------------------

describe('parseValidationRequest', () => {
  test('parses a valid JSON request', () => {
    const input = JSON.stringify({
      step_id: 'credentials',
      values: { api_key: 'sk-123' },
      context: { skill_id: 'weather', is_reconfigure: false },
    });
    const req = parseValidationRequest(input);
    expect(req.stepId).toBe('credentials');
    expect(req.values.api_key).toBe('sk-123');
    expect(req.context.skillId).toBe('weather');
    expect(req.context.isReconfigure).toBe(false);
  });

  test('throws on invalid JSON', () => {
    expect(() => parseValidationRequest('not json')).toThrow('invalid JSON');
  });

  test('throws when step_id is missing', () => {
    expect(() => parseValidationRequest(JSON.stringify({ values: {} }))).toThrow(
      'step_id is required'
    );
  });
});

describe('createValidationResponse', () => {
  test('creates success response', () => {
    const resp = createValidationResponse('success', 'All good', { transformed: true });
    expect(resp.status).toBe('success');
    expect(resp.message).toBe('All good');
    expect(resp.data).toEqual({ transformed: true });
  });

  test('creates minimal response', () => {
    const resp = createValidationResponse('failure');
    expect(resp.status).toBe('failure');
    expect(resp.message).toBeUndefined();
    expect(resp.data).toBeUndefined();
  });
});

describe('error helpers', () => {
  test('invalidCredentials returns failure response', () => {
    const resp = invalidCredentials();
    expect(resp.status).toBe('failure');
    expect(resp.message).toContain('credentials');
  });

  test('connectionFailed returns failure response', () => {
    const resp = connectionFailed();
    expect(resp.status).toBe('failure');
    expect(resp.message).toContain('Connection failed');
  });

  test('validationTimeout returns failure response', () => {
    const resp = validationTimeout();
    expect(resp.status).toBe('failure');
    expect(resp.message).toContain('timed out');
  });

  test('error helpers accept custom messages', () => {
    expect(invalidCredentials('Bad key').message).toBe('Bad key');
    expect(connectionFailed('No network').message).toBe('No network');
    expect(validationTimeout('Slow').message).toBe('Slow');
  });
});

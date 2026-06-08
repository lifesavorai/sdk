/**
 * Life Savor Skill Configuration SDK
 *
 * Pure utility module for defining configuration schemas, setup steps,
 * and validation command handlers for Life Savor skills. This module
 * does NOT depend on WebSocket or iframe communication — all configuration
 * flows through the REST API endpoints.
 *
 * JSON structures match the canonical Rust types defined in
 * `developer/sdk/rust/agent/src/skill_config.rs`.
 *
 * @module skill-config-sdk
 */

'use strict';

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/**
 * Supported configuration field types.
 * @type {string[]}
 */
var CONFIG_FIELD_TYPES = ['string', 'number', 'boolean', 'integer', 'array'];

var DEFAULT_SCHEMA_DRAFT = 'https://json-schema.org/draft/2020-12/schema';

// ---------------------------------------------------------------------------
// Schema creation
// ---------------------------------------------------------------------------

/**
 * Create a validated ConfigSchema object from a definition.
 *
 * The definition object should have the shape:
 * ```
 * {
 *   properties: {
 *     field_name: {
 *       type: 'string',
 *       title: 'Field Label',
 *       description: 'Help text',
 *       default: 'optional default',
 *       'x-secret': false,
 *       items: { type: 'string', title: '...', description: '...' },
 *       enum: ['a', 'b']
 *     }
 *   },
 *   required: ['field_name']
 * }
 * ```
 *
 * @param {Object} definition - Schema definition object
 * @param {Object} definition.properties - Map of field name to field definition
 * @param {string[]} [definition.required] - List of required field names
 * @returns {Object} A validated ConfigSchema object
 * @throws {Error} If any property is missing type, title, or description
 */
function createConfigSchema(definition) {
  if (!definition || typeof definition !== 'object') {
    throw new Error('createConfigSchema: definition must be an object');
  }
  if (!definition.properties || typeof definition.properties !== 'object') {
    throw new Error('createConfigSchema: definition.properties must be an object');
  }

  var properties = {};
  var propNames = Object.keys(definition.properties);

  for (var i = 0; i < propNames.length; i++) {
    var name = propNames[i];
    var prop = definition.properties[name];

    if (!prop || typeof prop !== 'object') {
      throw new Error(
        "createConfigSchema: property '" + name + "' must be an object"
      );
    }
    if (!prop.type || typeof prop.type !== 'string') {
      throw new Error(
        "createConfigSchema: property '" + name + "' is missing required attribute: type"
      );
    }
    if (!prop.title || typeof prop.title !== 'string') {
      throw new Error(
        "createConfigSchema: property '" + name + "' is missing required attribute: title"
      );
    }
    if (!prop.description || typeof prop.description !== 'string') {
      throw new Error(
        "createConfigSchema: property '" + name + "' is missing required attribute: description"
      );
    }
    if (CONFIG_FIELD_TYPES.indexOf(prop.type) === -1) {
      throw new Error(
        "createConfigSchema: property '" +
          name +
          "' has unsupported type: " +
          prop.type +
          '. Supported types: ' +
          CONFIG_FIELD_TYPES.join(', ')
      );
    }

    var field = {
      type: prop.type,
      title: prop.title,
      description: prop.description,
    };

    if (prop.default !== undefined) {
      field.default = prop.default;
    }
    if (prop['x-secret'] === true) {
      field['x-secret'] = true;
    }
    if (prop.items !== undefined && prop.items !== null) {
      field.items = prop.items;
    }
    if (prop.enum !== undefined && prop.enum !== null) {
      field.enum = prop.enum;
    }

    properties[name] = field;
  }

  var schema = {
    $schema: DEFAULT_SCHEMA_DRAFT,
    type: 'object',
    properties: properties,
    required: Array.isArray(definition.required) ? definition.required.slice() : [],
  };

  return schema;
}

// ---------------------------------------------------------------------------
// Setup step creation
// ---------------------------------------------------------------------------

/**
 * Create a validated SetupStep object.
 *
 * @param {Object} stepDef - Step definition
 * @param {string} stepDef.stepId - Unique step identifier
 * @param {string} stepDef.title - Display title (3-100 characters)
 * @param {string} stepDef.description - Description text (10-500 characters)
 * @param {string[]} stepDef.fields - Config field names for this step
 * @param {string} [stepDef.validationCommand] - Optional validation operation name
 * @returns {Object} A validated SetupStep object with snake_case keys
 * @throws {Error} If required fields are missing or constraints are violated
 */
function createSetupStep(stepDef) {
  if (!stepDef || typeof stepDef !== 'object') {
    throw new Error('createSetupStep: stepDef must be an object');
  }
  if (!stepDef.stepId || typeof stepDef.stepId !== 'string') {
    throw new Error('createSetupStep: stepId is required and must be a string');
  }
  if (!stepDef.title || typeof stepDef.title !== 'string') {
    throw new Error('createSetupStep: title is required and must be a string');
  }
  if (stepDef.title.length < 3 || stepDef.title.length > 100) {
    throw new Error(
      'createSetupStep: title must be 3-100 characters, got ' + stepDef.title.length
    );
  }
  if (!stepDef.description || typeof stepDef.description !== 'string') {
    throw new Error('createSetupStep: description is required and must be a string');
  }
  if (stepDef.description.length < 10 || stepDef.description.length > 500) {
    throw new Error(
      'createSetupStep: description must be 10-500 characters, got ' +
        stepDef.description.length
    );
  }
  if (!Array.isArray(stepDef.fields)) {
    throw new Error('createSetupStep: fields must be an array of strings');
  }

  var step = {
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

// ---------------------------------------------------------------------------
// Config value validation
// ---------------------------------------------------------------------------

/**
 * Validate a values object against a ConfigSchema.
 *
 * @param {Object} schema - A ConfigSchema object (as returned by createConfigSchema)
 * @param {Object} values - The values object to validate
 * @returns {{ valid: boolean, errors: Array<{ field: string, message: string }> }}
 */
function validateConfigValues(schema, values) {
  var errors = [];

  if (!schema || !schema.properties) {
    return { valid: false, errors: [{ field: '_schema', message: 'Invalid schema' }] };
  }

  var vals = values || {};

  // Check required fields
  var required = schema.required || [];
  for (var r = 0; r < required.length; r++) {
    var reqField = required[r];
    if (vals[reqField] === undefined || vals[reqField] === null) {
      errors.push({ field: reqField, message: 'Required field is missing' });
    }
  }

  // Check type conformance for provided values
  var fieldNames = Object.keys(vals);
  for (var i = 0; i < fieldNames.length; i++) {
    var fieldName = fieldNames[i];
    var fieldDef = schema.properties[fieldName];

    if (!fieldDef) {
      errors.push({ field: fieldName, message: 'Unknown field not in schema' });
      continue;
    }

    var value = vals[fieldName];
    if (value === null || value === undefined) {
      continue; // null/undefined handled by required check above
    }

    var expectedType = fieldDef.type;
    var typeError = _checkType(value, expectedType);
    if (typeError) {
      errors.push({ field: fieldName, message: typeError });
    }

    // Check enum constraint
    if (fieldDef.enum && Array.isArray(fieldDef.enum)) {
      var found = false;
      for (var e = 0; e < fieldDef.enum.length; e++) {
        if (JSON.stringify(fieldDef.enum[e]) === JSON.stringify(value)) {
          found = true;
          break;
        }
      }
      if (!found) {
        errors.push({
          field: fieldName,
          message: 'Value must be one of: ' + fieldDef.enum.join(', '),
        });
      }
    }
  }

  return { valid: errors.length === 0, errors: errors };
}

/**
 * Check if a value matches the expected JSON Schema type.
 * @private
 */
function _checkType(value, expectedType) {
  switch (expectedType) {
    case 'string':
      if (typeof value !== 'string') {
        return 'Expected type string, got ' + typeof value;
      }
      break;
    case 'number':
      if (typeof value !== 'number' || isNaN(value)) {
        return 'Expected type number, got ' + typeof value;
      }
      break;
    case 'integer':
      if (typeof value !== 'number' || !Number.isInteger(value)) {
        return 'Expected type integer, got ' + (Number.isInteger(value) ? typeof value : 'non-integer number');
      }
      break;
    case 'boolean':
      if (typeof value !== 'boolean') {
        return 'Expected type boolean, got ' + typeof value;
      }
      break;
    case 'array':
      if (!Array.isArray(value)) {
        return 'Expected type array, got ' + typeof value;
      }
      break;
    default:
      return 'Unsupported type: ' + expectedType;
  }
  return null;
}

// ---------------------------------------------------------------------------
// Validation command helpers (stdin/stdout JSON-RPC)
// ---------------------------------------------------------------------------

/**
 * Parse a JSON-RPC validation request payload.
 *
 * @param {string} input - Raw JSON string (typically from stdin)
 * @returns {{ stepId: string, values: Object, context: { skillId?: string, isReconfigure: boolean } }}
 * @throws {Error} If the input is not valid JSON or missing required fields
 */
function parseValidationRequest(input) {
  var parsed;
  try {
    parsed = typeof input === 'string' ? JSON.parse(input) : input;
  } catch (err) {
    throw new Error('parseValidationRequest: invalid JSON input – ' + err.message);
  }

  if (!parsed || typeof parsed !== 'object') {
    throw new Error('parseValidationRequest: input must be a JSON object');
  }

  // Accept both snake_case (from agent) and camelCase
  var stepId = parsed.step_id || parsed.stepId;
  if (!stepId || typeof stepId !== 'string') {
    throw new Error('parseValidationRequest: step_id is required');
  }

  var values = parsed.values;
  if (values === undefined || values === null) {
    values = {};
  }

  var rawContext = parsed.context || {};
  var context = {
    skillId: rawContext.skill_id || rawContext.skillId || null,
    isReconfigure: rawContext.is_reconfigure === true || rawContext.isReconfigure === true,
  };

  return {
    stepId: stepId,
    values: values,
    context: context,
  };
}

/**
 * Create a formatted validation response object.
 *
 * @param {string} status - "success" or "failure"
 * @param {string} [message] - Human-readable message (error details on failure)
 * @param {*} [data] - Optional transformed values or additional data
 * @returns {{ status: string, message?: string, data?: * }}
 */
function createValidationResponse(status, message, data) {
  var response = { status: status };
  if (message !== undefined && message !== null) {
    response.message = message;
  }
  if (data !== undefined && data !== null) {
    response.data = data;
  }
  return response;
}

/**
 * Wrap a developer's validation handler function for stdin/stdout JSON-RPC.
 *
 * Reads stdin, parses the ValidationRequest, calls the handler, and writes
 * the ValidationResponse to stdout.
 *
 * @param {function} handlerFn - Function that receives a parsed ValidationRequest
 *   and returns a ValidationResponse object (or a Promise resolving to one).
 * @returns {Promise<void>}
 */
function handleValidation(handlerFn) {
  return new Promise(function (resolve, reject) {
    var chunks = [];

    process.stdin.setEncoding('utf8');
    process.stdin.on('data', function (chunk) {
      chunks.push(chunk);
    });
    process.stdin.on('end', function () {
      var input = chunks.join('');
      try {
        var request = parseValidationRequest(input);
        var result = handlerFn(request);

        // Support both sync and async handlers
        if (result && typeof result.then === 'function') {
          result
            .then(function (response) {
              process.stdout.write(JSON.stringify(response) + '\n');
              resolve();
            })
            .catch(function (err) {
              var errResponse = createValidationResponse(
                'failure',
                err.message || 'Validation handler error'
              );
              process.stdout.write(JSON.stringify(errResponse) + '\n');
              resolve();
            });
        } else {
          process.stdout.write(JSON.stringify(result) + '\n');
          resolve();
        }
      } catch (err) {
        var errResponse = createValidationResponse(
          'failure',
          err.message || 'Failed to process validation request'
        );
        process.stdout.write(JSON.stringify(errResponse) + '\n');
        resolve();
      }
    });
    process.stdin.on('error', function (err) {
      var errResponse = createValidationResponse(
        'failure',
        'Failed to read stdin: ' + err.message
      );
      process.stdout.write(JSON.stringify(errResponse) + '\n');
      resolve();
    });

    // If stdin is already ended (e.g., piped input), resume it
    if (process.stdin.readable) {
      process.stdin.resume();
    }
  });
}

// ---------------------------------------------------------------------------
// Error response helpers
// ---------------------------------------------------------------------------

/**
 * Create a pre-formatted failure response for invalid credentials.
 *
 * @param {string} [msg] - Optional custom message
 * @returns {{ status: string, message: string }}
 */
function invalidCredentials(msg) {
  return createValidationResponse(
    'failure',
    msg || 'Invalid credentials. Please check your API key or token and try again.'
  );
}

/**
 * Create a pre-formatted failure response for connection failures.
 *
 * @param {string} [msg] - Optional custom message
 * @returns {{ status: string, message: string }}
 */
function connectionFailed(msg) {
  return createValidationResponse(
    'failure',
    msg || 'Connection failed. Please check the endpoint URL and your network connection.'
  );
}

/**
 * Create a pre-formatted failure response for validation timeouts.
 *
 * @param {string} [msg] - Optional custom message
 * @returns {{ status: string, message: string }}
 */
function validationTimeout(msg) {
  return createValidationResponse(
    'failure',
    msg || 'Validation timed out. Please check your network connection and retry.'
  );
}

// ---------------------------------------------------------------------------
// Canvas capability helpers
// ---------------------------------------------------------------------------

var CANVAS_CONTENT_TYPES = ['video', 'scene_3d', 'layout', 'image', 'web', 'custom'];
var CANVAS_PLATFORMS = ['tvos', 'ios', 'macos', 'windows', 'linux'];

/**
 * Create a validated canvas capability declaration for the skill manifest.
 *
 * @param {Object} definition
 * @param {string[]} definition.contentTypes - Canvas content types (video, scene_3d, layout, image, web, custom)
 * @param {string[]} [definition.platforms] - Target platforms (tvos, ios, macos, etc.)
 * @param {boolean} [definition.voiceInteractive=true] - Accept voice during canvas
 * @param {Object} [definition.assets] - Bundled asset declarations
 * @param {string[]} [definition.requiresSystemComponents] - Required system components
 * @returns {Object} A validated canvas capability object for the manifest
 * @throws {Error} If content types are invalid or missing
 */
function createCanvasCapability(definition) {
  if (!definition || typeof definition !== 'object') {
    throw new Error('createCanvasCapability: definition must be an object');
  }
  if (!Array.isArray(definition.contentTypes) || definition.contentTypes.length === 0) {
    throw new Error('createCanvasCapability: contentTypes must be a non-empty array');
  }
  for (var i = 0; i < definition.contentTypes.length; i++) {
    if (CANVAS_CONTENT_TYPES.indexOf(definition.contentTypes[i]) === -1) {
      throw new Error(
        'createCanvasCapability: invalid content type "' + definition.contentTypes[i] +
        '". Valid: ' + CANVAS_CONTENT_TYPES.join(', ')
      );
    }
  }
  if (definition.platforms) {
    for (var p = 0; p < definition.platforms.length; p++) {
      if (CANVAS_PLATFORMS.indexOf(definition.platforms[p]) === -1) {
        throw new Error(
          'createCanvasCapability: invalid platform "' + definition.platforms[p] +
          '". Valid: ' + CANVAS_PLATFORMS.join(', ')
        );
      }
    }
  }

  var capability = {
    content_types: definition.contentTypes.slice(),
    voice_interactive: definition.voiceInteractive !== false,
    dismissible: true
  };
  if (definition.platforms) {
    capability.platforms = definition.platforms.slice();
  }
  if (definition.assets) {
    capability.assets = definition.assets;
  }
  if (definition.requiresSystemComponents) {
    capability.requires_system_components = definition.requiresSystemComponents.slice();
  }
  return capability;
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = {
  CONFIG_FIELD_TYPES: CONFIG_FIELD_TYPES,
  CANVAS_CONTENT_TYPES: CANVAS_CONTENT_TYPES,
  CANVAS_PLATFORMS: CANVAS_PLATFORMS,
  createConfigSchema: createConfigSchema,
  createSetupStep: createSetupStep,
  createCanvasCapability: createCanvasCapability,
  validateConfigValues: validateConfigValues,
  parseValidationRequest: parseValidationRequest,
  createValidationResponse: createValidationResponse,
  handleValidation: handleValidation,
  invalidCredentials: invalidCredentials,
  connectionFailed: connectionFailed,
  validationTimeout: validationTimeout,
};

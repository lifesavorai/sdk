/**
 * Life Savor Memory SDK
 *
 * Typed definitions for the Soul Memory system. These types correspond to the
 * Rust types in `developer/sdk/rust/agent/src/memory.rs` and are used by
 * components interacting with the agent's local Soul Memory Store through
 * the bridge protocol.
 *
 * @module memory-sdk
 */

'use strict';

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/**
 * Memory bridge operation names.
 * @readonly
 * @enum {string}
 */
var MEMORY_OPERATIONS = Object.freeze({
  CREATE: 'memory.create',
  READ: 'memory.read',
  UPDATE: 'memory.update',
  DELETE: 'memory.delete',
  PROPOSE: 'memory.propose',
  SEARCH: 'memory.search',
});

/**
 * Valid memory type classifications.
 * @type {readonly string[]}
 */
var MEMORY_TYPES = Object.freeze(['fact', 'preference', 'profile', 'workflow', 'reference']);

/**
 * Valid content format values.
 * @type {readonly string[]}
 */
var CONTENT_FORMATS = Object.freeze(['text', 'json', 'html']);

/**
 * Valid scope type values.
 * @type {readonly string[]}
 */
var SCOPE_TYPES = Object.freeze(['global', 'assistant']);

/**
 * Valid memory status values.
 * @type {readonly string[]}
 */
var MEMORY_STATUSES = Object.freeze(['active', 'deprecated']);

/**
 * Valid verbosity levels.
 * @type {readonly string[]}
 */
var VERBOSITY_LEVELS = Object.freeze(['minimal', 'concise', 'normal', 'detailed', 'verbose']);

/**
 * Valid formality levels.
 * @type {readonly string[]}
 */
var FORMALITY_LEVELS = Object.freeze(['casual', 'neutral', 'formal']);


// ---------------------------------------------------------------------------
// Type Definitions (JSDoc)
// ---------------------------------------------------------------------------

/**
 * @typedef {'fact' | 'preference' | 'profile' | 'workflow' | 'reference'} MemoryType
 */

/**
 * @typedef {'text' | 'json' | 'html'} ContentFormat
 */

/**
 * @typedef {'global' | 'assistant'} ScopeType
 */

/**
 * @typedef {'active' | 'deprecated'} MemoryStatus
 */

/**
 * @typedef {'minimal' | 'concise' | 'normal' | 'detailed' | 'verbose'} Verbosity
 */

/**
 * @typedef {'casual' | 'neutral' | 'formal'} Formality
 */

/**
 * Provenance metadata describing the origin of a memory record or change.
 * @typedef {Object} Provenance
 * @property {string} source_type - Origin type: "user_explicit", "assistant_action", "seed", "proposal_approved"
 * @property {string} [source_ref] - Reference to the source (e.g., file path for seeds)
 * @property {string} [created_by] - Identifier of the entity that created/modified the record
 * @property {string} [reason] - Human-readable reason for the change
 */

/**
 * A stored memory record with full metadata.
 * @typedef {Object} MemoryRecord
 * @property {string} id - UUID identifier
 * @property {string} key - Memory key (max 256 chars)
 * @property {string} value - Memory value content (max 100,000 chars)
 * @property {MemoryType} memory_type - Classification of the memory
 * @property {ContentFormat} content_format - Format of the value content
 * @property {ScopeType} scope_type - Visibility scope
 * @property {string} [scope_id] - Scope identifier (required for assistant scope)
 * @property {number} confidence - Confidence score [0.0, 1.0]
 * @property {MemoryStatus} status - Lifecycle status
 * @property {Provenance} provenance - Origin metadata
 * @property {string} [conflict_notes] - Conflict information if applicable
 * @property {string} [previous_value] - Previous value before last update
 * @property {number} version_number - Current version number
 * @property {string} created_at - ISO 8601 timestamp
 * @property {string} updated_at - ISO 8601 timestamp
 */

/**
 * A proposed memory record awaiting user approval.
 * @typedef {Object} MemoryProposal
 * @property {string} id - UUID identifier
 * @property {string} key - Proposed memory key
 * @property {string} value - Proposed value
 * @property {MemoryType} memory_type - Classification
 * @property {ContentFormat} content_format - Format of the value
 * @property {ScopeType} scope_type - Visibility scope
 * @property {string} [scope_id] - Scope identifier
 * @property {number} confidence - Confidence score [0.0, 1.0]
 * @property {string} status - Proposal status ("pending", "approved", "rejected")
 * @property {Provenance} provenance - Origin metadata
 * @property {string} [conflict_with_id] - ID of conflicting existing record
 * @property {string} [conflict_value] - Value of conflicting existing record
 * @property {string} created_at - ISO 8601 timestamp
 */

/**
 * Scope definition for a memory seed entry.
 * @typedef {Object} SeedScope
 * @property {ScopeType} scope_type - Scope type
 * @property {string} [scope_id] - Scope identifier (required when scope_type is "assistant")
 */

/**
 * A memory seed entry defining an initial memory to pre-load.
 * @typedef {Object} MemorySeed
 * @property {string} key - Key identifier (max 256 chars)
 * @property {string} value - Value content
 * @property {MemoryType} memory_type - Classification
 * @property {SeedScope} scope - Scope definition
 * @property {number} [confidence=1.0] - Confidence score [0.0, 1.0]
 * @property {ContentFormat} [content_format='text'] - Content format
 */

/**
 * A single behavioral trait with a strength weight.
 * @typedef {Object} PersonaTrait
 * @property {string} key - Unique key identifier (max 64 chars)
 * @property {number} strength - Strength weight [0.0, 1.0]
 * @property {string} [description] - Human-readable description (max 300 chars)
 */

/**
 * Communication style preferences for the assistant.
 * @typedef {Object} CommunicationStyle
 * @property {string} [tone] - Desired tone (max 50 chars)
 * @property {Verbosity} [verbosity] - Verbosity level
 * @property {Formality} [formality] - Formality level
 * @property {string[]} [language_preferences] - BCP-47 language tags (max 10)
 */

/**
 * Top-level persona definition parsed from persona.toml.
 * @typedef {Object} PersonaDefinition
 * @property {string} identity - Required identity string (max 200 chars)
 * @property {string} purpose - Required purpose string (max 500 chars)
 * @property {PersonaTrait[]} [traits] - Behavioral traits (max 50)
 * @property {CommunicationStyle} [communication_style] - Communication preferences
 * @property {string[]} [constraints] - Rules the assistant must never violate (max 50)
 * @property {string[]} [directives] - Behavioral directives (max 50)
 */

/**
 * A memory record paired with a semantic relevance score.
 * @typedef {Object} ScoredMemory
 * @property {MemoryRecord} record - The memory record
 * @property {number} relevance_score - Semantic similarity score
 */

// ---------------------------------------------------------------------------
// Bridge Request Types
// ---------------------------------------------------------------------------

/**
 * Request payload for `memory.create` bridge operation.
 * @typedef {Object} MemoryCreateRequest
 * @property {string} key - Memory key (max 256 chars)
 * @property {string} value - Memory value
 * @property {MemoryType} memory_type - Classification
 * @property {ContentFormat} [content_format] - Format (defaults to "text")
 * @property {ScopeType} scope_type - Scope
 * @property {string} [scope_id] - Scope identifier
 * @property {number} [confidence] - Confidence [0.0, 1.0]
 * @property {string} [source_type] - Provenance source type
 * @property {string} [source_ref] - Provenance source reference
 */

/**
 * Request payload for `memory.read` bridge operation.
 * @typedef {Object} MemoryReadRequest
 * @property {string} id - The memory record ID to retrieve
 */

/**
 * Request payload for `memory.update` bridge operation.
 * @typedef {Object} MemoryUpdateRequest
 * @property {string} id - The memory record ID to update
 * @property {string} [value] - New value
 * @property {number} [confidence] - New confidence [0.0, 1.0]
 * @property {string} [source_type] - Source type for provenance
 */

/**
 * Request payload for `memory.delete` bridge operation.
 * @typedef {Object} MemoryDeleteRequest
 * @property {string} id - The memory record ID to delete
 */

/**
 * Request payload for `memory.propose` bridge operation.
 * @typedef {Object} MemoryProposeRequest
 * @property {string} key - Memory key
 * @property {string} value - Proposed value
 * @property {MemoryType} memory_type - Classification
 * @property {ContentFormat} [content_format] - Format
 * @property {ScopeType} scope_type - Scope
 * @property {string} [scope_id] - Scope identifier
 * @property {number} [confidence] - Confidence [0.0, 1.0]
 * @property {string} assistant_id - The assistant proposing the memory
 * @property {string} [reason] - Reason for the proposal
 */

/**
 * Request payload for `memory.search` bridge operation.
 * @typedef {Object} MemorySearchRequest
 * @property {string} query - Search query (max 1000 chars)
 * @property {number} [limit] - Max results (default: 20)
 * @property {ScopeType} [scope_type] - Scope filter
 * @property {string} [scope_id] - Scope ID filter
 */

// ---------------------------------------------------------------------------
// Bridge Response Types
// ---------------------------------------------------------------------------

/**
 * Response payload for `memory.create` bridge operation.
 * @typedef {Object} MemoryCreateResponse
 * @property {MemoryRecord} record - The created record
 */

/**
 * Response payload for `memory.read` bridge operation.
 * @typedef {Object} MemoryReadResponse
 * @property {MemoryRecord|null} record - The record, or null if not found
 */

/**
 * Response payload for `memory.update` bridge operation.
 * @typedef {Object} MemoryUpdateResponse
 * @property {MemoryRecord} record - The updated record
 */

/**
 * Response payload for `memory.delete` bridge operation.
 * @typedef {Object} MemoryDeleteResponse
 * @property {boolean} success - Whether the deletion succeeded
 */

/**
 * Response payload for `memory.propose` bridge operation.
 * @typedef {Object} MemoryProposeResponse
 * @property {MemoryProposal} proposal - The created proposal
 */

/**
 * Response payload for `memory.search` bridge operation.
 * @typedef {Object} MemorySearchResponse
 * @property {ScoredMemory[]} results - Ranked search results
 */

// ---------------------------------------------------------------------------
// Validation Helpers
// ---------------------------------------------------------------------------

/**
 * Validate a memory type string.
 * @param {string} type - The memory type to validate
 * @returns {boolean} True if valid
 */
function isValidMemoryType(type) {
  return MEMORY_TYPES.includes(type);
}

/**
 * Validate a content format string.
 * @param {string} format - The content format to validate
 * @returns {boolean} True if valid
 */
function isValidContentFormat(format) {
  return CONTENT_FORMATS.includes(format);
}

/**
 * Validate a scope type string.
 * @param {string} type - The scope type to validate
 * @returns {boolean} True if valid
 */
function isValidScopeType(type) {
  return SCOPE_TYPES.includes(type);
}

/**
 * Validate a confidence value is within bounds.
 * @param {number} confidence - The confidence score
 * @returns {boolean} True if valid (0.0 to 1.0 inclusive)
 */
function isValidConfidence(confidence) {
  return typeof confidence === 'number' && confidence >= 0.0 && confidence <= 1.0;
}

/**
 * Validate a MemoryCreateRequest payload.
 * @param {MemoryCreateRequest} request - The request to validate
 * @returns {{valid: boolean, errors: Array<{field: string, message: string}>}}
 */
function validateCreateRequest(request) {
  var errors = [];

  if (!request || typeof request !== 'object') {
    return { valid: false, errors: [{ field: '', message: 'Request must be an object' }] };
  }

  if (!request.key || typeof request.key !== 'string') {
    errors.push({ field: 'key', message: 'key is required and must be a string' });
  } else if (request.key.length > 256) {
    errors.push({ field: 'key', message: 'key must not exceed 256 characters' });
  }

  if (!request.value || typeof request.value !== 'string') {
    errors.push({ field: 'value', message: 'value is required and must be a string' });
  } else if (request.value.length > 100000) {
    errors.push({ field: 'value', message: 'value must not exceed 100,000 characters' });
  }

  if (!request.memory_type || !isValidMemoryType(request.memory_type)) {
    errors.push({
      field: 'memory_type',
      message: 'memory_type must be one of: ' + MEMORY_TYPES.join(', '),
    });
  }

  if (!request.scope_type || !isValidScopeType(request.scope_type)) {
    errors.push({
      field: 'scope_type',
      message: 'scope_type must be one of: ' + SCOPE_TYPES.join(', '),
    });
  }

  if (request.scope_type === 'assistant' && (!request.scope_id || typeof request.scope_id !== 'string')) {
    errors.push({ field: 'scope_id', message: 'scope_id is required when scope_type is "assistant"' });
  }

  if (request.content_format !== undefined && !isValidContentFormat(request.content_format)) {
    errors.push({
      field: 'content_format',
      message: 'content_format must be one of: ' + CONTENT_FORMATS.join(', '),
    });
  }

  if (request.confidence !== undefined && !isValidConfidence(request.confidence)) {
    errors.push({ field: 'confidence', message: 'confidence must be a number between 0.0 and 1.0' });
  }

  return { valid: errors.length === 0, errors: errors };
}

/**
 * Validate a MemoryProposeRequest payload.
 * @param {MemoryProposeRequest} request - The request to validate
 * @returns {{valid: boolean, errors: Array<{field: string, message: string}>}}
 */
function validateProposeRequest(request) {
  var result = validateCreateRequest(request);
  var errors = result.errors.slice();

  if (!request.assistant_id || typeof request.assistant_id !== 'string') {
    errors.push({ field: 'assistant_id', message: 'assistant_id is required and must be a string' });
  }

  return { valid: errors.length === 0, errors: errors };
}

/**
 * Validate a MemorySearchRequest payload.
 * @param {MemorySearchRequest} request - The request to validate
 * @returns {{valid: boolean, errors: Array<{field: string, message: string}>}}
 */
function validateSearchRequest(request) {
  var errors = [];

  if (!request || typeof request !== 'object') {
    return { valid: false, errors: [{ field: '', message: 'Request must be an object' }] };
  }

  if (!request.query || typeof request.query !== 'string') {
    errors.push({ field: 'query', message: 'query is required and must be a string' });
  } else if (request.query.length > 1000) {
    errors.push({ field: 'query', message: 'query must not exceed 1000 characters' });
  }

  if (request.limit !== undefined && (typeof request.limit !== 'number' || request.limit < 1)) {
    errors.push({ field: 'limit', message: 'limit must be a positive number' });
  }

  if (request.scope_type !== undefined && !isValidScopeType(request.scope_type)) {
    errors.push({
      field: 'scope_type',
      message: 'scope_type must be one of: ' + SCOPE_TYPES.join(', '),
    });
  }

  return { valid: errors.length === 0, errors: errors };
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = {
  // Constants
  MEMORY_OPERATIONS,
  MEMORY_TYPES,
  CONTENT_FORMATS,
  SCOPE_TYPES,
  MEMORY_STATUSES,
  VERBOSITY_LEVELS,
  FORMALITY_LEVELS,
  // Validators
  isValidMemoryType,
  isValidContentFormat,
  isValidScopeType,
  isValidConfidence,
  validateCreateRequest,
  validateProposeRequest,
  validateSearchRequest,
};

/**
 * Tests for the Life Savor Memory SDK.
 */
import { describe, it, expect } from 'vitest';

const {
  MEMORY_OPERATIONS,
  MEMORY_TYPES,
  CONTENT_FORMATS,
  SCOPE_TYPES,
  MEMORY_STATUSES,
  VERBOSITY_LEVELS,
  FORMALITY_LEVELS,
  isValidMemoryType,
  isValidContentFormat,
  isValidScopeType,
  isValidConfidence,
  validateCreateRequest,
  validateProposeRequest,
  validateSearchRequest,
} = require('./memory-sdk');

describe('memory-sdk constants', () => {
  it('MEMORY_OPERATIONS has all bridge operations', () => {
    expect(MEMORY_OPERATIONS.CREATE).toBe('memory.create');
    expect(MEMORY_OPERATIONS.READ).toBe('memory.read');
    expect(MEMORY_OPERATIONS.UPDATE).toBe('memory.update');
    expect(MEMORY_OPERATIONS.DELETE).toBe('memory.delete');
    expect(MEMORY_OPERATIONS.PROPOSE).toBe('memory.propose');
    expect(MEMORY_OPERATIONS.SEARCH).toBe('memory.search');
  });

  it('MEMORY_TYPES contains all valid types', () => {
    expect(MEMORY_TYPES).toEqual(['fact', 'preference', 'profile', 'workflow', 'reference']);
  });

  it('CONTENT_FORMATS contains all valid formats', () => {
    expect(CONTENT_FORMATS).toEqual(['text', 'json', 'html']);
  });

  it('SCOPE_TYPES contains all valid scopes', () => {
    expect(SCOPE_TYPES).toEqual(['global', 'assistant']);
  });

  it('MEMORY_STATUSES contains all valid statuses', () => {
    expect(MEMORY_STATUSES).toEqual(['active', 'deprecated']);
  });

  it('VERBOSITY_LEVELS contains all valid levels', () => {
    expect(VERBOSITY_LEVELS).toEqual(['minimal', 'concise', 'normal', 'detailed', 'verbose']);
  });

  it('FORMALITY_LEVELS contains all valid levels', () => {
    expect(FORMALITY_LEVELS).toEqual(['casual', 'neutral', 'formal']);
  });

  it('constants are frozen', () => {
    expect(Object.isFrozen(MEMORY_OPERATIONS)).toBe(true);
    expect(Object.isFrozen(MEMORY_TYPES)).toBe(true);
    expect(Object.isFrozen(CONTENT_FORMATS)).toBe(true);
    expect(Object.isFrozen(SCOPE_TYPES)).toBe(true);
  });
});

describe('memory-sdk validators', () => {
  describe('isValidMemoryType', () => {
    it('accepts valid types', () => {
      expect(isValidMemoryType('fact')).toBe(true);
      expect(isValidMemoryType('preference')).toBe(true);
      expect(isValidMemoryType('profile')).toBe(true);
      expect(isValidMemoryType('workflow')).toBe(true);
      expect(isValidMemoryType('reference')).toBe(true);
    });

    it('rejects invalid types', () => {
      expect(isValidMemoryType('invalid')).toBe(false);
      expect(isValidMemoryType('')).toBe(false);
    });
  });

  describe('isValidContentFormat', () => {
    it('accepts valid formats', () => {
      expect(isValidContentFormat('text')).toBe(true);
      expect(isValidContentFormat('json')).toBe(true);
      expect(isValidContentFormat('html')).toBe(true);
    });

    it('rejects invalid formats', () => {
      expect(isValidContentFormat('xml')).toBe(false);
    });
  });

  describe('isValidScopeType', () => {
    it('accepts valid scopes', () => {
      expect(isValidScopeType('global')).toBe(true);
      expect(isValidScopeType('assistant')).toBe(true);
    });

    it('rejects invalid scopes', () => {
      expect(isValidScopeType('user')).toBe(false);
    });
  });

  describe('isValidConfidence', () => {
    it('accepts valid confidence values', () => {
      expect(isValidConfidence(0.0)).toBe(true);
      expect(isValidConfidence(0.5)).toBe(true);
      expect(isValidConfidence(1.0)).toBe(true);
    });

    it('rejects out-of-range values', () => {
      expect(isValidConfidence(-0.1)).toBe(false);
      expect(isValidConfidence(1.1)).toBe(false);
    });

    it('rejects non-numbers', () => {
      expect(isValidConfidence('0.5')).toBe(false);
      expect(isValidConfidence(null)).toBe(false);
    });
  });
});

describe('validateCreateRequest', () => {
  const validRequest = {
    key: 'user_name',
    value: 'Alice',
    memory_type: 'fact',
    scope_type: 'global',
  };

  it('accepts a valid request', () => {
    const result = validateCreateRequest(validRequest);
    expect(result.valid).toBe(true);
    expect(result.errors).toEqual([]);
  });

  it('accepts a request with all optional fields', () => {
    const result = validateCreateRequest({
      ...validRequest,
      content_format: 'json',
      scope_type: 'assistant',
      scope_id: 'asst-1',
      confidence: 0.9,
      source_type: 'user_explicit',
      source_ref: 'ui',
    });
    expect(result.valid).toBe(true);
  });

  it('rejects missing key', () => {
    const result = validateCreateRequest({ ...validRequest, key: undefined });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('key');
  });

  it('rejects key exceeding 256 chars', () => {
    const result = validateCreateRequest({ ...validRequest, key: 'x'.repeat(257) });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('key');
  });

  it('rejects invalid memory_type', () => {
    const result = validateCreateRequest({ ...validRequest, memory_type: 'unknown' });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('memory_type');
  });

  it('rejects invalid scope_type', () => {
    const result = validateCreateRequest({ ...validRequest, scope_type: 'invalid' });
    expect(result.valid).toBe(false);
  });

  it('requires scope_id for assistant scope', () => {
    const result = validateCreateRequest({ ...validRequest, scope_type: 'assistant' });
    expect(result.valid).toBe(false);
    expect(result.errors.some(e => e.field === 'scope_id')).toBe(true);
  });

  it('rejects invalid content_format', () => {
    const result = validateCreateRequest({ ...validRequest, content_format: 'xml' });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('content_format');
  });

  it('rejects out-of-range confidence', () => {
    const result = validateCreateRequest({ ...validRequest, confidence: 1.5 });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('confidence');
  });

  it('rejects non-object input', () => {
    const result = validateCreateRequest(null);
    expect(result.valid).toBe(false);
  });
});

describe('validateProposeRequest', () => {
  const validRequest = {
    key: 'preference',
    value: 'dark mode',
    memory_type: 'preference',
    scope_type: 'assistant',
    scope_id: 'asst-1',
    assistant_id: 'asst-1',
  };

  it('accepts a valid propose request', () => {
    const result = validateProposeRequest(validRequest);
    expect(result.valid).toBe(true);
  });

  it('rejects missing assistant_id', () => {
    const result = validateProposeRequest({ ...validRequest, assistant_id: undefined });
    expect(result.valid).toBe(false);
    expect(result.errors.some(e => e.field === 'assistant_id')).toBe(true);
  });
});

describe('validateSearchRequest', () => {
  it('accepts a valid search request', () => {
    const result = validateSearchRequest({ query: 'user preferences' });
    expect(result.valid).toBe(true);
  });

  it('accepts a request with optional fields', () => {
    const result = validateSearchRequest({
      query: 'find memories',
      limit: 10,
      scope_type: 'assistant',
      scope_id: 'asst-1',
    });
    expect(result.valid).toBe(true);
  });

  it('rejects missing query', () => {
    const result = validateSearchRequest({});
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('query');
  });

  it('rejects query exceeding 1000 chars', () => {
    const result = validateSearchRequest({ query: 'x'.repeat(1001) });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('query');
  });

  it('rejects invalid limit', () => {
    const result = validateSearchRequest({ query: 'test', limit: 0 });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('limit');
  });

  it('rejects invalid scope_type', () => {
    const result = validateSearchRequest({ query: 'test', scope_type: 'invalid' });
    expect(result.valid).toBe(false);
    expect(result.errors[0].field).toBe('scope_type');
  });
});

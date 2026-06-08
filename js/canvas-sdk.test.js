/**
 * Tests for the Canvas SDK
 */

import { describe, it, expect } from 'vitest';
import fc from 'fast-check';

const canvas = require('./canvas-sdk');
const config = require('./skill-config-sdk');

describe('createCanvasOpen', () => {
  it('builds a valid canvas_open payload', () => {
    const cmd = canvas.createCanvasOpen({
      sessionId: 'test-123',
      componentId: 'my-skill',
      contentType: 'layout',
      content: { elements: [] },
    });

    expect(cmd.type).toBe('canvas_open');
    expect(cmd.session_id).toBe('test-123');
    expect(cmd.component_id).toBe('my-skill');
    expect(cmd.content_type).toBe('layout');
    expect(cmd.voice_active).toBe(true);
    expect(cmd.dismissible).toBe(true);
  });

  it('throws on missing sessionId', () => {
    expect(() => canvas.createCanvasOpen({ componentId: 'x', contentType: 'layout', content: {} }))
      .toThrow('sessionId');
  });

  it('throws on invalid contentType', () => {
    expect(() => canvas.createCanvasOpen({ sessionId: 'x', componentId: 'x', contentType: 'invalid', content: {} }))
      .toThrow('contentType');
  });

  it('property: voice_active defaults true for any valid input', () => {
    fc.assert(fc.property(
      fc.string({ minLength: 1 }),
      fc.string({ minLength: 1 }),
      fc.constantFrom(...canvas.CANVAS_CONTENT_TYPES),
      (sessionId, componentId, contentType) => {
        const cmd = canvas.createCanvasOpen({ sessionId, componentId, contentType, content: {} });
        return cmd.voice_active === true;
      }
    ));
  });
});

describe('createSceneCommand', () => {
  it('builds a valid scene command', () => {
    const cmd = canvas.createSceneCommand('add_node', { node: { id: 'test' } });
    expect(cmd.type).toBe('canvas_scene_command');
    expect(cmd.action).toBe('add_node');
    expect(cmd.node.id).toBe('test');
  });

  it('throws on unknown action', () => {
    expect(() => canvas.createSceneCommand('fly_away', {})).toThrow('Unknown scene action');
  });
});

describe('layout builders', () => {
  it('text creates a text element', () => {
    const el = canvas.text('Hello', { fontSize: 32 });
    expect(el.type).toBe('text');
    expect(el.content).toBe('Hello');
    expect(el.style.fontSize).toBe(32);
  });

  it('button creates a button with action', () => {
    const el = canvas.button('Click me', 'do_thing');
    expect(el.type).toBe('button');
    expect(el.action).toBe('do_thing');
  });

  it('stack nests children', () => {
    const el = canvas.stack('vertical', [
      canvas.text('A'),
      canvas.text('B'),
    ]);
    expect(el.type).toBe('stack');
    expect(el.style.axis).toBe('vertical');
    expect(el.children).toHaveLength(2);
  });

  it('createLayout wraps elements', () => {
    const layout = canvas.createLayout([canvas.text('Hi')]);
    expect(layout.elements).toHaveLength(1);
    expect(layout.elements[0].content).toBe('Hi');
  });
});

describe('createCanvasCapability (config-sdk)', () => {
  it('creates a valid capability with required fields', () => {
    const cap = config.createCanvasCapability({
      contentTypes: ['scene_3d', 'video'],
      platforms: ['tvos'],
    });

    expect(cap.content_types).toEqual(['scene_3d', 'video']);
    expect(cap.platforms).toEqual(['tvos']);
    expect(cap.voice_interactive).toBe(true);
    expect(cap.dismissible).toBe(true);
  });

  it('throws on empty contentTypes', () => {
    expect(() => config.createCanvasCapability({ contentTypes: [] }))
      .toThrow('non-empty');
  });

  it('throws on invalid content type', () => {
    expect(() => config.createCanvasCapability({ contentTypes: ['hologram'] }))
      .toThrow('invalid content type');
  });

  it('throws on invalid platform', () => {
    expect(() => config.createCanvasCapability({ contentTypes: ['layout'], platforms: ['ps5'] }))
      .toThrow('invalid platform');
  });

  it('property: always has dismissible=true', () => {
    fc.assert(fc.property(
      fc.subarray(canvas.CANVAS_CONTENT_TYPES, { minLength: 1 }),
      (types) => {
        const cap = config.createCanvasCapability({ contentTypes: types });
        return cap.dismissible === true;
      }
    ));
  });
});

describe('scene3D builders', () => {
  it('createScene3D builds a content payload', () => {
    const content = canvas.createScene3D({
      assetUrl: 'https://example.com/scene.usdz',
      camera: { position: [0, 1, 3] },
    });

    expect(content.asset_url).toBe('https://example.com/scene.usdz');
    expect(content.camera.position).toEqual([0, 1, 3]);
    expect(content.interactable).toBe(true);
  });

  it('geometry creates a typed geometry descriptor', () => {
    const geom = canvas.geometry('sphere', { radius: 0.5 });
    expect(geom.type).toBe('sphere');
    expect(geom.radius).toBe(0.5);
  });

  it('material creates a PBR material', () => {
    const mat = canvas.material({ color: '#ff0000', metalness: 0.8 });
    expect(mat.lighting_model).toBe('physically_based');
    expect(mat.color).toBe('#ff0000');
    expect(mat.metalness).toBe(0.8);
  });
});

describe('updateNode', () => {
  it('builds an animated update command', () => {
    const cmd = canvas.updateNode('my-node', { position: [1, 2, 3] }, true);
    expect(cmd.type).toBe('canvas_scene_command');
    expect(cmd.action).toBe('update_node');
    expect(cmd.node_id).toBe('my-node');
    expect(cmd.position).toEqual([1, 2, 3]);
    expect(cmd.animated).toBe(true);
  });
});

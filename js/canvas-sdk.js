/**
 * Life Savor Canvas SDK
 *
 * Helpers for building fullscreen canvas skills. Provides typed builders
 * for canvas sessions, scene commands, and layout descriptors.
 *
 * @module canvas-sdk
 */

'use strict';

// ---------------------------------------------------------------------------
// Canvas Content Types
// ---------------------------------------------------------------------------

var CANVAS_CONTENT_TYPES = ['video', 'scene_3d', 'layout', 'image', 'web', 'custom'];

var SCENE_ACTIONS = [
  'add_node', 'remove_node', 'update_node', 'animate_node',
  'stop_animation', 'set_material', 'set_camera', 'add_particle',
  'set_overlay', 'set_physics', 'apply_force', 'set_environment',
  'clone_node', 'set_visibility'
];

// ---------------------------------------------------------------------------
// Canvas Session Builder
// ---------------------------------------------------------------------------

/**
 * Build a canvas_open command payload.
 *
 * @param {Object} options
 * @param {string} options.sessionId - Unique session identifier
 * @param {string} options.componentId - Your skill/assistant ID
 * @param {string} options.contentType - One of CANVAS_CONTENT_TYPES
 * @param {Object} options.content - Content payload (scene, video, layout, etc.)
 * @param {string} [options.title] - Display title
 * @param {boolean} [options.voiceActive=true] - Accept voice input
 * @param {boolean} [options.dismissible=true] - User can dismiss
 * @param {string} [options.componentType='skill'] - skill, assistant, or system
 * @returns {Object} canvas_open WebSocket message payload
 */
function createCanvasOpen(options) {
  if (!options.sessionId) throw new Error('sessionId is required');
  if (!options.componentId) throw new Error('componentId is required');
  if (!options.contentType || !CANVAS_CONTENT_TYPES.includes(options.contentType)) {
    throw new Error('contentType must be one of: ' + CANVAS_CONTENT_TYPES.join(', '));
  }
  if (!options.content) throw new Error('content is required');

  return {
    type: 'canvas_open',
    session_id: options.sessionId,
    component_id: options.componentId,
    component_type: options.componentType || 'skill',
    content_type: options.contentType,
    title: options.title || null,
    voice_active: options.voiceActive !== false,
    dismissible: options.dismissible !== false,
    content: options.content
  };
}

/**
 * Build a canvas_close command.
 */
function createCanvasClose(sessionId) {
  return { type: 'canvas_close', session_id: sessionId };
}

/**
 * Build a canvas_update command.
 */
function createCanvasUpdate(sessionId, content, title) {
  var msg = { type: 'canvas_update', session_id: sessionId };
  if (content) msg.content = content;
  if (title) msg.title = title;
  return msg;
}

// ---------------------------------------------------------------------------
// Scene Command Builders
// ---------------------------------------------------------------------------

/**
 * Build a canvas_scene_command message.
 *
 * @param {string} action - One of SCENE_ACTIONS
 * @param {Object} params - Action-specific parameters
 * @returns {Object} WebSocket message payload
 */
function createSceneCommand(action, params) {
  if (!SCENE_ACTIONS.includes(action)) {
    throw new Error('Unknown scene action: ' + action + '. Valid: ' + SCENE_ACTIONS.join(', '));
  }
  return Object.assign({ type: 'canvas_scene_command', action: action }, params);
}

/** Move/rotate/scale a node. */
function updateNode(nodeId, updates, animated) {
  return createSceneCommand('update_node', Object.assign(
    { node_id: nodeId, animated: animated !== false },
    updates
  ));
}

/** Animate a node. */
function animateNode(nodeId, animation, key) {
  var params = { node_id: nodeId, animation: animation };
  if (key) params.key = key;
  return createSceneCommand('animate_node', params);
}

/** Add a node to the scene. */
function addNode(node, parentId) {
  var params = { node: node };
  if (parentId) params.parent_id = parentId;
  return createSceneCommand('add_node', params);
}

/** Remove a node from the scene. */
function removeNode(nodeId, animated) {
  return createSceneCommand('remove_node', { node_id: nodeId, animated: !!animated });
}

/** Update the HUD overlay. */
function setOverlay(elements) {
  return createSceneCommand('set_overlay', { elements: elements });
}

/** Move the camera. */
function setCamera(options) {
  return createSceneCommand('set_camera', options);
}

// ---------------------------------------------------------------------------
// Layout Builders
// ---------------------------------------------------------------------------

/** Create a text element. */
function text(content, style) {
  return { type: 'text', content: content, style: style || {} };
}

/** Create a button element. */
function button(label, actionId, style) {
  return { type: 'button', content: label, action: actionId, style: style || {} };
}

/** Create a vertical or horizontal stack. */
function stack(axis, children, style) {
  return {
    type: 'stack',
    style: Object.assign({ axis: axis }, style || {}),
    children: children
  };
}

/** Create a spacer. */
function spacer(minLength) {
  return { type: 'spacer', style: { padding: minLength || 20 } };
}

/** Create a progress bar (value 0-1). */
function progress(value, style) {
  return { type: 'progress', content: String(value), style: style || {} };
}

/** Create a timer display (seconds). */
function timer(seconds, style) {
  return { type: 'timer', content: String(seconds), style: style || {} };
}

/** Create a divider. */
function divider() {
  return { type: 'divider' };
}

/** Create a layout content payload from elements. */
function createLayout(elements) {
  return { elements: elements };
}

// ---------------------------------------------------------------------------
// Scene 3D Content Builders
// ---------------------------------------------------------------------------

/** Create a Scene3D content payload. */
function createScene3D(options) {
  return {
    asset_url: options.assetUrl || null,
    camera: options.camera || null,
    environment: options.environment || null,
    nodes: options.nodes || null,
    overlay: options.overlay || null,
    animation_name: options.animationName || null,
    interactable: options.interactable !== false,
    physics: options.physics || null,
    background: options.background || null
  };
}

/** Create a node descriptor for Scene3D. */
function createNode(id, options) {
  return Object.assign({ id: id }, options || {});
}

/** Create a geometry descriptor. */
function geometry(type, options) {
  return Object.assign({ type: type }, options || {});
}

/** Create a PBR material. */
function material(options) {
  return Object.assign({ lighting_model: 'physically_based' }, options || {});
}

// ---------------------------------------------------------------------------
// Video Content Builder
// ---------------------------------------------------------------------------

/** Create a video content payload. */
function createVideo(url, options) {
  return Object.assign({ url: url }, options || {});
}

// ---------------------------------------------------------------------------
// Manifest Canvas Capability
// ---------------------------------------------------------------------------

/**
 * Create a canvas capability declaration for skill.json.
 *
 * @param {string[]} contentTypes - Array of content types
 * @param {Object} [options] - Additional options
 * @returns {Object} Canvas capability object for the manifest
 */
function createCanvasCapability(contentTypes, options) {
  if (!Array.isArray(contentTypes) || contentTypes.length === 0) {
    throw new Error('At least one content type is required');
  }
  for (var i = 0; i < contentTypes.length; i++) {
    if (!CANVAS_CONTENT_TYPES.includes(contentTypes[i])) {
      throw new Error('Invalid content type: ' + contentTypes[i]);
    }
  }
  return Object.assign({
    content_types: contentTypes,
    voice_interactive: true,
    dismissible: true
  }, options || {});
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = {
  // Constants
  CANVAS_CONTENT_TYPES: CANVAS_CONTENT_TYPES,
  SCENE_ACTIONS: SCENE_ACTIONS,

  // Session lifecycle
  createCanvasOpen: createCanvasOpen,
  createCanvasClose: createCanvasClose,
  createCanvasUpdate: createCanvasUpdate,

  // Scene commands
  createSceneCommand: createSceneCommand,
  updateNode: updateNode,
  animateNode: animateNode,
  addNode: addNode,
  removeNode: removeNode,
  setOverlay: setOverlay,
  setCamera: setCamera,

  // Layout builders
  text: text,
  button: button,
  stack: stack,
  spacer: spacer,
  progress: progress,
  timer: timer,
  divider: divider,
  createLayout: createLayout,

  // Scene3D builders
  createScene3D: createScene3D,
  createNode: createNode,
  geometry: geometry,
  material: material,

  // Video builder
  createVideo: createVideo,

  // Manifest
  createCanvasCapability: createCanvasCapability,
};

/**
 * Example: Canvas Calendar Skill
 *
 * Demonstrates a simple fullscreen canvas skill using the `layout` content type.
 * Shows the user's daily calendar on the TV screen with voice interaction.
 *
 * This example shows:
 * - Declaring canvas capability in the manifest
 * - Opening a layout canvas session
 * - Handling voice commands
 * - Updating the display in real-time
 * - Closing the canvas
 */

'use strict';

var configSdk = require('../skill-config-sdk');
var canvasSdk = require('../canvas-sdk');

// ---------------------------------------------------------------------------
// 1. Manifest Declaration
// ---------------------------------------------------------------------------

var manifest = {
  skill_id: 'daily-calendar',
  name: 'Daily Calendar Display',
  version: '1.0.0',
  description: 'Shows your calendar for today on Apple TV with voice navigation',
  execution_tier: 1,
  capabilities: {
    canvas: configSdk.createCanvasCapability({
      contentTypes: ['layout'],
      platforms: ['tvos', 'ios'],
      voiceInteractive: true,
    }),
  },
};

console.log('Manifest:');
console.log(JSON.stringify(manifest, null, 2));
console.log('');

// ---------------------------------------------------------------------------
// 2. Mock Calendar Data
// ---------------------------------------------------------------------------

var events = [
  { time: '9:00 AM', title: 'Team Standup', duration: '30 min', color: '#4488ff' },
  { time: '10:00 AM', title: 'Code Review: Auth Refactor', duration: '1 hour', color: '#44cc88' },
  { time: '11:30 AM', title: 'Lunch Break', duration: '1 hour', color: '#888888' },
  { time: '1:00 PM', title: 'Design Review — Canvas SDK', duration: '45 min', color: '#ff8844' },
  { time: '2:00 PM', title: 'Deep Work Block', duration: '2 hours', color: '#aa44ff' },
  { time: '4:00 PM', title: 'Yoga Session', duration: '30 min', color: '#44ddff' },
];

// ---------------------------------------------------------------------------
// 3. Build the Layout
// ---------------------------------------------------------------------------

function buildCalendarLayout(events, highlightIndex) {
  var eventElements = events.map(function (event, index) {
    var isHighlighted = index === highlightIndex;
    var timeStyle = { fontSize: 24, color: '#888888', fontWeight: isHighlighted ? 'bold' : undefined };
    var titleStyle = { fontSize: 28, color: isHighlighted ? event.color : '#ffffff', fontWeight: isHighlighted ? 'bold' : undefined };
    var durationStyle = { fontSize: 18, color: '#666666' };

    return canvasSdk.stack('horizontal', [
      canvasSdk.text(event.time, timeStyle),
      canvasSdk.stack('vertical', [
        canvasSdk.text(event.title, titleStyle),
        canvasSdk.text(event.duration, durationStyle),
      ], { spacing: 4 }),
    ], { spacing: 24 });
  });

  return canvasSdk.createLayout([
    canvasSdk.text('Today — Monday, January 15', { fontSize: 48, fontWeight: 'bold' }),
    canvasSdk.spacer(20),
    canvasSdk.stack('vertical', eventElements, { spacing: 20 }),
    canvasSdk.spacer(40),
    canvasSdk.text('Say "what\'s next?" or "details" for more info', { fontSize: 20, color: '#555555' }),
  ]);
}

// ---------------------------------------------------------------------------
// 4. Open Canvas Session
// ---------------------------------------------------------------------------

var layout = buildCalendarLayout(events, 0); // Highlight first event

var openCmd = canvasSdk.createCanvasOpen({
  sessionId: 'cal-' + Date.now(),
  componentId: 'daily-calendar',
  contentType: 'layout',
  title: 'Today — Monday, January 15',
  voiceActive: true,
  content: layout,
});

console.log('canvas_open command:');
console.log(JSON.stringify(openCmd, null, 2));
console.log('');

// ---------------------------------------------------------------------------
// 5. Handle Voice Input
// ---------------------------------------------------------------------------

function handleVoiceInput(text) {
  var lower = text.toLowerCase();

  if (lower.includes('next') || lower.includes("what's next")) {
    // Highlight next event
    console.log('Voice: "' + text + '" → Highlighting next event');
    var updatedLayout = buildCalendarLayout(events, 1);
    var updateCmd = canvasSdk.createCanvasUpdate(openCmd.session_id, updatedLayout);
    console.log(JSON.stringify(updateCmd, null, 2));
  } else if (lower.includes('close') || lower.includes('dismiss') || lower.includes('done')) {
    console.log('Voice: "' + text + '" → Closing canvas');
    var closeCmd = canvasSdk.createCanvasClose(openCmd.session_id);
    console.log(JSON.stringify(closeCmd, null, 2));
  } else {
    console.log('Voice: "' + text + '" → Not recognized, no action');
  }
}

console.log('--- Voice command handling ---');
handleVoiceInput("what's next?");
console.log('');
handleVoiceInput('close');
console.log('');
console.log('=== Example complete ===');

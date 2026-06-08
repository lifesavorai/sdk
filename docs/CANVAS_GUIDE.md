# Canvas Guide — Fullscreen Rendering for Skills & Assistants

Build skills and assistants that take over the screen on Apple TV (and other canvas-capable agents). Render video workouts, 3D scenes, dashboards, calendars, guided experiences, and interactive UIs directly on the TV.

## Overview

The Canvas system lets your skill display fullscreen content on agents that support it. Instead of just text responses in Savo chat, your skill can:

- Play guided workout videos with overlay instructions
- Render an interactive 3D yoga instructor with SceneKit
- Show a live dashboard or daily calendar
- Display image slideshows or ambient art
- Build custom interactive UIs with declarative layouts

**Supported platforms:** tvOS (Apple TV), iOS (iPad fullscreen), macOS (window takeover)

## Quick Start

### 1. Declare canvas capability in your manifest

In your `skill.json`:

```json
{
  "skill_id": "yoga-instructor",
  "name": "Yoga Instructor Pro",
  "version": "1.0.0",
  "description": "AI-powered yoga sessions with 3D pose visualization",
  "execution_tier": 2,
  "capabilities": {
    "canvas": {
      "content_types": ["scene_3d", "video", "layout"],
      "voice_interactive": true,
      "platforms": ["tvos"],
      "assets": {
        "scenes": [
          { "id": "instructor-model", "path": "assets/instructor.usdz", "description": "3D instructor model with pose animations" }
        ],
        "total_size_mb": 45
      }
    }
  }
}
```

### 2. Open a canvas session

When your skill wants to display fullscreen content, send a `canvas_open` command via the WebSocket command channel:

```json
{
  "type": "canvas_open",
  "session_id": "unique-session-id",
  "component_id": "yoga-instructor",
  "component_type": "skill",
  "content_type": "scene_3d",
  "title": "Morning Yoga — 15 min",
  "voice_active": true,
  "dismissible": true,
  "content": {
    "asset_url": "https://assets.example.com/instructor.usdz",
    "camera": {
      "position": [0, 1.5, 3],
      "look_at": [0, 1, 0],
      "fov": 60
    },
    "environment": {
      "ambient_color": "#334455",
      "directional_light_color": "#ffffff",
      "environment_map": "https://assets.example.com/studio.hdr"
    },
    "overlay": [
      { "id": "timer", "type": "text", "content": "00:30", "position": "top_right", "style": { "fontSize": 48 } },
      { "id": "pose-name", "type": "text", "content": "Warrior I", "position": "bottom_center", "style": { "fontSize": 32, "color": "#88ccff" } }
    ]
  }
}
```

### 3. Stream updates in real-time

Send granular commands to manipulate the scene without rebuilding it:

```json
{ "type": "canvas_scene_command", "action": "animate_node", "node_id": "instructor", "animation": { "type": "rotate", "to_rotation": [0, 1.57, 0], "duration": 2.0 } }
{ "type": "canvas_scene_command", "action": "set_overlay", "elements": [{ "id": "timer", "type": "text", "content": "00:15", "position": "top_right" }] }
```

### 4. Handle user input

Your skill receives voice and action events:

```json
// User tapped a node in the 3D scene
{ "type": "canvas_action", "session_id": "...", "action_id": "instructor", "data": {} }

// User spoke a voice command
{ "type": "canvas_voice_input", "session_id": "...", "text": "skip this pose" }

// User dismissed the canvas
{ "type": "canvas_dismissed", "session_id": "...", "reason": "user_dismiss" }
```

### 5. Close when done

```json
{ "type": "canvas_close", "session_id": "unique-session-id" }
```

---

## Content Types

### `scene_3d` — SceneKit 3D Scenes

Render interactive 3D content using Apple's SceneKit framework. Ships free with every Apple TV — no additional runtime needed.

**Asset formats:** `.usdz`, `.scn`, `.dae`, `.obj`

**Features:**
- PBR materials (metalness, roughness, textures)
- Physics simulation (gravity, collisions, rigid bodies)
- Skeletal animations (named clips from your 3D tool)
- Particle systems (rain, fire, sparkles)
- Camera orbit via Siri Remote touch surface
- Node tap detection → reported back to your skill
- Real-time scene manipulation via commands

**Content payload:**

```json
{
  "asset_url": "https://...",
  "camera": { "position": [0, 2, 5], "look_at": [0, 0, 0], "fov": 60, "auto_rotate": true },
  "environment": { "ambient_color": "#222233", "skybox": "https://.../sky.hdr" },
  "nodes": [
    {
      "id": "floor",
      "geometry": { "type": "plane", "width": 10, "height": 10 },
      "rotation": [-1.5708, 0, 0],
      "material": { "color": "#333333", "roughness": 0.8 }
    }
  ],
  "overlay": [...]
}
```

**Scene commands (real-time updates):**

| Command | Description |
|---------|-------------|
| `add_node` | Add a new node to the scene |
| `remove_node` | Remove a node (optionally animated fade-out) |
| `update_node` | Change position/rotation/scale/opacity |
| `animate_node` | Start a move/rotate/scale/path animation |
| `stop_animation` | Stop a running animation |
| `set_material` | Change color, texture, metalness, etc. |
| `set_camera` | Move/animate the camera |
| `add_particle` | Attach particle system to a node |
| `set_overlay` | Update the HUD overlay |
| `set_physics` | Enable/disable physics on a node |
| `apply_force` | Push a node with a physics impulse |
| `clone_node` | Duplicate a node |
| `set_visibility` | Show/hide a node |
| `set_environment` | Change lighting/fog/gravity |

---

### `video` — Video Playback

Play HLS streams or MP4 files fullscreen. Hardware-decoded with near-zero CPU.

```json
{
  "url": "https://stream.example.com/workout.m3u8",
  "start_time": 30.0,
  "loop": false,
  "show_controls": true,
  "overlay_text": "Next: Downward Dog in 10s"
}
```

---

### `layout` — Declarative UI

Describe a UI as a JSON tree of elements. Rendered natively as SwiftUI. No code needed — just data.

**Element types:** `text`, `image`, `button`, `spacer`, `divider`, `stack`, `progress`, `timer`

```json
{
  "elements": [
    { "type": "text", "content": "Today's Schedule", "style": { "fontSize": 48, "fontWeight": "bold" } },
    { "type": "divider" },
    {
      "type": "stack", "style": { "axis": "vertical", "spacing": 16 },
      "children": [
        { "type": "text", "content": "9:00 AM — Team Standup", "style": { "fontSize": 28 } },
        { "type": "text", "content": "11:00 AM — Design Review", "style": { "fontSize": 28 } },
        { "type": "text", "content": "2:00 PM — Deep Work Block", "style": { "fontSize": 28, "color": "#4488ff" } }
      ]
    },
    { "type": "spacer" },
    { "type": "button", "content": "Dismiss", "action": "close", "style": { "backgroundColor": "#333333" } }
  ]
}
```

---

### `image` — Images & Slideshows

Display single images or auto-advancing slideshows.

```json
{
  "urls": ["https://.../photo1.jpg", "https://.../photo2.jpg"],
  "display_duration": 5.0,
  "transition": "fade",
  "fit": "fill"
}
```

---

### `web` — Web Content (limited on tvOS)

Display a URL or inline HTML. On tvOS, web content is rendered as text (WebKit is not available). For rich interactive content on TV, use `layout` or `scene_3d` instead.

---

## Voice Integration

When `voice_active: true` in the session, voice input from the Siri Remote microphone is transcribed and forwarded to your skill:

```json
{ "type": "canvas_voice_input", "session_id": "...", "component_id": "yoga-instructor", "text": "pause the workout" }
```

Common voice commands to handle:
- "pause" / "resume"
- "skip" / "next" / "previous"
- "stop" / "exit" / "go back"
- Custom commands specific to your skill

---

## Asset Management

### Bundled assets (installed with the skill)

Declare assets in your manifest under `capabilities.canvas.assets`. These download when the user installs the skill and are cached locally on the agent.

### Remote assets (loaded on demand)

Reference any HTTPS URL in your canvas content. The agent downloads and caches them automatically (via `SceneAssetCache`). Subsequent loads are instant.

### Asset size guidelines

| Platform | Recommended max total | Notes |
|----------|----------------------|-------|
| tvOS | 200 MB | Apple TV has limited storage |
| iOS | 100 MB | Users are more storage-conscious |
| Desktop | 500 MB | More relaxed |

---

## User Experience Guidelines

1. **Always dismissible** — Users can press Menu (tvOS) or Back (iOS) to exit at any time. Never block this.
2. **Show loading state** — If assets take time to download, show a progress indicator. Don't leave the screen black.
3. **Voice hint** — On first canvas display, show a brief hint about voice commands if your skill supports them.
4. **Overlay sparingly** — Don't cover the 3D scene with too much text. Keep overlays minimal and positioned at edges.
5. **Respond to dismiss** — When you receive `canvas_dismissed`, clean up gracefully. Don't re-open the canvas immediately.
6. **Test on real hardware** — The tvOS simulator doesn't fully represent the 10-foot viewing experience. Test on an actual Apple TV.

---

## Example: Calendar Dashboard Skill

A simple example using the `layout` content type — no 3D, no video, just clean data display:

```json
{
  "skill_id": "daily-calendar",
  "name": "Daily Calendar Display",
  "version": "1.0.0",
  "description": "Shows your calendar for today on the TV",
  "execution_tier": 1,
  "capabilities": {
    "canvas": {
      "content_types": ["layout"],
      "voice_interactive": true,
      "platforms": ["tvos"]
    }
  }
}
```

The skill fetches the user's calendar, then opens a layout canvas:

```json
{
  "type": "canvas_open",
  "session_id": "cal-2024-01-15",
  "component_id": "daily-calendar",
  "content_type": "layout",
  "title": "Today — Monday, January 15",
  "content": {
    "elements": [
      { "type": "text", "content": "Monday, January 15", "style": { "fontSize": 56, "fontWeight": "bold" } },
      { "type": "spacer" },
      { "type": "stack", "style": { "axis": "vertical", "spacing": 20 }, "children": [
        { "type": "stack", "style": { "axis": "horizontal", "spacing": 16 }, "children": [
          { "type": "text", "content": "9:00", "style": { "fontSize": 24, "color": "#888888" } },
          { "type": "text", "content": "Team Standup", "style": { "fontSize": 28 } }
        ]},
        { "type": "stack", "style": { "axis": "horizontal", "spacing": 16 }, "children": [
          { "type": "text", "content": "11:00", "style": { "fontSize": 24, "color": "#888888" } },
          { "type": "text", "content": "Design Review", "style": { "fontSize": 28 } }
        ]}
      ]}
    ]
  }
}
```

The user says "what's next?" → skill responds with TTS and updates the overlay to highlight the next event.

---

## Platform Detection

Your skill should check the agent's capabilities before attempting to open a canvas. The agent reports its capabilities in the `agent_status` message:

```json
{
  "type": "agent_status",
  "platform": "tvos",
  "capabilities": ["voice_io", "canvas_video", "canvas_scene_3d", "canvas_layout", "canvas_image"]
}
```

If the agent doesn't report the canvas type you need, fall back to text responses in Savo chat.

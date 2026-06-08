//! Canvas capability types for fullscreen rendering on tvOS and other platforms.
//!
//! Skills declare canvas capabilities in their manifest and send canvas commands
//! over the WebSocket to control what's displayed on screen. This module provides
//! strongly-typed Rust structs for all canvas payloads and commands.
//!
//! # Content Types
//!
//! | Type | Description | Use case |
//! |------|-------------|----------|
//! | `Video` | HLS/MP4 playback | Guided workouts, tutorials |
//! | `Scene3D` | SceneKit 3D scenes | Interactive experiences, visualizations |
//! | `Layout` | Declarative JSON UI | Dashboards, calendars, status displays |
//! | `Image` | Static images/slideshows | Photo displays, ambient art |
//! | `Web` | Web content (limited on tvOS) | Dashboards on desktop/iOS |
//! | `Custom` | Plugin-provided renderer | Advanced game engine content |
//!
//! # Example
//!
//! ```rust,ignore
//! use lifesavor_skill_sdk::canvas::*;
//!
//! let session = CanvasOpen {
//!     session_id: "yoga-001".into(),
//!     component_id: "yoga-instructor".into(),
//!     component_type: ComponentType::Skill,
//!     content_type: CanvasContentType::Scene3D,
//!     title: Some("Morning Yoga".into()),
//!     voice_active: true,
//!     dismissible: true,
//!     content: serde_json::to_value(&Scene3DContent {
//!         asset_url: Some("https://assets.example.com/instructor.usdz".into()),
//!         camera: Some(SceneCamera { position: vec![0.0, 1.5, 3.0], ..Default::default() }),
//!         ..Default::default()
//!     }).unwrap(),
//! };
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Canvas Session
// ---------------------------------------------------------------------------

/// Command to open a fullscreen canvas session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasOpen {
    pub session_id: String,
    pub component_id: String,
    #[serde(default = "default_component_type")]
    pub component_type: ComponentType,
    pub content_type: CanvasContentType,
    pub title: Option<String>,
    #[serde(default = "default_true")]
    pub voice_active: bool,
    #[serde(default = "default_true")]
    pub dismissible: bool,
    pub content: Value,
}

/// Command to update canvas content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasUpdate {
    pub session_id: String,
    pub content: Option<Value>,
    pub title: Option<String>,
}

/// Command to close a canvas session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasClose {
    pub session_id: String,
}

/// Event received when user dismisses the canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasDismissed {
    pub session_id: String,
    pub component_id: String,
    pub reason: String,
}

/// Event received when user interacts with a canvas element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasAction {
    pub session_id: String,
    pub component_id: String,
    pub action_id: String,
    pub data: Option<Value>,
}

/// Event received when user speaks during canvas display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasVoiceInput {
    pub session_id: String,
    pub component_id: String,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Content Types
// ---------------------------------------------------------------------------

/// Types of content a canvas session can render.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CanvasContentType {
    Video,
    Scene3D,
    Web,
    Layout,
    Image,
    Custom,
}

/// Component types that can open a canvas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Skill,
    Assistant,
    System,
}

fn default_component_type() -> ComponentType {
    ComponentType::Skill
}
fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Scene 3D Content
// ---------------------------------------------------------------------------

/// Content payload for a 3D SceneKit canvas session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scene3DContent {
    pub asset_url: Option<String>,
    pub scene_data: Option<String>,
    pub additional_assets: Option<Vec<SceneAsset>>,
    pub camera: Option<SceneCamera>,
    pub animation_name: Option<String>,
    #[serde(default = "default_true")]
    pub interactable: bool,
    pub environment: Option<SceneEnvironment>,
    pub nodes: Option<Vec<SceneNode>>,
    pub overlay: Option<Vec<OverlayElement>>,
    pub physics: Option<ScenePhysicsConfig>,
    pub background: Option<String>,
}

/// A downloadable scene asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAsset {
    pub id: String,
    pub url: String,
    #[serde(rename = "type")]
    pub asset_type: String,
}

/// Camera configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneCamera {
    pub position: Vec<f64>,
    pub look_at: Option<Vec<f64>>,
    pub fov: Option<f64>,
    pub orthographic: Option<bool>,
    pub allows_orbit: Option<bool>,
    pub auto_rotate: Option<bool>,
    pub auto_rotate_speed: Option<f64>,
}

/// Environment (lighting, fog, skybox).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneEnvironment {
    pub ambient_color: Option<String>,
    pub ambient_intensity: Option<f64>,
    pub directional_light_color: Option<String>,
    pub directional_light_intensity: Option<f64>,
    pub directional_light_direction: Option<Vec<f64>>,
    pub skybox: Option<String>,
    pub environment_map: Option<String>,
    pub fog: Option<SceneFog>,
}

/// Fog configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFog {
    pub color: Option<String>,
    pub start_distance: f64,
    pub end_distance: f64,
    pub density: Option<f64>,
}

/// A node descriptor for adding to the scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: String,
    pub geometry: Option<NodeGeometry>,
    pub position: Option<Vec<f64>>,
    pub rotation: Option<Vec<f64>>,
    pub scale: Option<Vec<f64>>,
    pub material: Option<NodeMaterial>,
    pub animation: Option<NodeAnimation>,
    pub physics: Option<NodePhysics>,
    pub children: Option<Vec<SceneNode>>,
    pub casts_shadow: Option<bool>,
    pub is_hidden: Option<bool>,
}

/// Built-in geometry primitives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGeometry {
    #[serde(rename = "type")]
    pub geom_type: String,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub depth: Option<f64>,
    pub radius: Option<f64>,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub chamfer_radius: Option<f64>,
}

/// PBR material properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMaterial {
    pub color: Option<String>,
    pub texture_url: Option<String>,
    pub metalness: Option<f64>,
    pub roughness: Option<f64>,
    pub emission: Option<String>,
    pub emission_intensity: Option<f64>,
    pub transparency: Option<f64>,
    pub double_sided: Option<bool>,
    pub lighting_model: Option<String>,
}

/// Animation descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnimation {
    #[serde(rename = "type")]
    pub anim_type: String,
    pub duration: f64,
    pub repeat_count: Option<f64>,
    pub autoreverses: Option<bool>,
    pub to_position: Option<Vec<f64>>,
    pub to_rotation: Option<Vec<f64>>,
    pub to_scale: Option<Vec<f64>>,
    pub path: Option<Vec<Vec<f64>>>,
    pub timing_function: Option<String>,
    pub delay: Option<f64>,
}

/// Physics body descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePhysics {
    #[serde(rename = "type")]
    pub body_type: String,
    pub mass: Option<f64>,
    pub restitution: Option<f64>,
    pub friction: Option<f64>,
    pub affected_by_gravity: Option<bool>,
}

/// Physics world configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenePhysicsConfig {
    pub gravity: Option<Vec<f64>>,
    pub speed: Option<f64>,
    pub enabled: Option<bool>,
}

/// HUD overlay element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub content: Option<String>,
    pub position: String,
    pub style: Option<OverlayStyle>,
}

/// Style for overlay elements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OverlayStyle {
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub padding: Option<f64>,
    pub corner_radius: Option<f64>,
    pub opacity: Option<f64>,
}

// ---------------------------------------------------------------------------
// Video Content
// ---------------------------------------------------------------------------

/// Content payload for a video canvas session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub url: String,
    pub start_time: Option<f64>,
    pub r#loop: Option<bool>,
    pub show_controls: Option<bool>,
    pub overlay_text: Option<String>,
}

// ---------------------------------------------------------------------------
// Layout Content
// ---------------------------------------------------------------------------

/// Content payload for a declarative layout canvas session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutContent {
    pub elements: Vec<LayoutElement>,
}

/// A single layout element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub id: Option<String>,
    pub content: Option<String>,
    pub style: Option<LayoutStyle>,
    pub children: Option<Vec<LayoutElement>>,
    pub action: Option<String>,
}

/// Style properties for layout elements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutStyle {
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub padding: Option<f64>,
    pub corner_radius: Option<f64>,
    pub alignment: Option<String>,
    pub spacing: Option<f64>,
    pub axis: Option<String>,
    pub max_width: Option<f64>,
    pub opacity: Option<f64>,
}

// ---------------------------------------------------------------------------
// Image Content
// ---------------------------------------------------------------------------

/// Content payload for an image/slideshow canvas session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub urls: Vec<String>,
    pub display_duration: Option<f64>,
    pub transition: Option<String>,
    pub fit: Option<String>,
}

// ---------------------------------------------------------------------------
// Scene Commands (real-time manipulation)
// ---------------------------------------------------------------------------

/// A command to manipulate the live 3D scene in real-time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCommand {
    pub action: SceneAction,
    pub node_id: Option<String>,
    #[serde(flatten)]
    pub params: Value,
}

/// Actions available for real-time scene manipulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneAction {
    AddNode,
    RemoveNode,
    UpdateNode,
    AnimateNode,
    StopAnimation,
    SetMaterial,
    SetCamera,
    AddParticle,
    SetOverlay,
    SetPhysics,
    ApplyForce,
    SetEnvironment,
    CloneNode,
    SetVisibility,
}

// ---------------------------------------------------------------------------
// Manifest Canvas Capability
// ---------------------------------------------------------------------------

/// Canvas capability declaration for the skill manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasCapability {
    pub content_types: Vec<CanvasContentType>,
    pub platforms: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub voice_interactive: bool,
    #[serde(default = "default_true")]
    pub dismissible: bool,
    pub assets: Option<CanvasAssets>,
    pub requires_system_components: Option<Vec<String>>,
}

/// Asset declarations for the manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanvasAssets {
    pub scenes: Option<Vec<CanvasAssetEntry>>,
    pub videos: Option<Vec<CanvasAssetEntry>>,
    pub images: Option<Vec<CanvasAssetEntry>>,
    pub total_size_mb: Option<f64>,
}

/// A single declared asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasAssetEntry {
    pub id: String,
    pub path: String,
    pub description: Option<String>,
}

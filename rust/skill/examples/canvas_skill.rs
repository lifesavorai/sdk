//! Example: Canvas Skill — Yoga Instructor
//!
//! Demonstrates how to build a fullscreen canvas skill that renders
//! 3D content on Apple TV via SceneKit. This skill:
//! - Opens a 3D scene with a humanoid model
//! - Animates through yoga poses on a timer
//! - Shows overlay instructions and countdown
//! - Responds to voice commands ("skip", "pause", "stop")
//! - Closes the canvas when the workout is complete
//!
//! Run with: cargo run --example canvas_skill

use lifesavor_skill_sdk::canvas::*;
use serde_json::json;

fn main() {
    println!("=== Canvas Skill Example: Yoga Instructor ===\n");

    // 1. Define the canvas capability for the manifest
    let capability = CanvasCapability {
        content_types: vec![CanvasContentType::Scene3D, CanvasContentType::Video],
        platforms: Some(vec!["tvos".into()]),
        voice_interactive: true,
        dismissible: true,
        assets: Some(CanvasAssets {
            scenes: Some(vec![
                CanvasAssetEntry {
                    id: "instructor-model".into(),
                    path: "assets/instructor.usdz".into(),
                    description: Some("3D yoga instructor with pose animations".into()),
                },
            ]),
            videos: Some(vec![
                CanvasAssetEntry {
                    id: "intro-video".into(),
                    path: "assets/intro.mp4".into(),
                    description: Some("Welcome video for new users".into()),
                },
            ]),
            images: None,
            total_size_mb: Some(45.0),
        }),
        requires_system_components: None,
    };

    println!("Canvas Capability (for skill.json):");
    println!("{}\n", serde_json::to_string_pretty(&capability).unwrap());

    // 2. Build the canvas_open command
    let scene_content = Scene3DContent {
        asset_url: Some("https://assets.lifesavor.ai/yoga-instructor/instructor.usdz".into()),
        scene_data: None,
        additional_assets: Some(vec![
            SceneAsset {
                id: "studio-hdr".into(),
                url: "https://assets.lifesavor.ai/yoga-instructor/studio.hdr".into(),
                asset_type: "texture".into(),
            },
        ]),
        camera: Some(SceneCamera {
            position: vec![0.0, 1.5, 3.0],
            look_at: Some(vec![0.0, 1.0, 0.0]),
            fov: Some(60.0),
            orthographic: None,
            allows_orbit: Some(true),
            auto_rotate: None,
            auto_rotate_speed: None,
        }),
        animation_name: Some("warrior_1".into()),
        interactable: true,
        environment: Some(SceneEnvironment {
            ambient_color: Some("#334455".into()),
            ambient_intensity: Some(0.5),
            directional_light_color: Some("#ffffff".into()),
            directional_light_intensity: Some(1.0),
            directional_light_direction: Some(vec![-0.5, -1.0, -0.5]),
            skybox: None,
            environment_map: Some("https://assets.lifesavor.ai/yoga-instructor/studio.hdr".into()),
            fog: None,
        }),
        nodes: Some(vec![
            SceneNode {
                id: "floor".into(),
                geometry: Some(NodeGeometry {
                    geom_type: "plane".into(),
                    width: Some(10.0),
                    height: Some(10.0),
                    depth: None,
                    radius: None,
                    text: None,
                    font_size: None,
                    chamfer_radius: None,
                }),
                position: Some(vec![0.0, 0.0, 0.0]),
                rotation: Some(vec![-1.5708, 0.0, 0.0]),
                scale: None,
                material: Some(NodeMaterial {
                    color: Some("#2a2a2a".into()),
                    roughness: Some(0.9),
                    metalness: Some(0.0),
                    ..Default::default()
                }),
                animation: None,
                physics: None,
                children: None,
                casts_shadow: Some(false),
                is_hidden: None,
            },
        ]),
        overlay: Some(vec![
            OverlayElement {
                id: "timer".into(),
                element_type: "text".into(),
                content: Some("00:30".into()),
                position: "top_right".into(),
                style: Some(OverlayStyle {
                    font_size: Some(48.0),
                    font_weight: Some("bold".into()),
                    color: Some("#ffffff".into()),
                    ..Default::default()
                }),
            },
            OverlayElement {
                id: "pose-name".into(),
                element_type: "text".into(),
                content: Some("Warrior I — Hold".into()),
                position: "bottom_center".into(),
                style: Some(OverlayStyle {
                    font_size: Some(32.0),
                    color: Some("#88ccff".into()),
                    ..Default::default()
                }),
            },
        ]),
        physics: None,
        background: Some("#111111".into()),
    };

    let canvas_open = CanvasOpen {
        session_id: "yoga-session-001".into(),
        component_id: "yoga-instructor-pro".into(),
        component_type: ComponentType::Skill,
        content_type: CanvasContentType::Scene3D,
        title: Some("Morning Yoga — 15 min".into()),
        voice_active: true,
        dismissible: true,
        content: serde_json::to_value(&scene_content).unwrap(),
    };

    println!("canvas_open command:");
    println!("{}\n", serde_json::to_string_pretty(&json!({
        "type": "canvas_open",
        "session_id": canvas_open.session_id,
        "component_id": canvas_open.component_id,
        "content_type": canvas_open.content_type,
        "title": canvas_open.title,
        "voice_active": canvas_open.voice_active,
        "content": canvas_open.content,
    })).unwrap());

    // 3. Real-time scene command: transition to next pose
    let animate_cmd = json!({
        "type": "canvas_scene_command",
        "action": "animate_node",
        "node_id": "instructor",
        "animation": {
            "type": "rotate",
            "to_rotation": [0.0, 1.5708, 0.0],
            "duration": 2.0,
            "timing_function": "ease_in_out"
        }
    });

    println!("Scene command (animate to next pose):");
    println!("{}\n", serde_json::to_string_pretty(&animate_cmd).unwrap());

    // 4. Update overlay
    let overlay_cmd = json!({
        "type": "canvas_scene_command",
        "action": "set_overlay",
        "elements": [
            { "id": "timer", "type": "text", "content": "00:15", "position": "top_right", "style": { "font_size": 48, "color": "#ffffff" } },
            { "id": "pose-name", "type": "text", "content": "Transitioning to Warrior II...", "position": "bottom_center", "style": { "font_size": 32, "color": "#ffcc00" } }
        ]
    });

    println!("Scene command (update overlay):");
    println!("{}\n", serde_json::to_string_pretty(&overlay_cmd).unwrap());

    // 5. Handle voice input
    println!("Voice input handling:");
    println!("  Receive: {{ \"type\": \"canvas_voice_input\", \"text\": \"skip this pose\" }}");
    println!("  Action:  Advance to next pose, update timer and overlay\n");

    // 6. Close when done
    let close = CanvasClose {
        session_id: "yoga-session-001".into(),
    };

    println!("canvas_close command:");
    println!("{}\n", serde_json::to_string_pretty(&json!({
        "type": "canvas_close",
        "session_id": close.session_id
    })).unwrap());

    println!("=== Example complete ===");
}

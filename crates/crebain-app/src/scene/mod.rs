use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Resource, Serialize, Deserialize)]
pub struct SceneConfig {
    pub version: String,
    pub name: String,
    pub cameras: Vec<CameraConfig>,
    pub drones: Vec<DroneConfig>,
    pub detection_enabled: bool,
    pub render_quality: String,
    pub view_position: [f32; 3],
    pub view_target: [f32; 3],
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            name: "Neue Szene".to_string(),
            cameras: Vec::new(),
            drones: Vec::new(),
            detection_enabled: true,
            render_quality: "high".to_string(),
            view_position: [0.0, 5.0, 10.0],
            view_target: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraConfig {
    pub id: String,
    pub name: String,
    pub cam_type: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub fov: f32,
    pub is_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DroneConfig {
    pub id: String,
    pub drone_type: String,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
    pub velocity: [f32; 3],
    pub armed: bool,
    pub battery: f32,
}

pub fn save_scene(config: &SceneConfig, path: &str, app_data_dir: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config).map_err(|e| format!("Serialize error: {}", e))?;
    crebain_core::scene_save_file(path, &json, app_data_dir)
}

pub fn load_scene(path: &str, app_data_dir: &str) -> Result<SceneConfig, String> {
    let json = crebain_core::scene_load_file(path, app_data_dir)?;
    serde_json::from_str(&json).map_err(|e| format!("Deserialize error: {}", e))
}
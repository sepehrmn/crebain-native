use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Resource, Serialize, Deserialize)]
pub struct CrebainConfig {
    pub detection_enabled: bool,
    pub detection_interval_ms: u64,
    pub confidence_threshold: f64,
    pub iou_threshold: f64,
    pub max_detections: u32,
    pub render_quality: RenderQuality,
    pub physics_enabled: bool,
    pub sensor_simulation_enabled: bool,
}

impl Default for CrebainConfig {
    fn default() -> Self {
        Self {
            detection_enabled: true,
            detection_interval_ms: 100,
            confidence_threshold: 0.25,
            iou_threshold: 0.45,
            max_detections: 100,
            render_quality: RenderQuality::High,
            physics_enabled: true,
            sensor_simulation_enabled: true,
        }
    }
}

#[derive(Clone, Debug, Resource, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl RenderQuality {
    pub fn msaa_samples(&self) -> u32 {
        match self {
            RenderQuality::Low => 1,
            RenderQuality::Medium => 2,
            RenderQuality::High => 4,
            RenderQuality::Ultra => 8,
        }
    }

    pub fn shadow_resolution(&self) -> u32 {
        match self {
            RenderQuality::Low => 512,
            RenderQuality::Medium => 1024,
            RenderQuality::High => 2048,
            RenderQuality::Ultra => 4096,
        }
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    Running,
    Paused,
}
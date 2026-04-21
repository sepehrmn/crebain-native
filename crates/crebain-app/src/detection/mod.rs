use bevy::prelude::*;
use crebain_core;

pub struct DetectionPlugin;

#[derive(Clone, Debug, Resource)]
pub struct DetectionState {
    pub is_ready: bool,
    pub is_detecting: bool,
    pub backend_name: String,
    pub last_inference_ms: f64,
    pub detection_count: usize,
    pub error: Option<String>,
    pub detections: Vec<crebain_core::common::detection::Detection>,
}

impl Default for DetectionState {
    fn default() -> Self {
        Self {
            is_ready: false,
            is_detecting: false,
            backend_name: "Uninitialized".to_string(),
            last_inference_ms: 0.0,
            detection_count: 0,
            error: None,
            detections: Vec::new(),
        }
    }
}

#[derive(Resource)]
pub struct NativeDetector {
    pub initialized: bool,
    pub backend: String,
}

impl Default for NativeDetector {
    fn default() -> Self {
        Self {
            initialized: false,
            backend: "Uninitialized".to_string(),
        }
    }
}

impl Plugin for DetectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DetectionState>()
            .init_resource::<NativeDetector>()
            .add_systems(Startup, init_detection);
    }
}

fn init_detection(mut detector: ResMut<NativeDetector>, mut state: ResMut<DetectionState>) {
    log::info!("[Detection] Initializing platform detector...");
    crebain_core::init_platform_detector();

    let info = crebain_core::get_system_info();
    detector.initialized = true;
    detector.backend = info.backend.clone();
    state.is_ready = true;
    state.backend_name = info.backend;
    log::info!("[Detection] Backend: {}", detector.backend);
}
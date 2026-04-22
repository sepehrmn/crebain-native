use bevy::prelude::*;
use crebain_core::DetectedObject;

use crate::viewer::detection_overlay::{self, Detection3D, DetectionStateVisual};

pub struct DetectionPlugin;

#[derive(Clone, Debug, Resource)]
pub struct DetectionState {
    pub is_ready: bool,
    pub is_detecting: bool,
    pub backend_name: String,
    pub last_inference_ms: f64,
    pub last_preprocess_ms: f64,
    pub last_postprocess_ms: f64,
    pub detection_count: usize,
    pub total_inferences: u64,
    pub error: Option<String>,
    pub last_objects: Vec<DetectedObject>,
    pub fps: f64,
}

impl Default for DetectionState {
    fn default() -> Self {
        Self {
            is_ready: false,
            is_detecting: false,
            backend_name: String::new(),
            last_inference_ms: 0.0,
            last_preprocess_ms: 0.0,
            last_postprocess_ms: 0.0,
            detection_count: 0,
            total_inferences: 0,
            error: None,
            last_objects: Vec::new(),
            fps: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct NativeDetector {
    pub initialized: bool,
    pub backend: String,
}

#[derive(Resource)]
pub struct DetectionTimer {
    pub timer: Timer,
}

impl Default for DetectionTimer {
    fn default() -> Self {
        Self { timer: Timer::from_seconds(0.1, TimerMode::Repeating) }
    }
}

#[derive(Resource, Default)]
pub struct DetectionImageBuffer {
    pub data: Option<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

#[derive(Event)]
pub struct DetectionImageEvent {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Plugin for DetectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DetectionState>()
            .init_resource::<NativeDetector>()
            .init_resource::<DetectionStateVisual>()
            .init_resource::<DetectionTimer>()
            .init_resource::<DetectionImageBuffer>()
            .add_event::<DetectionImageEvent>()
            .add_systems(Startup, (
                init_detection,
                detection_overlay::setup_detection_assets,
            ))
            .add_systems(Update, (
                run_detection_loop,
                sync_detections_to_visuals.before(detection_overlay::update_detection_overlays),
                handle_detection_image_events,
            ))
            .add_systems(Update, detection_overlay::update_detection_overlays);
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

    if crebain_core::fusion_init(None).is_ok() {
        log::info!("[Detection] Sensor fusion initialized with EKF");
    }
}

fn run_detection_loop(
    time: Res<Time>,
    mut timer: ResMut<DetectionTimer>,
    mut state: ResMut<DetectionState>,
    image_buffer: Res<DetectionImageBuffer>,
    config: Res<crate::app_state::CrebainConfig>,
) {
    if !config.detection_enabled || !state.is_ready {
        return;
    }

    let interval_secs = config.detection_interval_ms as f64 / 1000.0;
    if timer.timer.duration().as_secs_f64() != interval_secs {
        timer.timer.set_duration(std::time::Duration::from_secs_f64(interval_secs));
    }

    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    let rgba_data = match image_buffer.data {
        Some(ref data) if image_buffer.width > 0 && image_buffer.height > 0 => data,
        _ => return,
    };

    state.is_detecting = true;

    match crebain_core::detect_native(
        rgba_data,
        image_buffer.width,
        image_buffer.height,
        Some(config.confidence_threshold),
        Some(config.max_detections),
    ) {
        Ok(result) => {
            state.last_inference_ms = result.inference_time_ms;
            state.last_preprocess_ms = result.preprocess_time_ms;
            state.last_postprocess_ms = result.postprocess_time_ms;
            state.detection_count = result.detections.len();
            state.total_inferences += 1;
            state.error = None;
            let total_ms = result.preprocess_time_ms + result.inference_time_ms + result.postprocess_time_ms;
            state.fps = 1000.0 / total_ms.max(1.0);
            state.last_objects = result.detections;
            state.is_detecting = false;
        }
        Err(e) => {
            state.error = Some(e);
            state.is_detecting = false;
        }
    }
}

/// Threat level lookup using ASCII case-insensitive first-byte matching.
/// Avoids `String::to_lowercase()` allocation by comparing lowercased first bytes.
fn threat_level_for_class(class_label: &str) -> u32 {
    let bytes = class_label.as_bytes();
    match bytes.first().map(|b| b.to_ascii_lowercase()) {
        Some(b'd') => 4, // drone
        Some(b'q') => 4, // quadcopter
        Some(b'u') => 4, // uav
        Some(b'a') => match bytes.get(1).map(|b| b.to_ascii_lowercase()) {
            Some(b'i') => 3, // airplane
            Some(b'r') => 3, // aircraft
            Some(b'e') => 3, // aeroplane
            _ => 1,
        },
        Some(b'h') => 3, // helicopter
        Some(b'b') => 2, // bird
        Some(b'p') => 1, // person
        _ => 1,
    }
}

fn sync_detections_to_visuals(
    detection_state: Res<DetectionState>,
    mut visual_state: ResMut<DetectionStateVisual>,
) {
    if detection_state.is_changed() {
        visual_state.detections.clear();
        let objects = &detection_state.last_objects;
        visual_state.detections.reserve(objects.len());
        for det in objects.iter() {
            visual_state.detections.push(Detection3D {
                id: det.id.clone(),
                class_label: det.class_label.clone(),
                confidence: det.confidence as f32,
                position: Vec3::new(det.bbox.x1 as f32 * 0.01, 1.0, det.bbox.y1 as f32 * 0.01),
                bbox_2d: [det.bbox.x1 as f32, det.bbox.y1 as f32, det.bbox.x2 as f32, det.bbox.y2 as f32],
                threat_level: threat_level_for_class(&det.class_label),
            });
        }
    }
}

fn handle_detection_image_events(
    mut events: EventReader<DetectionImageEvent>,
    mut buffer: ResMut<DetectionImageBuffer>,
) {
    for event in events.read() {
        buffer.data = Some(std::mem::take(&mut event.data.clone()));
        buffer.width = event.width;
        buffer.height = event.height;
    }
}
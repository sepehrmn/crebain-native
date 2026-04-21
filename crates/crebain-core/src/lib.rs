pub mod common;
pub mod inference;
pub mod sensor_fusion;
pub mod transport;

mod coreml;
mod onnx_detector;
mod zig_detector;

use sensor_fusion::{FusionConfig, FusionStats, MultiSensorFusion, SensorMeasurement, TrackOutput};
use std::sync::{Mutex, Once};

pub use common::detection::{BBox, Detection as CoreDetection, DetectionResult as CoreDetectionResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NativeDetectionResult {
    pub success: bool,
    pub detections: Vec<DetectedObject>,
    pub inference_time_ms: f64,
    pub preprocess_time_ms: f64,
    pub postprocess_time_ms: f64,
    pub backend: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedObject {
    pub id: String,
    pub class_label: String,
    pub class_index: u32,
    pub confidence: f64,
    pub bbox: BoundingBox,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

static INIT: Once = Once::new();

lazy_static::lazy_static! {
    static ref FUSION_ENGINE: Mutex<Option<MultiSensorFusion>> = Mutex::new(None);
}

const MAX_IMAGE_DIMENSION: u32 = 8192;
const MAX_IMAGE_SIZE_BYTES: usize = 64 * 1024 * 1024;

pub struct SystemInfo {
    pub platform: String,
    pub arch: String,
    pub coreml_available: bool,
    pub onnx_available: bool,
    pub backend: String,
}

pub fn init_platform_detector() {
    #[cfg(target_os = "macos")]
    {
        INIT.call_once(|| {
            let mut possible_paths: Vec<Option<std::path::PathBuf>> = vec![
                std::env::current_dir()
                    .map(|p| p.join("resources/yolov8s.mlmodelc"))
                    .ok(),
                std::env::current_dir()
                    .map(|p| p.join("yolov8s.mlmodelc"))
                    .ok(),
            ];

            if let Ok(custom_path) = std::env::var("CREBAIN_MODEL_PATH") {
                match common::path::validate_model_path(&custom_path, Some(&["mlmodelc"])) {
                    Ok(validated_path) => {
                        possible_paths.insert(0, Some(validated_path));
                    }
                    Err(e) => {
                        log::warn!("Invalid CREBAIN_MODEL_PATH: {}", e);
                    }
                }
            }

            for path_opt in possible_paths.into_iter().flatten() {
                if path_opt.exists() {
                    let path_str = path_opt.to_string_lossy().to_string();
                    log::info!("Initializing native CoreML detector with model: {}", path_str);

                    match coreml::init_detector(&path_str) {
                        Ok(()) => {
                            log::info!("Native CoreML detector initialized successfully");
                            return;
                        }
                        Err(e) => {
                            log::warn!("Failed to init CoreML with {}: {}", path_str, e);
                        }
                    }
                }
            }

            log::error!("Could not find CoreML model at any expected path");
        });
    }

    init_onnx_detector();

    #[cfg(target_os = "linux")]
    {
        if let Err(e) = zig_detector::init_global_detector() {
            log::warn!("Zig detector not available: {}", e);
        }
    }
}

fn init_onnx_detector() {
    log::info!("Initializing ONNX Runtime detector");
    match onnx_detector::init_global_detector() {
        Ok(()) => {
            log::info!("ONNX Runtime detector initialized successfully");
        }
        Err(e) => {
            log::warn!("Failed to initialize ONNX detector: {}", e);
        }
    }
}

fn convert_onnx_result(r: onnx_detector::OnnxDetectionResult) -> NativeDetectionResult {
    NativeDetectionResult {
        success: r.success,
        detections: r.detections.into_iter().map(|d| DetectedObject {
            id: format!("DET-ONNX-{:08}", d.timestamp),
            class_label: d.class_label,
            class_index: d.class_index.max(0) as u32,
            confidence: d.confidence as f64,
            bbox: BoundingBox {
                x1: d.bbox.x1 as f64,
                y1: d.bbox.y1 as f64,
                x2: d.bbox.x2 as f64,
                y2: d.bbox.y2 as f64,
            },
            timestamp: d.timestamp,
        }).collect(),
        inference_time_ms: r.inference_time_ms,
        preprocess_time_ms: r.preprocess_time_ms,
        postprocess_time_ms: r.postprocess_time_ms,
        backend: r.backend,
        error: r.error,
    }
}

fn convert_coreml_result(r: coreml::DetectionResult) -> NativeDetectionResult {
    NativeDetectionResult {
        success: r.success,
        detections: r.detections.into_iter().map(|d| DetectedObject {
            id: d.id,
            class_label: d.class_label,
            class_index: d.class_index.max(0) as u32,
            confidence: d.confidence as f64,
            bbox: BoundingBox {
                x1: d.bbox.x1 as f64,
                y1: d.bbox.y1 as f64,
                x2: d.bbox.x2 as f64,
                y2: d.bbox.y2 as f64,
            },
            timestamp: d.timestamp,
        }).collect(),
        inference_time_ms: r.inference_time_ms,
        preprocess_time_ms: r.preprocess_time_ms.unwrap_or(0.0),
        postprocess_time_ms: r.postprocess_time_ms.unwrap_or(0.0),
        backend: "CoreML Native FFI (Metal/Neural Engine)".to_string(),
        error: r.error,
    }
}

pub fn detect_native(
    rgba_data: &[u8],
    width: u32,
    height: u32,
    confidence_threshold: Option<f64>,
    max_detections: Option<u32>,
) -> Result<NativeDetectionResult, String> {
    if width == 0 || height == 0 {
        return Err("Invalid image dimensions: width and height must be > 0".to_string());
    }
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(format!(
            "Image dimensions too large: {}x{} exceeds maximum {}x{}",
            width, height, MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION
        ));
    }

    let expected_size = (width as usize) * (height as usize) * 4;
    if expected_size > MAX_IMAGE_SIZE_BYTES {
        return Err(format!(
            "Image too large: {} bytes exceeds maximum {} bytes",
            expected_size, MAX_IMAGE_SIZE_BYTES
        ));
    }
    if rgba_data.len() != expected_size {
        return Err(format!(
            "Invalid RGBA data size: expected {} bytes for {}x{}, got {}",
            expected_size, width, height, rgba_data.len()
        ));
    }

    let conf = confidence_threshold.unwrap_or(0.25).clamp(0.0, 1.0);
    let max_det = max_detections.unwrap_or(100).clamp(1, 1000) as usize;

    #[cfg(target_os = "macos")]
    {
        match coreml::detect_raw(rgba_data, width, height, conf, max_det) {
            Ok(result) => {
                let mut native = convert_coreml_result(result);
                if conf > 0.0 {
                    native.detections.retain(|d| d.confidence >= conf);
                }
                if native.detections.len() > max_det {
                    native.detections.truncate(max_det);
                }
                Ok(native)
            }
            Err(coreml_err) => {
                if onnx_detector::is_onnx_detector_ready() {
                    let onnx_result = onnx_detector::detect_with_onnx(rgba_data, width, height)
                        .map_err(|e| format!("CoreML failed: {}; ONNX failed: {}", coreml_err, e))?;
                    let mut native = convert_onnx_result(onnx_result);
                    if conf > 0.0 {
                        native.detections.retain(|d| d.confidence >= conf);
                    }
                    if native.detections.len() > max_det {
                        native.detections.truncate(max_det);
                    }
                    Ok(native)
                } else {
                    Err(coreml_err)
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let onnx_result = onnx_detector::detect_with_onnx(rgba_data, width, height)
            .map_err(|e| format!("ONNX detection failed: {}", e))?;
        let mut native = convert_onnx_result(onnx_result);
        if conf > 0.0 {
            native.detections.retain(|d| d.confidence >= conf);
        }
        if native.detections.len() > max_det {
            native.detections.truncate(max_det);
        }
        Ok(native)
    }
}

pub fn get_system_info() -> SystemInfo {
    #[cfg(target_os = "macos")]
    let platform = "macos";
    #[cfg(target_os = "linux")]
    let platform = "linux";
    #[cfg(target_os = "windows")]
    let platform = "windows";
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let platform = "unknown";

    #[cfg(target_os = "macos")]
    let coreml_available = coreml::NativeCoreMLDetector::get_global().is_some();
    #[cfg(not(target_os = "macos"))]
    let coreml_available = false;

    let onnx_available = onnx_detector::is_onnx_detector_ready();

    #[cfg(target_os = "macos")]
    let backend = if coreml_available {
        "CoreML Native FFI (Metal/Neural Engine)".to_string()
    } else if onnx_available {
        "ONNX Runtime".to_string()
    } else {
        "No Backend Available".to_string()
    };

    #[cfg(not(target_os = "macos"))]
    let backend = if onnx_available {
        "ONNX Runtime".to_string()
    } else {
        "No Backend Available".to_string()
    };

    SystemInfo {
        platform: platform.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        coreml_available,
        onnx_available,
        backend,
    }
}

pub fn fusion_init(config: Option<FusionConfig>) -> Result<(), String> {
    let cfg = config.unwrap_or_default();
    let fusion = MultiSensorFusion::new(cfg);
    let mut guard = FUSION_ENGINE.lock().map_err(|e| e.to_string())?;
    *guard = Some(fusion);
    log::info!("Sensor fusion engine initialized");
    Ok(())
}

pub fn fusion_process(
    measurements: Vec<SensorMeasurement>,
    timestamp_ms: u64,
) -> Result<Vec<TrackOutput>, String> {
    let mut guard = FUSION_ENGINE.lock().map_err(|e| e.to_string())?;
    let fusion = guard.as_mut().ok_or("Fusion engine not initialized")?;
    Ok(fusion.process_measurements(measurements, timestamp_ms))
}

pub fn fusion_get_tracks() -> Result<Vec<TrackOutput>, String> {
    let guard = FUSION_ENGINE.lock().map_err(|e| e.to_string())?;
    let fusion = guard.as_ref().ok_or("Fusion engine not initialized")?;
    Ok(fusion.get_tracks())
}

pub fn fusion_get_stats() -> Result<FusionStats, String> {
    let guard = FUSION_ENGINE.lock().map_err(|e| e.to_string())?;
    let fusion = guard.as_ref().ok_or("Fusion engine not initialized")?;
    Ok(fusion.get_stats())
}

pub fn fusion_set_config(config: FusionConfig) -> Result<(), String> {
    let mut guard = FUSION_ENGINE.lock().map_err(|e| e.to_string())?;
    let fusion = guard.as_mut().ok_or("Fusion engine not initialized")?;
    fusion.set_config(config);
    log::info!("Sensor fusion configuration updated");
    Ok(())
}

pub fn fusion_clear() -> Result<(), String> {
    let mut guard = FUSION_ENGINE.lock().map_err(|e| e.to_string())?;
    let fusion = guard.as_mut().ok_or("Fusion engine not initialized")?;
    fusion.clear();
    log::info!("Sensor fusion tracks cleared");
    Ok(())
}

pub fn fusion_get_algorithms() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "Kalman",
            "name": "Kalman Filter",
            "description": "Standard linear Kalman filter for constant velocity motion"
        }),
        serde_json::json!({
            "id": "ExtendedKalman",
            "name": "Extended Kalman Filter (EKF)",
            "description": "Handles non-linear measurement models via linearization"
        }),
        serde_json::json!({
            "id": "UnscentedKalman",
            "name": "Unscented Kalman Filter (UKF)",
            "description": "Sigma-point filter for highly non-linear systems"
        }),
        serde_json::json!({
            "id": "Particle",
            "name": "Particle Filter",
            "description": "Sequential Monte Carlo for multi-modal distributions"
        }),
        serde_json::json!({
            "id": "IMM",
            "name": "Interacting Multiple Model (IMM)",
            "description": "Adaptive filter for maneuvering target tracking"
        }),
    ]
}

pub fn fusion_get_modalities() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "id": "visual", "name": "Visual/RGB Camera", "icon": "camera" }),
        serde_json::json!({ "id": "thermal", "name": "Thermal/IR Camera", "icon": "thermometer" }),
        serde_json::json!({ "id": "acoustic", "name": "Acoustic Sensor", "icon": "audio" }),
        serde_json::json!({ "id": "radar", "name": "RADAR", "icon": "radar" }),
        serde_json::json!({ "id": "lidar", "name": "LIDAR", "icon": "scan" }),
        serde_json::json!({ "id": "radiofrequency", "name": "RF Detection", "icon": "radio" }),
    ]
}

const MAX_SCENE_STATE_BYTES: usize = 10 * 1024 * 1024;

pub fn scene_save_file(path: &str, json: &str, app_data_dir: &str) -> Result<(), String> {
    if json.is_empty() {
        return Err("Empty scene JSON".to_string());
    }
    if json.len() > MAX_SCENE_STATE_BYTES {
        return Err(format!(
            "Scene JSON too large: {} bytes exceeds maximum {} bytes",
            json.len(),
            MAX_SCENE_STATE_BYTES
        ));
    }

    let scenes_dir = std::path::PathBuf::from(app_data_dir).join("scenes");

    std::fs::create_dir_all(&scenes_dir)
        .map_err(|e| format!("Failed to create scenes directory: {}", e))?;

    let validated_path = common::path::validate_path(path, Some(&scenes_dir))?;

    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Invalid scene JSON: {}", e))?;
    let pretty =
        serde_json::to_string_pretty(&value).map_err(|e| format!("JSON encode error: {}", e))?;

    if let Some(parent) = validated_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create directory {}: {}", parent.display(), e)
        })?;
    }

    let file_name = validated_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid scene path: missing file name".to_string())?;
    let tmp_path = validated_path.with_file_name(format!("{}.tmp", file_name));

    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create {}: {}", tmp_path.display(), e))?;
        file.write_all(pretty.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", tmp_path.display(), e))?;
        let _ = file.sync_all();
    }

    if let Err(rename_err) = std::fs::rename(&tmp_path, &validated_path) {
        if validated_path.exists() {
            std::fs::remove_file(&validated_path).map_err(|e| {
                format!(
                    "Failed to remove existing {}: {}",
                    validated_path.display(),
                    e
                )
            })?;
            std::fs::rename(&tmp_path, &validated_path).map_err(|e| {
                format!(
                    "Failed to move {} -> {}: {}",
                    tmp_path.display(),
                    validated_path.display(),
                    e
                )
            })?;
        } else {
            return Err(format!(
                "Failed to move {} -> {}: {}",
                tmp_path.display(),
                validated_path.display(),
                rename_err
            ));
        }
    }

    Ok(())
}

pub fn scene_load_file(path: &str, app_data_dir: &str) -> Result<String, String> {
    let scenes_dir = std::path::PathBuf::from(app_data_dir).join("scenes");

    std::fs::create_dir_all(&scenes_dir)
        .map_err(|e| format!("Failed to create scenes directory: {}", e))?;

    let validated_path = common::path::validate_path(path, Some(&scenes_dir))?;
    let validated_path = match validated_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => validated_path,
        _ => return Err("Scene file path must end with .json".to_string()),
    };

    let meta = std::fs::metadata(&validated_path).map_err(|e| {
        format!("Failed to stat {}: {}", validated_path.display(), e)
    })?;
    if meta.len() as usize > MAX_SCENE_STATE_BYTES {
        return Err(format!(
            "Scene file too large: {} bytes exceeds maximum {} bytes",
            meta.len(),
            MAX_SCENE_STATE_BYTES
        ));
    }

    let contents = std::fs::read_to_string(&validated_path).map_err(|e| {
        format!("Failed to read {}: {}", validated_path.display(), e)
    })?;

    let _: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("Invalid scene JSON: {}", e))?;

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info() {
        let info = get_system_info();
        assert!(!info.platform.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_fusion_algorithms() {
        let algorithms = fusion_get_algorithms();
        assert!(!algorithms.is_empty());
        assert_eq!(algorithms.len(), 5);
    }

    #[test]
    fn test_fusion_modalities() {
        let modalities = fusion_get_modalities();
        assert!(!modalities.is_empty());
        assert_eq!(modalities.len(), 6);
    }
}
//! Shared detection types used across all detector backends.
//!
//! This module provides a unified `Detection` type that all detectors
//! should use, ensuring consistency across CoreML, ONNX, TensorRT, etc.

use serde::{Deserialize, Serialize};

/// Bounding box in pixel coordinates.
///
/// Coordinates are in the format [x1, y1, x2, y2] where:
/// - (x1, y1) is the top-left corner
/// - (x2, y2) is the bottom-right corner
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl BBox {
    #[inline]
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.x2 - self.x1
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.y2 - self.y1
    }

    #[inline]
    pub fn area(&self) -> f32 {
        let w = self.width();
        let h = self.height();
        if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
    }

    #[inline]
    pub fn clamp(&self, width: f32, height: f32) -> Self {
        Self {
            x1: self.x1.max(0.0).min(width),
            y1: self.y1.max(0.0).min(height),
            x2: self.x2.max(0.0).min(width),
            y2: self.y2.max(0.0).min(height),
        }
    }

    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        [self.x1, self.y1, self.x2, self.y2]
    }

    #[inline]
    pub fn from_array(arr: [f32; 4]) -> Self {
        Self {
            x1: arr[0],
            y1: arr[1],
            x2: arr[2],
            y2: arr[3],
        }
    }
}

impl From<[f32; 4]> for BBox {
    fn from(arr: [f32; 4]) -> Self {
        Self::from_array(arr)
    }
}

impl From<BBox> for [f32; 4] {
    fn from(bbox: BBox) -> Self {
        bbox.to_array()
    }
}

/// A single object detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub bbox: BBox,
    pub confidence: f32,
    pub class_id: u32,
    pub class_label: String,
}

/// Detection result from inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub detections: Vec<Detection>,
    pub inference_time_ms: f64,
    pub preprocess_time_ms: f64,
    pub postprocess_time_ms: f64,
    pub backend: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_dimensions() {
        let bbox = BBox::new(10.0, 20.0, 110.0, 70.0);
        assert_eq!(bbox.width(), 100.0);
        assert_eq!(bbox.height(), 50.0);
        assert_eq!(bbox.area(), 5000.0);
    }

    #[test]
    fn test_bbox_clamp() {
        let bbox = BBox::new(-10.0, -10.0, 200.0, 200.0);
        let clamped = bbox.clamp(100.0, 100.0);
        assert_eq!(clamped.x1, 0.0);
        assert_eq!(clamped.y1, 0.0);
        assert_eq!(clamped.x2, 100.0);
        assert_eq!(clamped.y2, 100.0);
    }
}
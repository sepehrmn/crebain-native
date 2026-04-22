use bevy::prelude::*;
use bevy_egui::egui;

use crate::app_state::CrebainConfig;
use crate::detection::DetectionState;

pub fn panels_system(
    mut ctx: bevy_egui::EguiContexts,
    detection_state: Res<DetectionState>,
    config: Res<CrebainConfig>,
    time: Res<Time>,
    mut show_performance: Local<bool>,
    mut show_sensor: Local<bool>,
    mut show_ros: Local<bool>,
) {
    let ctx = ctx.ctx_mut();

    let fps = 1.0 / time.delta_secs().max(0.001);

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Backend: {}", detection_state.backend_name));
            ui.separator();
            ui.label(format!("Detections: {}", detection_state.detection_count));
            ui.separator();
            if detection_state.last_inference_ms > 0.0 {
                ui.colored_label(
                    if detection_state.last_inference_ms < 50.0 {
                        egui::Color32::GREEN
                    } else if detection_state.last_inference_ms < 100.0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    },
                    format!("{:.1}ms", detection_state.last_inference_ms),
                );
            }
            ui.separator();
            ui.label(format!("FPS: {:.0}", fps));
            ui.separator();
            ui.checkbox(&mut show_performance, "Performance");
            ui.checkbox(&mut show_sensor, "Sensor Fusion");
            ui.checkbox(&mut show_ros, "ROS");
        });
    });

    if *show_performance {
        egui::SidePanel::right("performance_panel")
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Performance");
                ui.separator();
                ui.label(format!("Detection: {}", if config.detection_enabled { "ON" } else { "OFF" }));
                ui.label(format!("Confidence: {:.2}", config.confidence_threshold));
                ui.label(format!("IOU: {:.2}", config.iou_threshold));
                ui.label(format!("Max detections: {}", config.max_detections));

                if !detection_state.last_objects.is_empty() {
                    ui.separator();
                    ui.heading("Latest Detections");
                    for det in detection_state.last_objects.iter().take(20) {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::LIGHT_GREEN, &det.class_label);
                            ui.label(format!("{:.2}", det.confidence));
                        });
                    }
                }
            });
    }

    if *show_sensor {
        egui::SidePanel::left("sensor_panel")
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Sensor Fusion");
                ui.separator();
                ui.label("Fusion engine status panel");
                ui.label("(Initialize fusion to see stats)");
            });
    }

    if *show_ros {
        egui::Window::new("ROS Connection")
            .show(ctx, |ui| {
                ui.heading("ROS2 / Zenoh Bridge");
                ui.separator();
                ui.label("Connect to Zenoh transport for real-time data");
                if ui.button("Connect").clicked() {
                    log::info!("Zenoh connect requested");
                }
                if ui.button("Disconnect").clicked() {
                    log::info!("Zenoh disconnect requested");
                }
            });
    }
}
use bevy::prelude::*;
use bevy_egui::egui;

use crate::app_state::CrebainConfig;
use crate::detection::{DetectionState, SensorFusionState};
use crate::transport::TransportState;

#[allow(clippy::too_many_arguments)]
pub fn panels_system(
    mut ctx: bevy_egui::EguiContexts,
    detection_state: Res<DetectionState>,
    config: Res<CrebainConfig>,
    transport_state: Res<TransportState>,
    fusion_state: Res<SensorFusionState>,
    time: Res<Time>,
    mut show_performance: Local<bool>,
    mut show_sensor: Local<bool>,
    mut show_ros: Local<bool>,
) {
    let ctx = ctx.ctx_mut();

    let fps = 1.0 / time.delta_secs().max(0.001);

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let conn_color = if transport_state.connected {
                egui::Color32::GREEN
            } else {
                egui::Color32::DARK_GRAY
            };
            ui.colored_label(conn_color, if transport_state.connected { "● Connected" } else { "○ Disconnected" });
            ui.separator();
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
            if detection_state.fps > 0.0 {
                ui.label(format!("Pipeline: {:.1} FPS", detection_state.fps));
            }
            if let Some(ref err) = detection_state.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
            if transport_state.messages_received > 0 {
                ui.label(format!("RX: {}", transport_state.messages_received));
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
                ui.label(format!("Inferences: {}", detection_state.total_inferences));
                ui.label(format!("Confidence: {:.2}", config.confidence_threshold));
                ui.label(format!("IOU: {:.2}", config.iou_threshold));
                ui.label(format!("Max detections: {}", config.max_detections));

                if detection_state.last_inference_ms > 0.0 {
                    ui.separator();
                    ui.heading("Timing");
                    ui.label(format!("Preprocess: {:.2}ms", detection_state.last_preprocess_ms));
                    ui.label(format!("Inference: {:.2}ms", detection_state.last_inference_ms));
                    ui.label(format!("Postprocess: {:.2}ms", detection_state.last_postprocess_ms));
                    ui.label(format!("Pipeline FPS: {:.1}", detection_state.fps));
                }

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
                if fusion_state.initialized {
                    ui.colored_label(egui::Color32::GREEN, "● Active");
                    ui.label(format!("Algorithm: {}", fusion_state.algorithm));
                    ui.label(format!("Confirmed tracks: {}", fusion_state.track_count));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "○ Standby");
                    ui.label("Waiting for sensor data...");
                }
            });
    }

    if *show_ros {
        egui::Window::new("ROS Connection")
            .show(ctx, |ui| {
                ui.heading("ROS2 / Zenoh Bridge");
                ui.separator();

                if transport_state.connected {
                    ui.colored_label(egui::Color32::GREEN, "● Connected");
                    ui.label(format!("Messages received: {}", transport_state.messages_received));
                    ui.label(format!("Messages sent: {}", transport_state.messages_sent));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "○ Not connected");
                    ui.label("Configure Zenoh to connect");
                }

                ui.separator();
                ui.label("Environment:");
                ui.horizontal(|ui| {
                    ui.label("CREBAIN_ZENOH=");
                    let zenoh_env = std::env::var("CREBAIN_ZENOH").unwrap_or_else(|_| "not set (default: enabled)".to_string());
                    ui.label(zenoh_env);
                });

                if ui.button("Connect").clicked() {
                    log::info!("Zenoh connect requested");
                }
                if ui.button("Disconnect").clicked() {
                    log::info!("Zenoh disconnect requested");
                }
            });
    }
}
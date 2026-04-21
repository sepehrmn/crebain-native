use bevy::prelude::*;
use bevy_egui::egui;

use crate::app_state::CrebainConfig;

pub fn menu_bar_system(mut ctx: bevy_egui::EguiContexts, mut config: ResMut<CrebainConfig>) {
    let ctx = ctx.ctx_mut();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save Scene").clicked() {
                    ui.close_menu();
                }
                if ui.button("Load Scene").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut config.detection_enabled, "Detection");
                ui.separator();

                let quality_labels = ["Low", "Medium", "High", "Ultra"];
                let qualities = [
                    crate::app_state::RenderQuality::Low,
                    crate::app_state::RenderQuality::Medium,
                    crate::app_state::RenderQuality::High,
                    crate::app_state::RenderQuality::Ultra,
                ];
                for (label, quality) in quality_labels.iter().zip(qualities.iter()) {
                    if ui.radio(config.render_quality == *quality, *label).clicked() {
                        config.render_quality = quality.clone();
                    }
                }
            });

            ui.menu_button("Detection", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Confidence:");
                    ui.add(egui::DragValue::new(&mut config.confidence_threshold).speed(0.01).range(0.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("IOU Threshold:");
                    ui.add(egui::DragValue::new(&mut config.iou_threshold).speed(0.01).range(0.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Max Detections:");
                    ui.add(egui::DragValue::new(&mut config.max_detections as &mut u32).speed(1).range(1..=1000));
                });
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About CREBAIN").clicked() {
                    ui.close_menu();
                }
            });
        });
    });
}
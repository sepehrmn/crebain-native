use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod app_state;
mod camera;
mod detection;
mod transport;
mod ui;
mod viewer;

use app_state::AppState;
use camera::TacticalCameraPlugin;
use detection::DetectionPlugin;
use transport::TransportPlugin;
use viewer::ViewerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CREBAIN - Adaptives Reaktions- und Aufklärungssystem".into(),
                resolution: (1600.0, 1000.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(DetectionPlugin)
        .add_plugins(TransportPlugin)
        .add_plugins(ViewerPlugin)
        .add_plugins(TacticalCameraPlugin)
        .init_state::<AppState>()
        .insert_resource(app_state::CrebainConfig::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            ui::hud::panels_system,
            ui::top_menu::menu_bar_system,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera::TacticalCamera::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    commands.spawn((
        PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
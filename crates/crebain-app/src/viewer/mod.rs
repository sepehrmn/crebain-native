use bevy::prelude::*;

pub mod detection_overlay;
pub mod drone;
pub mod grid;
pub mod terrain;

pub struct ViewerPlugin;

impl Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<grid::GridConfig>()
            .init_resource::<drone::DroneRegistry>()
            .add_systems(Startup, (
                grid::spawn_tactical_grid,
                grid::spawn_origin_axes,
                terrain::spawn_terrain,
            ))
            .add_systems(Update, (
                drone::update_drone_positions,
                drone::update_or_create_drone_visuals,
            ));
    }
}
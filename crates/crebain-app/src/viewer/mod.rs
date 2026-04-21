pub mod terrain;
pub mod grid;

use bevy::prelude::*;

pub struct ViewerPlugin;

impl Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (grid::spawn_tactical_grid, terrain::spawn_terrain));
    }
}
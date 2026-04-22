use bevy::prelude::*;

#[derive(Component)]
pub struct TacticalGrid;

#[derive(Resource)]
pub struct GridConfig {
    pub size: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            size: 200.0,
        }
    }
}

pub fn spawn_tactical_grid(
    mut commands: Commands,
    config: Res<GridConfig>,
) {
    let _size = config.size;
    commands.spawn(TacticalGrid);
}

#[derive(Component)]
pub struct GridOriginMarker;

pub fn spawn_origin_axes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let axis_length = 5.0f32;
    let axis_thickness = 0.05f32;

    // X axis (red)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(axis_length, axis_thickness, axis_thickness))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.1, 0.1),
            emissive: LinearRgba::new(0.5, 0.0, 0.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(axis_length / 2.0, 0.01, 0.0),
        GridOriginMarker,
    ));

    // Z axis (blue)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(axis_thickness, axis_thickness, axis_length))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.8),
            emissive: LinearRgba::new(0.0, 0.0, 0.5, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.01, axis_length / 2.0),
        GridOriginMarker,
    ));

    // Y axis (green) - up
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(axis_thickness, axis_length, axis_thickness))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.8, 0.1),
            emissive: LinearRgba::new(0.0, 0.5, 0.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, axis_length / 2.0, 0.0),
        GridOriginMarker,
    ));
}
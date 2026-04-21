use bevy::prelude::*;

#[derive(Component)]
pub struct TacticalGrid;

pub fn spawn_tactical_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = 200.0f32;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.12, 0.05),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
        TacticalGrid,
    ));
}
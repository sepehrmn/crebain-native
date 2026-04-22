use bevy::prelude::*;

#[derive(Component)]
pub struct Terrain;

#[derive(Resource, Clone, Default)]
pub enum FloorStyle {
    #[default]
    Concrete,
    Grass,
    Asphalt,
    Checker,
    Terrain,
}

pub fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = 200.0f32;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.12, 0.08),
            double_sided: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.05, 0.0),
        Terrain,
    ));
}
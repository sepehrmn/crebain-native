use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, Resource, Default)]
pub struct DetectionStateVisual {
    pub detections: Vec<Detection3D>,
}

#[derive(Clone, Debug)]
pub struct Detection3D {
    pub id: String,
    pub class_label: String,
    pub confidence: f32,
    pub position: Vec3,
    pub bbox_2d: [f32; 4],
    pub threat_level: u32,
}

#[derive(Component)]
pub struct DetectionBox {
    pub detection_id: String,
    pub threat_level: u32,
}

#[derive(Resource)]
pub struct DetectionAssets {
    mesh: Handle<Mesh>,
    materials: [Handle<StandardMaterial>; 5],
}

pub fn setup_detection_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    let materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.4),
            emissive: LinearRgba::new(0.1, 0.1, 0.1, 1.0),
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.2),
            emissive: LinearRgba::new(0.0, 0.3, 0.0, 1.0),
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.8, 0.2),
            emissive: LinearRgba::new(0.3, 0.4, 0.0, 1.0),
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.2),
            emissive: LinearRgba::new(0.4, 0.0, 0.0, 1.0),
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            emissive: LinearRgba::new(0.5, 0.0, 0.0, 1.0),
            ..default()
        }),
    ];
    commands.insert_resource(DetectionAssets { mesh, materials });
}

pub fn update_detection_overlays(
    assets: Res<DetectionAssets>,
    detection_state: Res<DetectionStateVisual>,
    existing_query: Query<(Entity, &DetectionBox)>,
    mut transform_query: Query<&mut Transform>,
    mut commands: Commands,
) {
    if !detection_state.is_changed() {
        return;
    }

    let current_ids: HashSet<&String> = detection_state.detections.iter().map(|d| &d.id).collect();

    // Despawn removed detections and update existing ones
    for (entity, detection_box) in existing_query.iter() {
        if !current_ids.contains(&detection_box.detection_id) {
            commands.entity(entity).despawn();
        } else if let Some(det) = detection_state.detections.iter().find(|d| d.id == detection_box.detection_id) {
            if let Ok(mut transform) = transform_query.get_mut(entity) {
                transform.translation = det.position;
            }
            if detection_box.threat_level != det.threat_level {
                commands.entity(entity).insert(MeshMaterial3d(assets.materials[det.threat_level as usize].clone()));
                commands.entity(entity).insert(DetectionBox {
                    detection_id: det.id.clone(),
                    threat_level: det.threat_level,
                });
            }
        }
    }

    // Spawn new detections
    let existing_ids: HashSet<String> = existing_query
        .iter()
        .map(|(_, db)| db.detection_id.clone())
        .collect();

    for det in &detection_state.detections {
        if !existing_ids.contains(&det.id) {
            let material = assets.materials[det.threat_level as usize].clone();
            commands.spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(det.position),
                DetectionBox {
                    detection_id: det.id.clone(),
                    threat_level: det.threat_level,
                },
            ));
        }
    }
}
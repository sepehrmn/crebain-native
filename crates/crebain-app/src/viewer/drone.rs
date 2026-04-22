use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub const DRONE_TRAIL_LENGTH: usize = 50;

#[derive(Clone, Debug, Resource, Default)]
pub struct DroneRegistry {
    pub drones: HashMap<String, DroneState>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DroneState {
    pub id: String,
    pub name: String,
    pub drone_type: DroneType,
    pub position: Vec3,
    pub orientation: Quat,
    pub velocity: Vec3,
    pub armed: bool,
    pub battery: f32,
    pub flight_mode: DroneFlightMode,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum DroneType {
    Friendly,
    Hostile,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum DroneFlightMode {
    Manual,
    Stabilized,
    AltitudeHold,
    PositionHold,
    Waypoint,
}

impl DroneType {
    pub fn to_threat_color(&self) -> Color {
        match self {
            DroneType::Friendly => Color::srgb(0.0, 0.8, 0.0),
            DroneType::Hostile => Color::srgb(0.8, 0.1, 0.1),
            DroneType::Unknown => Color::srgb(0.8, 0.6, 0.1),
        }
    }

    pub fn to_emissive(&self) -> LinearRgba {
        match self {
            DroneType::Friendly => LinearRgba::new(0.0, 0.4, 0.0, 1.0),
            DroneType::Hostile => LinearRgba::new(0.4, 0.0, 0.0, 1.0),
            DroneType::Unknown => LinearRgba::new(0.4, 0.3, 0.0, 1.0),
        }
    }
}

#[derive(Component)]
#[allow(dead_code)]
pub struct DroneVisual {
    pub drone_id: String,
    pub drone_type: DroneType,
    pub rotor_angle: f32,
}

#[derive(Component)]
pub struct DroneTrail {
    pub points: VecDeque<Vec3>,
    pub max_points: usize,
}

fn create_drone_mesh(
    drone_type: &DroneType,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>) {
    let color = drone_type.to_threat_color();
    let emissive = drone_type.to_emissive();

    (
        Mesh3d(meshes.add(Cuboid::new(0.3, 0.1, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            emissive,
            metallic: 0.7,
            ..default()
        })),
    )
}

pub fn spawn_drone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    id: &str,
    drone_type: DroneType,
    position: Vec3,
) {
    let (mesh3d, mat3d) = create_drone_mesh(&drone_type, meshes, materials);

    let mut trail = VecDeque::with_capacity(DRONE_TRAIL_LENGTH);
    trail.push_back(position);

    commands.spawn((
        mesh3d,
        mat3d,
        Transform::from_translation(position),
        DroneVisual {
            drone_id: id.to_string(),
            drone_type,
            rotor_angle: 0.0,
        },
        DroneTrail {
            points: trail,
            max_points: DRONE_TRAIL_LENGTH,
        },
    ));
}

pub fn update_drone_positions(
    mut drone_query: Query<(&mut Transform, &DroneVisual, &mut DroneTrail)>,
    time: Res<Time>,
) {
    let bob = (time.elapsed_secs() * 2.0).sin() * 0.0002;
    for (mut transform, _drone, mut trail) in drone_query.iter_mut() {
        transform.translation.y += bob;
        trail.points.push_back(transform.translation);
        while trail.points.len() > trail.max_points {
            trail.points.pop_front();
        }
    }
}

pub fn update_or_create_drone_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drone_query: Query<(Entity, &mut Transform, &DroneVisual)>,
    registry: Res<DroneRegistry>,
) {
    if !registry.is_changed() {
        return;
    }

    let mut existing: HashMap<String, (Entity, bool)> = HashMap::new();
    for (entity, _, drone_visual) in drone_query.iter() {
        existing.insert(drone_visual.drone_id.clone(), (entity, false));
    }

    for (id, state) in registry.drones.iter() {
        if let Some(entry) = existing.get_mut(id) {
            entry.1 = true;
            if let Ok((_, mut transform, _)) = drone_query.get_mut(entry.0) {
                transform.translation = state.position;
                transform.rotation = state.orientation;
            }
        } else {
            spawn_drone(
                &mut commands,
                &mut meshes,
                &mut materials,
                id,
                state.drone_type.clone(),
                state.position,
            );
        }
    }

    let to_despawn: HashSet<Entity> = existing
        .iter()
        .filter(|(_, &(_, in_registry))| !in_registry)
        .map(|(_, &(e, _))| e)
        .collect();

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}
use bevy::prelude::*;

#[derive(Component)]
pub struct TacticalCamera {
    pub speed: f32,
    pub sprint_multiplier: f32,
    pub precision_multiplier: f32,
    #[allow(dead_code)]
    pub rotation_speed: f32,
    pub zoom_speed: f32,
}

impl Default for TacticalCamera {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sprint_multiplier: 3.0,
            precision_multiplier: 0.2,
            rotation_speed: 0.003,
            zoom_speed: 1.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub sprint: bool,
    pub precision: bool,
}

pub struct TacticalCameraPlugin;

impl Plugin for TacticalCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraInput>()
            .add_systems(Update, (
                keyboard_input_system,
                camera_movement_system,
                camera_zoom_system,
            ));
    }
}

fn keyboard_input_system(
    mut input: ResMut<CameraInput>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    input.forward = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    input.backward = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
    input.left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft);
    input.right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight);
    input.up = keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::Space);
    input.down = keys.pressed(KeyCode::KeyE);
    input.sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    input.precision = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
}

fn camera_movement_system(
    time: Res<Time>,
    input: Res<CameraInput>,
    mut camera_query: Query<(&TacticalCamera, &GlobalTransform, &mut Transform)>,
) {
    if !input.forward && !input.backward && !input.left && !input.right && !input.up && !input.down {
        return;
    }

    let dt = time.delta_secs();

    for (config, global_transform, mut transform) in camera_query.iter_mut() {
        let mut speed = config.speed;

        if input.sprint {
            speed *= config.sprint_multiplier;
        }
        if input.precision {
            speed *= config.precision_multiplier;
        }

        let forward = global_transform.forward();
        let right = global_transform.right();

        let mut movement = Vec3::ZERO;

        if input.forward {
            movement += *forward;
        }
        if input.backward {
            movement -= *forward;
        }
        if input.left {
            movement -= *right;
        }
        if input.right {
            movement += *right;
        }
        if input.up {
            movement += Vec3::Y;
        }
        if input.down {
            movement -= Vec3::Y;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }

        transform.translation += movement * speed * dt;
    }
}

fn camera_zoom_system(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    camera_query: Query<&TacticalCamera>,
    mut projection_query: Query<&mut Projection, With<TacticalCamera>>,
) {
    let config = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let zoom_speed = config.zoom_speed;
    for event in scroll_events.read() {
        for mut projection in projection_query.iter_mut() {
            if let Projection::Perspective(ref mut persp) = *projection {
                persp.fov = (persp.fov - event.y * 0.05 * zoom_speed).clamp(0.1, std::f32::consts::FRAC_PI_2);
            }
        }
    }
}
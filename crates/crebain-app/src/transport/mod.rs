use bevy::prelude::*;
use crebain_core::transport;

pub mod events;

pub use events::*;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransportState>()
            .add_event::<CameraFrameEvent>()
            .add_event::<ImuDataEvent>()
            .add_event::<PoseDataEvent>()
            .add_event::<ModelStateEvent>()
            .add_event::<TransportConnectedEvent>()
            .add_event::<TransportDisconnectedEvent>()
            .add_event::<TransportErrorEvent>()
            .add_systems(Update, update_transport_state);
    }
}

#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct TransportState {
    pub connected: bool,
    pub subscriptions: Vec<String>,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub stats: Option<transport::TransportStats>,
}

#[allow(clippy::too_many_arguments)]
fn update_transport_state(
    mut state: ResMut<TransportState>,
    mut connected_events: EventReader<TransportConnectedEvent>,
    mut disconnected_events: EventReader<TransportDisconnectedEvent>,
    mut error_events: EventReader<TransportErrorEvent>,
    mut model_events: EventReader<ModelStateEvent>,
    mut camera_events: EventReader<CameraFrameEvent>,
    mut imu_events: EventReader<ImuDataEvent>,
    mut pose_events: EventReader<PoseDataEvent>,
) {
    for _ in connected_events.read() {
        state.connected = true;
        log::info!("[Transport] Connected");
    }
    for _ in disconnected_events.read() {
        state.connected = false;
        log::info!("[Transport] Disconnected");
    }
    for err in error_events.read() {
        log::error!("[Transport] Error: {}", err.message);
    }

    let count = model_events.read().count()
        + camera_events.read().count()
        + imu_events.read().count()
        + pose_events.read().count();
    if count > 0 {
        state.messages_received += count as u64;
    }
}
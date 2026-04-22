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
            .add_event::<TransportErrorEvent>();
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
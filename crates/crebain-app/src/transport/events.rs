use bevy::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct CameraFrameEvent {
    pub topic: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub timestamp: f64,
    pub frame_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct ImuDataEvent {
    pub topic: String,
    pub orientation: [f64; 4],
    pub angular_velocity: [f64; 3],
    pub linear_acceleration: [f64; 3],
    pub timestamp: f64,
}

#[derive(Event, Clone, Debug)]
pub struct PoseDataEvent {
    pub topic: String,
    pub position: [f64; 3],
    pub orientation: [f64; 4],
    pub timestamp: f64,
    pub frame_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct ModelStateEvent {
    pub topic: String,
    pub names: Vec<String>,
    pub positions: Vec<[f64; 3]>,
    pub orientations: Vec<[f64; 4]>,
    pub velocities: Vec<[f64; 3]>,
}

#[derive(Event, Clone, Debug)]
pub struct TransportConnectedEvent;

#[derive(Event, Clone, Debug)]
pub struct TransportDisconnectedEvent;

#[derive(Event, Clone, Debug)]
pub struct TransportErrorEvent {
    pub message: String,
}
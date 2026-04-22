use tokio::sync::{Mutex, broadcast};
use super::{
    create_bridge, CameraFrame, CameraInfoData, ImuData, ModelStates, PoseData, Transport,
    TransportStats, TwistStampedData, VelocityCmd,
};

lazy_static::lazy_static! {
    static ref TRANSPORT_ENGINE: Mutex<Option<Box<dyn Transport>>> = Mutex::new(None);
}

pub type TransportEvent = TransportEventType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TransportEventType {
    CameraFrame { topic: String, frame: CameraFrame },
    CameraInfo { topic: String, info: Box<CameraInfoData> },
    ImuData { topic: String, data: ImuData },
    PoseData { topic: String, data: PoseData },
    ModelStates { topic: String, data: ModelStates },
}

lazy_static::lazy_static! {
    static ref EVENT_SENDER: broadcast::Sender<TransportEventType> = {
        let (tx, _) = broadcast::channel(1024);
        tx
    };
}

pub fn event_receiver() -> broadcast::Receiver<TransportEventType> {
    EVENT_SENDER.subscribe()
}

pub async fn transport_connect() -> Result<(), String> {
    log::info!("Connecting to transport layer...");

    let mut bridge = create_bridge().await.map_err(|e| e.to_string())?;
    bridge.connect().await.map_err(|e| e.to_string())?;

    let mut guard = TRANSPORT_ENGINE.lock().await;
    *guard = Some(bridge);

    log::info!("Transport connected successfully");
    Ok(())
}

pub async fn transport_disconnect() -> Result<(), String> {
    log::info!("Disconnecting transport...");

    let mut guard = TRANSPORT_ENGINE.lock().await;

    if let Some(bridge) = guard.as_mut() {
        bridge.disconnect().await.map_err(|e| e.to_string())?;
    }

    *guard = None;
    Ok(())
}

pub async fn transport_subscribe_camera(topic: &str) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    let event_topic = topic.to_string();
    let sender = EVENT_SENDER.clone();

    let callback = Box::new(move |frame: CameraFrame| {
        let _ = sender.send(TransportEventType::CameraFrame {
            topic: event_topic.clone(),
            frame,
        });
    });

    bridge.subscribe_camera(topic, callback).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_subscribe_camera_info(topic: &str) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    let event_topic = topic.to_string();
    let sender = EVENT_SENDER.clone();

    let callback = Box::new(move |info: CameraInfoData| {
        let _ = sender.send(TransportEventType::CameraInfo {
            topic: event_topic.clone(),
            info: Box::new(info),
        });
    });

    bridge.subscribe_camera_info(topic, callback).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_subscribe_imu(topic: &str) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    let event_topic = topic.to_string();
    let sender = EVENT_SENDER.clone();

    let callback = Box::new(move |data: ImuData| {
        let _ = sender.send(TransportEventType::ImuData {
            topic: event_topic.clone(),
            data,
        });
    });

    bridge.subscribe_imu(topic, callback).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_subscribe_pose(topic: &str) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    let event_topic = topic.to_string();
    let sender = EVENT_SENDER.clone();

    let callback = Box::new(move |data: PoseData| {
        let _ = sender.send(TransportEventType::PoseData {
            topic: event_topic.clone(),
            data,
        });
    });

    bridge.subscribe_pose(topic, callback).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_subscribe_model_states(topic: &str) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    let event_topic = topic.to_string();
    let sender = EVENT_SENDER.clone();

    let callback = Box::new(move |data: ModelStates| {
        let _ = sender.send(TransportEventType::ModelStates {
            topic: event_topic.clone(),
            data,
        });
    });

    bridge.subscribe_model_states(topic, callback).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_unsubscribe(topic: &str) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;
    bridge.unsubscribe(topic).await.map_err(|e| e.to_string())
}

pub async fn transport_publish_velocity(topic: &str, cmd: VelocityCmd) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    bridge.publish_velocity(topic, cmd).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_publish_twist_stamped(topic: &str, cmd: TwistStampedData) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    bridge.publish_twist_stamped(topic, cmd).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_publish_pose(topic: &str, pose: PoseData) -> Result<(), String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    bridge.publish_pose(topic, pose).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn transport_get_stats() -> Result<TransportStats, String> {
    let guard = TRANSPORT_ENGINE.lock().await;
    let bridge = guard.as_ref().ok_or("Transport not connected")?;

    Ok(bridge.stats())
}
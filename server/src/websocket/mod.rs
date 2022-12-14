use home_automation_common::websocket::dto::{AutomationMessage, ClientDeviceType};
use std::sync::Arc;

use crate::ServicesContext;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::websocket::dto::{
    AutomationServerStatusUpdate, AutomationServerWebsocketMessage, MessageDistribution,
    WebsocketEvent,
};
use crate::websocket::handler::WebsocketEventHandler;
use crate::websocket::server::WebsocketServer;

pub mod convert;
pub mod dto;
pub mod handler;
pub mod route;
pub mod server;

struct ClientState {
    connected_since: chrono::DateTime<chrono::Utc>,
    name: String,
    macros_executed: u32,
    device_type: ClientDeviceType,
}

impl ClientState {
    fn new() -> ClientState {
        ClientState {
            name: "".to_owned(),
            connected_since: chrono::Utc::now(),
            macros_executed: 0,
            device_type: ClientDeviceType::Desktop,
        }
    }
}

pub async fn setup(
    services_context: Arc<ServicesContext>,
    websocket_server: Arc<tokio::sync::Mutex<WebsocketServer>>,
    status_update_rx: UnboundedReceiver<AutomationServerStatusUpdate>,
    websocket_event_rx: UnboundedReceiver<WebsocketEvent>,
) {
    let websocket_event_handler = WebsocketEventHandler::new(services_context);

    let websocket_ping_server = websocket_server;
    // ws ping
    let websocket_ping_task = server::ping_websocket_clients(websocket_ping_server.clone());
    tokio::spawn(websocket_ping_task);
    // ws status update
    let websocket_status_update_task =
        create_status_update_task(websocket_ping_server.clone(), status_update_rx);
    tokio::spawn(websocket_status_update_task);
    // ws handle message sending
    let (websocket_message_tx, websocket_message_rx) = tokio::sync::mpsc::unbounded_channel();
    let websocket_message_send_task =
        create_send_message_task(websocket_ping_server, websocket_message_rx);
    tokio::spawn(websocket_message_send_task);
    // ws handle events
    let websocket_event_handler_task = create_handle_event_task(
        websocket_event_handler,
        websocket_event_rx,
        websocket_message_tx,
    );
    tokio::spawn(websocket_event_handler_task);
}

async fn create_status_update_task(
    server: Arc<tokio::sync::Mutex<WebsocketServer>>,
    mut status_update_rx: UnboundedReceiver<AutomationServerStatusUpdate>,
) {
    while let Some(status_update) = status_update_rx.recv().await {
        let message = AutomationMessage::StatusUpdate {
            update: status_update.update,
        };
        send_message(server.clone(), message, status_update.distribution).await
    }
}

async fn create_handle_event_task(
    mut websocket_event_handler: WebsocketEventHandler,
    event_receiver: UnboundedReceiver<WebsocketEvent>,
    websocket_message_sender: UnboundedSender<AutomationServerWebsocketMessage>,
) {
    websocket_event_handler
        .handle_events(event_receiver, websocket_message_sender)
        .await
}

async fn create_send_message_task(
    server: Arc<tokio::sync::Mutex<WebsocketServer>>,
    mut websocket_message_rx: UnboundedReceiver<AutomationServerWebsocketMessage>,
) {
    while let Some(websocket_message) = websocket_message_rx.recv().await {
        let message = websocket_message.message;
        send_message(server.clone(), message, websocket_message.distribution).await
    }
}

async fn send_message(
    server: Arc<tokio::sync::Mutex<WebsocketServer>>,
    message: AutomationMessage,
    distribution: MessageDistribution,
) {
    let locked_server = server.lock().await;
    match distribution {
        MessageDistribution::Broadcast => {
            if let Err(err) = locked_server.broadcast_message(message).await {
                debug!("Error occurred when sending broadcast message: {}", err);
            }
        }
        MessageDistribution::SingleClient { client_id } => {
            if let Err(err) = locked_server
                .send_message_to_client(message, client_id)
                .await
            {
                debug!(
                    "Error occurred when sending message to single client with id {}: {}",
                    client_id, err
                );
            }
        }
    }
}

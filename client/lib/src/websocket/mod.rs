use std::sync::Arc;

use futures_util::sink::Sink;
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use home_automation_common::websocket::convert::{
    convert_message_to_text, parse_message_from_string,
};
use home_automation_common::websocket::dto::{
    AutomationMessage, ClientDeviceType, SingleClientUpdate,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite;

use crate::websocket::handler::AutomationStatusUpdateHandler;

pub mod handler;

#[derive(Clone)]
pub struct WebsocketClientInfo {
    pub client_type: ClientDeviceType,
    pub client_name: String,
}

pub struct WebsocketRunner {
    sender: UnboundedSender<AutomationMessage>,
    join_handle: JoinHandle<()>,
}

impl WebsocketRunner {
    pub fn new(
        client_info: WebsocketClientInfo,
        ws_server_url: String,
        message_handler: Arc<Mutex<dyn AutomationStatusUpdateHandler>>,
    ) -> WebsocketRunner {
        let (automation_message_tx, automation_message_rx) = tokio::sync::mpsc::unbounded_channel();

        let join_handle = tokio::spawn(Self::run_websocket(
            ws_server_url,
            client_info,
            automation_message_rx,
            message_handler,
            automation_message_tx.clone(),
        ));

        WebsocketRunner {
            sender: automation_message_tx,
            join_handle,
        }
    }

    pub async fn stop(self) {
        self.join_handle.await.unwrap_or_else(|err| {
            error!("Could not wait for stopped websocket runner: {}.", err);
        })
    }

    pub fn get_ws_sender(&self) -> UnboundedSender<AutomationMessage> {
        self.sender.clone()
    }

    async fn run_websocket(
        server_url: String,
        client_info: WebsocketClientInfo,
        automation_message_receiver: UnboundedReceiver<AutomationMessage>,
        bcp_message_handler: Arc<Mutex<dyn AutomationStatusUpdateHandler>>,
        ws_sender: UnboundedSender<AutomationMessage>,
    ) {
        let websocket_writer = Arc::new(Mutex::new(Option::None));

        // sender task
        tokio::spawn(Self::send_messages(
            automation_message_receiver,
            websocket_writer.clone(),
        ));

        match tokio_tungstenite::connect_async(&server_url).await {
            Ok((ws_stream, _)) => {
                let (ws_write, ws_read) = ws_stream.split();
                let mut locked_writer = websocket_writer.lock().await;
                *locked_writer = Some(ws_write);
                drop(locked_writer);

                // receiver task
                let receiver_handle = tokio::spawn(Self::handle_messages(
                    client_info.clone(),
                    ws_read,
                    bcp_message_handler.clone(),
                    ws_sender.clone(),
                ));

                receiver_handle.await.unwrap_or_else(|err| {
                    error!("Could not await receiver task: {}.", err);
                });
            }
            Err(err) => {
                error!("Could not connect to automation server: {}.", err);
            }
        }
    }

    async fn send_messages(
        mut automation_message_receiver: UnboundedReceiver<AutomationMessage>,
        websocket_message_writer: Arc<
            Mutex<
                Option<
                    impl Sink<tungstenite::Message, Error = tungstenite::Error> + std::marker::Unpin,
                >,
            >,
        >,
    ) {
        loop {
            match automation_message_receiver.recv().await {
                Some(message) => match convert_message_to_tungstenite_message(message) {
                    Ok(converted_message) => {
                        let mut locked_writer = websocket_message_writer.lock().await;
                        match *locked_writer {
                            Some(ref mut writer) => {
                                if let Err(err) = writer.send(converted_message).await {
                                    error!("Could not send message on websocket: {}.", err);
                                    break;
                                }
                            }
                            None => {
                                error!("No websocket writer found in optional value.");
                                break;
                            }
                        }
                    }
                    Err(err) => error!("Could not convert message to websocket message: {}", err),
                },
                None => {
                    error!("Could not receive message to send anymore.");
                    break;
                }
            }
        }
        info!("Sender task terminated.");
    }

    async fn handle_messages(
        client_info: WebsocketClientInfo,
        ws_read: impl Stream<Item = Result<tungstenite::Message, tungstenite::Error>>
            + std::marker::Unpin,
        message_handler: Arc<Mutex<dyn AutomationStatusUpdateHandler>>,
        ws_sender: UnboundedSender<AutomationMessage>,
    ) {
        let mut websocket_receiver = ws_read;
        loop {
            match websocket_receiver.next().await {
                Some(message_result) => match message_result {
                    Ok(message) => {
                        Self::handle_message(&client_info, message, &message_handler, &ws_sender)
                            .await;
                    }
                    Err(err) => {
                        error!(
                            "Error occurred while receiving message from server: {}.",
                            err
                        );
                        break;
                    }
                },
                None => {
                    warn!("No message could be received anymore from server.");
                    break;
                }
            }
        }
    }

    async fn handle_message(
        client_info: &WebsocketClientInfo,
        message: tungstenite::Message,
        message_handler: &Arc<Mutex<dyn AutomationStatusUpdateHandler>>,
        ws_sender: &UnboundedSender<AutomationMessage>,
    ) {
        match message.to_text() {
            Ok(text) => match parse_message_from_string(text) {
                Ok(message) => match message {
                    AutomationMessage::StatusUpdate { update } => {
                        let mut locked_message_handler = message_handler.lock().await;
                        locked_message_handler.on_status_update(update);
                    }
                    AutomationMessage::Ping => {
                        let message = AutomationMessage::Pong {
                            client_update: SingleClientUpdate {
                                name: client_info.client_name.clone(),
                                device_type: client_info.client_type.clone(),
                            },
                        };
                        if let Err(err) = ws_sender.send(message) {
                            error!("Could not send pong message from websocket: {}.", err);
                        }
                    }
                    _ => {}
                },
                Err(err) => error!("Could not parse message from message text: {}.", err),
            },
            Err(err) => error!("Could not convert message to text:  {}.", err),
        }
    }
}

fn convert_message_to_tungstenite_message(
    message: AutomationMessage,
) -> anyhow::Result<tungstenite::Message> {
    let text = convert_message_to_text(message)?;
    Ok(tungstenite::Message::Text(text))
}

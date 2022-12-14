use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::anyhow;
use futures::{FutureExt, StreamExt};
use home_automation_common::websocket::convert::parse_message_from_string;
use home_automation_common::websocket::dto::AutomationMessage;
use home_automation_common::websocket::MessageSender;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::Duration;

use crate::websocket::convert::convert_message_to_ws_message;
use crate::websocket::dto::WebsocketEvent;

const WS_PING_INTERVAL: Duration = Duration::from_secs(10);

pub async fn add_websocket_to_server(
    websocket: axum::extract::ws::WebSocket,
    server: Arc<tokio::sync::Mutex<WebsocketServer>>,
) -> usize {
    let mut locked_server = server.lock().await;
    locked_server.add_client_socket(websocket).await
}

pub async fn ping_websocket_clients(server: Arc<tokio::sync::Mutex<WebsocketServer>>) {
    let interval = tokio::time::interval(WS_PING_INTERVAL);
    let mut interval_stream = tokio_stream::wrappers::IntervalStream::new(interval);

    while interval_stream.next().await.is_some() {
        let locked_server = server.lock().await;
        if let Err(err) = locked_server.broadcast_ping().await {
            debug!("Error occurred during server broadcast ping: {}", err);
        }
    }
}

struct WebsocketTask {
    name: &'static str,
    join_handle: JoinHandle<()>,
}

impl WebsocketTask {
    async fn join_task(self) {
        let join_result = self.join_handle.await;
        if join_result.is_err() {
            error!("Could not join websocket task {}.", self.name);
        }
    }
}

struct WebsocketClientConnection {
    sender: tokio::sync::mpsc::UnboundedSender<Result<axum::extract::ws::Message, axum::Error>>,
    tasks: Vec<WebsocketTask>,
}

impl WebsocketClientConnection {
    async fn join_tasks(&mut self) {
        let drained_tasks: Vec<WebsocketTask> = self.tasks.drain(..).collect();
        for task in drained_tasks {
            task.join_task().await;
        }
    }
}

pub struct WebsocketServer {
    next_user_id: AtomicUsize,
    client_connections: Arc<RwLock<HashMap<usize, WebsocketClientConnection>>>,
    websocket_event_sender: UnboundedSender<WebsocketEvent>,
}

impl WebsocketServer {
    pub fn new(websocket_event_sender: UnboundedSender<WebsocketEvent>) -> WebsocketServer {
        WebsocketServer {
            websocket_event_sender,
            next_user_id: AtomicUsize::new(0),
            client_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_client_socket(&mut self, websocket: axum::extract::ws::WebSocket) -> usize {
        let next_id = self.next_user_id.fetch_add(1, Ordering::SeqCst);
        info!("Connected to new websocket client with id {}.", next_id);

        let (websocket_sink, websocket_stream) = websocket.split();

        // sending
        let (sending_tx, sending_rx) = tokio::sync::mpsc::unbounded_channel();
        let sending_rx_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(sending_rx);
        let sending_stream = sending_rx_stream.take_while(move |result| match result {
            Ok(_) => futures::future::ready(true),
            Err(err) => {
                error!(
                    "Websocket error on sending for client with id {}: {}.",
                    next_id, err
                );
                futures::future::ready(false)
            }
        });
        let send_task_join_handle =
            tokio::spawn(sending_stream.forward(websocket_sink).then(move |result| {
                if let Err(err) = result {
                    error!(
                        "Error occurred when forwarding send task for client with id {}: {}.",
                        next_id, err
                    );
                }
                futures::future::ready(())
            }));
        let send_task = WebsocketTask {
            name: "Websocket Client Sender Task",
            join_handle: send_task_join_handle,
        };

        // receiving
        let handle_messages_task_join_handle = tokio::spawn(WebsocketServer::handle_user_messages(
            next_id,
            websocket_stream,
            self.websocket_event_sender.clone(),
        ));
        let handle_messages_task = WebsocketTask {
            name: "Websocket Message Handler Task",
            join_handle: handle_messages_task_join_handle,
        };

        let client_connection = WebsocketClientConnection {
            sender: sending_tx,
            tasks: vec![send_task, handle_messages_task],
        };
        self.client_connections
            .write()
            .await
            .insert(next_id, client_connection);

        let event = WebsocketEvent::ClientConnected { client_id: next_id };
        if let Err(err) = self.websocket_event_sender.send(event) {
            error!(
                "Could not send connected event on websocket event sender: {}.",
                err
            );
        }

        next_id
    }

    async fn handle_user_messages(
        id: usize,
        mut stream: futures::stream::SplitStream<axum::extract::ws::WebSocket>,
        websocket_event_sender: UnboundedSender<WebsocketEvent>,
    ) {
        loop {
            match stream.next().await {
                Some(message_result) => {
                    match message_result {
                        Ok(message) => {
                            match message.to_text() {
                                Ok(text) => {
                                    match parse_message_from_string(text) {
                                        Ok(message) => {
                                            let event = WebsocketEvent::MessageReceived { client_id: id, message };
                                            if let Err(err) = websocket_event_sender.send(event) {
                                                error!("Could not send websocket event: {}.", err);
                                            }
                                        }
                                        Err(err) => {
                                            error!(                            "Could not parse text {} to websocket message: {}.",                text, err            );
                                        }
                                    }
                                }
                                Err(_) => debug!("Could not convert text websocket message to text for client with id {}.", id)
                            }
                        }
                        Err(err) => {
                            error!("Could not receive message for client with id {}: {}.", id, err);
                            break;
                        }
                    }
                }
                None => {
                    error!("Could not await new message on websocket.");
                    break;
                }
            }
        }
    }

    pub async fn send_message_to_client(
        &self,
        message: AutomationMessage,
        client_id: usize,
    ) -> anyhow::Result<()> {
        let converted_message = convert_message_to_ws_message(message)?;
        let users = self.client_connections.read().await;
        match users.get(&client_id) {
            Some(client_connection) => {
                Self::send_message_on_connection(&converted_message, client_connection)
            }
            None => Err(anyhow!(
                "Could not send message. No client with id {} was found.",
                client_id
            )),
        }
    }

    pub async fn broadcast_message(&self, message: AutomationMessage) -> anyhow::Result<()> {
        let converted_message = convert_message_to_ws_message(message)?;
        let users = self.client_connections.read().await;
        for client_connection in users.values() {
            Self::send_message_on_connection(&converted_message, client_connection)?;
        }
        Ok(())
    }

    fn send_message_on_connection(
        message: &axum::extract::ws::Message,
        client_connection: &WebsocketClientConnection,
    ) -> anyhow::Result<()> {
        client_connection.sender.send(Ok(message.clone()))?;
        Ok(())
    }

    pub async fn broadcast_ping(&self) -> anyhow::Result<()> {
        let converted_message = convert_message_to_ws_message(AutomationMessage::Ping)?;
        let mut concat_error_message: Option<String> = None;

        let mut ids_to_remove: Vec<usize> = Vec::new();

        let users = self.client_connections.read().await;
        for (user_id, client_connection) in users.iter() {
            if let Err(err) = client_connection.sender.send(Ok(converted_message.clone())) {
                let error_message = format!(
                    "Could not send ping for client with id {}: {}.\n ",
                    user_id, err
                );
                match concat_error_message {
                    Some(mut msg) => {
                        msg.push_str(error_message.as_str());
                        concat_error_message = Some(msg)
                    }
                    None => concat_error_message = Some(error_message.to_owned()),
                }
                ids_to_remove.push(*user_id);
            }
        }
        drop(users);

        if !ids_to_remove.is_empty() {
            let mut client_connections = self.client_connections.write().await;
            for id in ids_to_remove {
                debug!("Removing websocket with id {}.", id);
                match client_connections.get_mut(&id) {
                    Some(client_connection) => {
                        client_connection.join_tasks().await;
                    }
                    None => error!(
                        "Could not find client connection with id {} in connected clients.",
                        id
                    ),
                }
                client_connections.remove(&id);
                let event = WebsocketEvent::ClientDisconnected { client_id: id };
                if let Err(err) = self.websocket_event_sender.send(event) {
                    error!(
                        "Could not send disconnected event on websocket event sender: {}.",
                        err
                    );
                }
                warn!("Removed websocket with id {}.", id);
            }
        }

        match concat_error_message {
            Some(msg) => Err(anyhow!("Error on ping broadcast: {}", msg)),
            None => Ok(()),
        }
    }
}

struct ServerWebsocketSender {
    sender: UnboundedSender<Result<axum::extract::ws::Message, axum::Error>>,
}

impl MessageSender for ServerWebsocketSender {
    fn send_message(&mut self, message: AutomationMessage) -> anyhow::Result<()> {
        let ws_message = convert_message_to_ws_message(message)?;
        self.sender.send(Ok(ws_message))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::Mutex;

    use axum::extract::{Extension, WebSocketUpgrade};
    use axum::response::IntoResponse;
    use axum::Router;
    use axum_server::Handle;
    use home_automation_common::action::{AutomationAction, AutomationStatusUpdate};
    use home_automation_common::automacro::AutomationMacro;
    use home_automation_common::websocket::convert::convert_message_to_text;
    use tokio::sync::mpsc::UnboundedReceiver;
    use tokio::sync::oneshot::Receiver;

    use crate::websocket::server;

    use super::*;

    type ShutdownSignalSender = Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>;

    async fn wait_for_expected_message(
        expected_message: AutomationMessage,
        mut event_receiver: UnboundedReceiver<WebsocketEvent>,
        shutdown_signal_sender: ShutdownSignalSender,
    ) {
        while let Some(event) = event_receiver.recv().await {
            match event {
                WebsocketEvent::MessageReceived {
                    message,
                    client_id: _,
                } => {
                    let success = message.eq(&expected_message);

                    if success {
                        let sender = shutdown_signal_sender.lock().unwrap().take();
                        if let Some(sender) = sender {
                            sender
                                .send(())
                                .expect("Could not send signal to shut down server.");
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn convert_message_to_tungstenite_message(
        message: AutomationMessage,
    ) -> anyhow::Result<tokio_tungstenite::tungstenite::Message> {
        let text = convert_message_to_text(message)?;
        Ok(tokio_tungstenite::tungstenite::Message::Text(text))
    }

    #[tokio::test]
    async fn test_websocket_integration() {
        let ping_message = AutomationMessage::StatusUpdate {
            update: AutomationStatusUpdate::SoundPlayed {
                sound: "test".to_string(),
            },
        };
        let pong_message = AutomationMessage::ExecuteMacro {
            mac: AutomationMacro::new("Play sound".to_owned(), vec![AutomationAction::PlaySound]),
        };

        let url = [127, 0, 0, 1];

        let (server_shutdown_tx, server_shutdown_rx) = tokio::sync::oneshot::channel();
        let server_shutdown_tx = Arc::new(Mutex::new(Some(server_shutdown_tx)));

        let (websocket_event_tx, websocket_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let websocket_event_handler_join_handle = tokio::spawn(wait_for_expected_message(
            pong_message.clone(),
            websocket_event_rx,
            server_shutdown_tx,
        ));

        let websocket_server = Arc::new(tokio::sync::Mutex::new(WebsocketServer::new(
            websocket_event_tx,
        )));

        let router = Router::new()
            .route("/", axum::routing::get(ws_handler))
            .layer(axum::extract::Extension(websocket_server.clone()));

        let handle = axum_server::Handle::new();
        let addr = SocketAddr::from((url, 0));
        let server_task = axum_server::bind(addr)
            .handle(handle.clone())
            .serve(router.into_make_service());
        tokio::spawn(shutdown_server(server_shutdown_rx, handle.clone()));
        let server_join_handle = tokio::spawn(server_task);

        let port = handle.listening().await.unwrap().port();

        let ws_url = format!("ws://{}.{}.{}.{}:{}", url[0], url[1], url[2], url[3], port);
        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url)
            .await
            .expect("Could not connect to websocket.");
        let (write, read) = ws_stream.split();

        let client_ping_message = ping_message.clone();
        let client_pong_message = pong_message.clone();
        let client_task = read
            .take(1)
            .map(move |item| {
                let ping_message = client_ping_message.clone();
                if let Ok(item) = item {
                    if item.is_text() {
                        if let Ok(message) = parse_message_from_string(item.to_text().unwrap()) {
                            if message.eq(&ping_message) {
                                return client_pong_message.clone();
                            }
                        }
                    }
                }
                panic!("Invalid message received.");
            })
            .map(|message| convert_message_to_tungstenite_message(message))
            .filter_map(|parsing_result| {
                let result = match parsing_result {
                    Ok(message) => Some(Ok(message)),
                    Err(_) => None,
                };
                futures::future::ready(result)
            })
            .forward(write);
        let client_join_handle = tokio::spawn(client_task);

        websocket_server
            .lock()
            .await
            .broadcast_message(ping_message)
            .await
            .expect("Could not broadcast ping message.");

        server_join_handle
            .await
            .expect("Could not join server.")
            .expect("Could not shut down server successfully.");
        client_join_handle
            .await
            .expect("Could not join client")
            .expect("Could not shut down client successfully.");
        websocket_event_handler_join_handle
            .await
            .expect("Could not join event handler.");
    }

    async fn shutdown_server(
        server_shutdown_rx: tokio::sync::oneshot::Receiver<()>,
        server_handle: Handle,
    ) {
        server_shutdown_rx.await.unwrap();
        server_handle.shutdown();
    }

    async fn ws_handler(
        ws: WebSocketUpgrade,
        Extension(websocket_server): Extension<Arc<tokio::sync::Mutex<WebsocketServer>>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(|websocket| async {
            server::add_websocket_to_server(websocket, websocket_server).await;
        })
    }
}

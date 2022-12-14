use crate::{AutomationModule, ServicesContext, WebsocketServer};
use axum::extract::ws::WebSocket;
use axum::extract::{Extension, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(context_data): Extension<Arc<ServicesContext>>,
    Extension(websocket_server_data): Extension<Arc<tokio::sync::Mutex<WebsocketServer>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|websocket| handle_socket(websocket, context_data, websocket_server_data))
}

pub async fn handle_socket(
    socket: WebSocket,
    context_data: Arc<ServicesContext>,
    websocket_server_data: Arc<tokio::sync::Mutex<WebsocketServer>>,
) {
    let client_id =
        crate::websocket::server::add_websocket_to_server(socket, websocket_server_data).await;
    // send initial status updates
    let locked_modules = context_data.modules.lock().unwrap();
    locked_modules
        .send_initial_state(client_id)
        .unwrap_or_else(|err| error!("Could not send status updates: {}.", err));
}

use home_automation_common::action::AutomationAction;
use home_automation_common::websocket::dto::AutomationMessage;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::automodule::AutomationModule;
use crate::services::ServicesContext;
use crate::websocket::dto::{
    AutomationServerWebsocketMessage, MessageDistribution, WebsocketEvent,
};
use crate::websocket::ClientState;

pub struct WebsocketEventHandler {
    context: Arc<ServicesContext>,
    client_states: HashMap<usize, ClientState>,
}

impl WebsocketEventHandler {
    pub fn new(context: Arc<ServicesContext>) -> WebsocketEventHandler {
        WebsocketEventHandler {
            context,
            client_states: HashMap::new(),
        }
    }

    pub async fn handle_events(
        &mut self,
        mut event_receiver: UnboundedReceiver<WebsocketEvent>,
        websocket_message_sender: UnboundedSender<AutomationServerWebsocketMessage>,
    ) {
        while let Some(event) = event_receiver.recv().await {
            match event {
                WebsocketEvent::ClientConnected { client_id } => {
                    let new_client_state = ClientState::new();
                    let previous_state = self.client_states.insert(client_id, new_client_state);
                    if previous_state.is_some() {
                        warn!("Overwriting client state for client with id {}.", client_id);
                    }
                }
                WebsocketEvent::ClientDisconnected { client_id } => {
                    self.client_states.remove(&client_id);
                }
                WebsocketEvent::MessageReceived { client_id, message } => {
                    self.handle_automation_message(client_id, message, &websocket_message_sender);
                }
            }
        }
    }

    fn handle_automation_message(
        &mut self,
        client_id: usize,
        message: AutomationMessage,
        message_sender: &UnboundedSender<AutomationServerWebsocketMessage>,
    ) {
        self.update_client_state(client_id, &message);

        trace!("Got message: {:?}", &message);

        match message {
            AutomationMessage::ExecuteMacro { mac } => {
                self.handle_actions(mac.actions);
            }
            AutomationMessage::RequestClientStates => {
                let states = self
                    .client_states
                    .values()
                    .map(
                        |client_state| home_automation_common::websocket::dto::ClientState {
                            name: client_state.name.clone(),
                            macros_executed: client_state.macros_executed,
                            connected_since: client_state.connected_since.to_rfc3339(),
                            device_type: client_state.device_type.clone(),
                        },
                    )
                    .collect::<Vec<home_automation_common::websocket::dto::ClientState>>();
                let response_message = AutomationMessage::ClientStates { states };
                let websocket_message = AutomationServerWebsocketMessage {
                    message: response_message,
                    distribution: MessageDistribution::SingleClient { client_id },
                };
                if let Err(err) = message_sender.send(websocket_message) {
                    error!("Could not send client state response message: {}.", err);
                }
            }
            _ => {}
        }
    }

    fn execute_action(services_context: &Arc<ServicesContext>, action: AutomationAction) {
        let mut modules = services_context.modules.lock().unwrap();
        match modules.handle_action(&action) {
            Ok(handled) => {
                if !handled {
                    warn!("Action was not handled by any module.");
                }
            }
            Err(err) => error!("Error occurred while handling action: {}.", err),
        }
    }

    fn handle_actions(&self, actions: Vec<AutomationAction>) {
        for action in actions {
            Self::execute_action(&self.context, action);
        }
    }

    fn update_client_state(&mut self, client_id: usize, message: &AutomationMessage) {
        match self.client_states.get_mut(&client_id) {
            Some(client_state) => match message {
                AutomationMessage::ExecuteMacro { .. } => {
                    client_state.macros_executed += 1;
                }
                AutomationMessage::Pong { client_update } => {
                    client_state.name = client_update.name.clone();
                    client_state.device_type = client_update.device_type.clone();
                }
                _ => {}
            },
            None => error!("No client state for client with id {} found.", client_id),
        }
    }
}

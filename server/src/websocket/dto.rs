use home_automation_common::action::AutomationStatusUpdate;
use home_automation_common::websocket::dto::AutomationMessage;

#[allow(clippy::large_enum_variant)]
pub enum WebsocketEvent {
    MessageReceived {
        client_id: usize,
        message: AutomationMessage,
    },
    ClientConnected {
        client_id: usize,
    },
    ClientDisconnected {
        client_id: usize,
    },
}

pub struct AutomationServerWebsocketMessage {
    pub message: AutomationMessage,
    pub distribution: MessageDistribution,
}

pub enum MessageDistribution {
    Broadcast,
    SingleClient { client_id: usize },
}

pub struct AutomationServerStatusUpdate {
    pub update: AutomationStatusUpdate,
    pub distribution: MessageDistribution,
}

impl AutomationServerStatusUpdate {
    pub fn broadcast(update: AutomationStatusUpdate) -> AutomationServerStatusUpdate {
        AutomationServerStatusUpdate {
            update,
            distribution: MessageDistribution::Broadcast,
        }
    }

    pub fn single_client(
        update: AutomationStatusUpdate,
        client: usize,
    ) -> AutomationServerStatusUpdate {
        AutomationServerStatusUpdate {
            update,
            distribution: MessageDistribution::SingleClient { client_id: client },
        }
    }
}

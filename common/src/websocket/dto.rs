use crate::action::AutomationStatusUpdate;
use crate::automacro::AutomationMacro;

#[derive(Serialize, Deserialize)]
pub struct MessageHeader {
    pub version: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SingleClientUpdate {
    pub name: String,
    pub device_type: ClientDeviceType,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ClientDeviceType {
    Desktop,
    Panel,
    Streamdeck,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientState {
    pub connected_since: String,
    pub name: String,
    pub macros_executed: u32,
    pub device_type: ClientDeviceType,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "tag", content = "payload")]
pub enum AutomationMessage {
    Ping,
    Pong {
        #[serde(rename = "clientUpdate")]
        client_update: SingleClientUpdate,
    },
    ExecuteMacro {
        #[serde(rename = "macro")]
        mac: AutomationMacro,
    },
    StatusUpdate {
        update: AutomationStatusUpdate,
    },
    RequestClientStates,
    ClientStates {
        states: Vec<ClientState>,
    },
}

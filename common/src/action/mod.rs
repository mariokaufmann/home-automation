use crate::automodule::streamdeck::StreamdeckDevicesConfiguration;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "tag", content = "payload")]
pub enum AutomationAction {
    PlaySound,
    /* Can be sent to instruct the server to reload a certain streamdeck's configuration and send it to the connected streamdeck. */
    StreamdeckClientReloadDeviceConfiguration,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "tag", content = "payload")]
pub enum AutomationStatusUpdate {
    // TODO remove
    SoundPlayed { sound: String },
    StreamdeckClientReloadedDevicesConfiguration(StreamdeckDevicesConfiguration),
}

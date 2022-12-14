use crate::automacro::AutomationMacro;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StreamdeckDevicesConfiguration {
    pub devices: Vec<StreamdeckDeviceConfiguration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StreamdeckDeviceConfiguration {
    pub device_id: String,
    pub configuration: StreamdeckAutomationConfiguration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StreamdeckAutomationConfiguration {
    pub device_name: String,
    pub button_configurations: Vec<StreamdeckButtonConfiguration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StreamdeckButtonConfiguration {
    pub key: u8,
    pub text: String,
    pub press_macro: AutomationMacro,
    pub release_macro: Option<AutomationMacro>,
}

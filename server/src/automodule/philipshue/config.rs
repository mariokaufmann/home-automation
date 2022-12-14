pub const CONFIG_FILE_NAME: &str = "philipsHueConfig.json";

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhilipsHueAutomationModuleConfiguration {
    pub bridge_ip: String,
    pub username: String,
}

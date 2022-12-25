pub const CONFIG_FILE_NAME: &str = "philipsHueConfig.json";

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct PhilipsHueAutomationModuleConfiguration {
    pub bridge_ip: String,
    pub api_key: String,
    pub groups: Vec<PhilipsHueGroupConfiguration>,
    pub presets: Vec<PhilipsHuePresetConfiguration>,
}

impl PhilipsHueAutomationModuleConfiguration {
    pub fn find_group(&self, group_id: &str) -> Option<&PhilipsHueGroupConfiguration> {
        self.groups.iter().find(|group| group.id.eq(group_id))
    }

    pub fn find_preset(&self, preset_id: &str) -> Option<&PhilipsHuePresetConfiguration> {
        self.presets.iter().find(|preset| preset.id.eq(preset_id))
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct PhilipsHueGroupConfiguration {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct PhilipsHuePresetConfiguration {
    pub id: String,
    pub on: bool,
    pub brightness: u16,
    pub color_temperature: u16,
}

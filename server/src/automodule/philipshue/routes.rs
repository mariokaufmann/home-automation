use tokio::sync::mpsc::UnboundedSender;
use crate::automodule::philipshue::ConfigureHuePayload;
use ts::TS;

pub struct HueState {
    request_sender: UnboundedSender<ConfigureHuePayload>,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureLightDto {
    light: u16,
    brightness: u16,
    saturation: u16,
    hue: u16,
    color_temperature: u16,
}

async fn configure_light()
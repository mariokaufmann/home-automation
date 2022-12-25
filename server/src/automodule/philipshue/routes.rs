use crate::automodule::philipshue::api::ConfigureHueGroupedLightRequest;
use crate::automodule::philipshue::config::PhilipsHueAutomationModuleConfiguration;
use axum::extract::State;
use axum::Json;
use hyper::StatusCode;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use ts_rs::TS;

#[derive(Clone)]
pub struct HueState {
    pub(super) request_sender: UnboundedSender<ConfigureHueGroupedLightRequest>,
    pub(super) configuration: Arc<PhilipsHueAutomationModuleConfiguration>,
}

#[derive(Serialize, Deserialize, TS)]
pub struct PhilipsHueConfigureGroupDto {
    group_id: String,
    preset_id: String,
}

#[derive(Serialize, Deserialize, TS)]
pub struct PhilipsHueGroupDto {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, TS)]
pub struct PhilipsHuePresetDto {
    id: String,
    on: bool,
    brightness: u16,
    color_temperature: u16,
}

#[axum_macros::debug_handler]
pub async fn get_groups(State(state): State<HueState>) -> Json<Vec<PhilipsHueGroupDto>> {
    Json(
        state
            .configuration
            .groups
            .iter()
            .map(|group| PhilipsHueGroupDto {
                id: group.id.clone(),
                name: group.name.clone(),
            })
            .collect(),
    )
}

#[axum_macros::debug_handler]
pub async fn get_presets(State(state): State<HueState>) -> Json<Vec<PhilipsHuePresetDto>> {
    Json(
        state
            .configuration
            .presets
            .iter()
            .map(|preset| PhilipsHuePresetDto {
                id: preset.id.clone(),
                on: preset.on,
                brightness: preset.brightness,
                color_temperature: preset.color_temperature,
            })
            .collect(),
    )
}

#[axum_macros::debug_handler]
pub async fn configure_group(
    State(state): State<HueState>,
    Json(dto): Json<PhilipsHueConfigureGroupDto>,
) -> Result<(), StatusCode> {
    let group = state.configuration.find_group(&dto.group_id);
    let preset = state.configuration.find_preset(&dto.preset_id);

    if let Some(group) = group {
        if let Some(preset) = preset {
            let payload = ConfigureHueGroupedLightRequest {
                id: group.id.clone(),
                on: preset.on,
                brightness: preset.brightness,
                color_temperature: preset.color_temperature,
            };

            match state.request_sender.send(payload) {
                Ok(()) => Ok(()),
                Err(err) => {
                    error!(
                        "Could not send request to philips hue request sender: {}.",
                        err
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            warn!("Preset {} was not found in configuration.", dto.preset_id);
            Err(StatusCode::BAD_REQUEST)
        }
    } else {
        warn!("Group {} was not found in configuration", dto.group_id);
        Err(StatusCode::BAD_REQUEST)
    }
}

#[cfg(test)]
mod test {
    use crate::automodule::philipshue::routes::{
        PhilipsHueConfigureGroupDto, PhilipsHueGroupDto, PhilipsHuePresetDto,
    };
    use home_automation_common::types::export_type;

    #[test]
    fn export_types() {
        export_type::<PhilipsHueConfigureGroupDto>();
        export_type::<PhilipsHuePresetDto>();
        export_type::<PhilipsHueGroupDto>();
    }
}

use crate::automodule::philipshue::api::ApiClient;
use crate::automodule::philipshue::config::{
    PhilipsHueAutomationModuleConfiguration, CONFIG_FILE_NAME,
};
use crate::automodule::philipshue::routes::{configure_group, get_groups, get_presets, HueState};
use crate::automodule::AutomationModule;
use crate::websocket::dto::AutomationServerStatusUpdate;
use axum::Router;
use home_automation_common::action::AutomationAction;
use home_automation_common::config::ConfigurationManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

mod api;
mod config;
mod dto;
mod routes;

pub struct PhilipsHueAutomationModule {
    configuration: PhilipsHueAutomationModuleConfiguration,
    api_client: ApiClient,
}

impl PhilipsHueAutomationModule {}

impl AutomationModule for PhilipsHueAutomationModule {
    fn new(
        application_folder: &Path,
        _: UnboundedSender<AutomationServerStatusUpdate>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let configuration_manager = Arc::new(ConfigurationManager::<
            PhilipsHueAutomationModuleConfiguration,
        >::load(application_folder, CONFIG_FILE_NAME)?);

        let api_client = ApiClient::new(configuration_manager.get_configuration().clone());

        Ok(PhilipsHueAutomationModule {
            api_client,
            configuration: configuration_manager.get_configuration().clone(),
        })
    }

    fn get_routes(&self) -> Option<Router> {
        let hue_state = HueState {
            request_sender: self.api_client.request_sender(),
            configuration: Arc::new(self.configuration.clone()),
        };
        Some(
            Router::new().nest(
                "/philipshue",
                Router::new()
                    .route("/groups", axum::routing::get(get_groups))
                    .route("/groups", axum::routing::put(configure_group))
                    .route("/presets", axum::routing::get(get_presets))
                    .with_state(hue_state),
            ),
        )
    }

    fn handle_action(&mut self, _: &AutomationAction) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn send_initial_state(&self, _: usize) -> anyhow::Result<()> {
        Ok(())
    }
}

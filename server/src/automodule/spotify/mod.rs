use crate::automodule::spotify::config::{
    SpotifyAutomationModuleCachedConfiguration, SpotifyAutomationModuleConfiguration,
    CACHED_CONFIG_FILE_NAME, CONFIG_FILE_NAME,
};
use crate::automodule::spotify::routes::auth::callback_auth;
use crate::automodule::AutomationModule;
use crate::websocket::dto::AutomationServerStatusUpdate;
use axum::Router;
use home_automation_common::action::AutomationAction;
use home_automation_common::config::ConfigurationManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use self::routes::{get_playlists, SpotifyState};

mod api;
mod config;
mod routes;

pub struct SpotifyAutomationModule {
    state: Arc<Mutex<SpotifyState>>,
}

impl AutomationModule for SpotifyAutomationModule {
    fn new(
        application_folder: &Path,
        _: UnboundedSender<AutomationServerStatusUpdate>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let configuration_manager = Arc::new(ConfigurationManager::<
            SpotifyAutomationModuleConfiguration,
        >::load(application_folder, CONFIG_FILE_NAME)?);

        let cached_configuration_manager = ConfigurationManager::<
            SpotifyAutomationModuleCachedConfiguration,
        >::load(
            application_folder, CACHED_CONFIG_FILE_NAME
        )?;
        let state = Arc::new(Mutex::new(SpotifyState::new(cached_configuration_manager)));

        Ok(SpotifyAutomationModule { state })
    }

    fn get_routes(&self) -> Option<Router> {
        Some(
            Router::new().nest(
                "/spotify",
                Router::new()
                    .route("/auth/callback", axum::routing::get(callback_auth))
                    .route("/playlists", axum::routing::get(get_playlists))
                    .with_state(self.state.clone()),
            ),
        )
    }

    fn handle_action(&mut self, automation_action: &AutomationAction) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn send_initial_state(&self, client_id: usize) -> anyhow::Result<()> {
        Ok(())
    }
}

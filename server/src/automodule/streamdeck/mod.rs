use std::path::Path;

use anyhow::anyhow;
use home_automation_common::action::{AutomationAction, AutomationStatusUpdate};
use home_automation_common::automodule::streamdeck::StreamdeckDevicesConfiguration;
use home_automation_common::config::ConfigurationManager;
use tokio::sync::mpsc::UnboundedSender;

use crate::automodule::AutomationModule;
use crate::websocket::dto::AutomationServerStatusUpdate;

const CONFIG_FILE_NAME: &str = "streamdeckDevicesConfig.json";

pub struct StreamdeckAutomationModule {
    devices_configuration_manager: ConfigurationManager<StreamdeckDevicesConfiguration>,
    status_update_sender: UnboundedSender<AutomationServerStatusUpdate>,
}

impl StreamdeckAutomationModule {
    fn reload_devices_configuration(&mut self) -> anyhow::Result<()> {
        self.devices_configuration_manager.reload_configuration()?;
        let devices_configuration = self.devices_configuration_manager.get_configuration();
        let update = AutomationServerStatusUpdate::broadcast(
            AutomationStatusUpdate::StreamdeckClientReloadedDevicesConfiguration(
                devices_configuration.clone(),
            ),
        );
        self.status_update_sender
            .send(update)
            .map_err(|err| anyhow!("Could not send status update from BCP module: {}", err))?;
        Ok(())
    }
}

impl AutomationModule for StreamdeckAutomationModule {
    fn new(
        application_folder: &Path,
        status_update_sender: UnboundedSender<AutomationServerStatusUpdate>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let devices_configuration_manager =
            ConfigurationManager::<StreamdeckDevicesConfiguration>::load(
                application_folder,
                CONFIG_FILE_NAME,
            )?;
        Ok(StreamdeckAutomationModule {
            status_update_sender,
            devices_configuration_manager,
        })
    }

    fn get_routes(&self) -> Option<axum::Router> {
        None
    }

    fn handle_action(&mut self, automation_action: &AutomationAction) -> anyhow::Result<bool> {
        match automation_action {
            AutomationAction::StreamdeckClientReloadDeviceConfiguration => {
                if let Err(err) = self.reload_devices_configuration() {
                    error!("Could not reload streamdeck devices config: {}", err);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn send_initial_state(&self, client_id: usize) -> anyhow::Result<()> {
        let update = AutomationServerStatusUpdate::single_client(
            AutomationStatusUpdate::StreamdeckClientReloadedDevicesConfiguration(
                self.devices_configuration_manager
                    .get_configuration()
                    .clone(),
            ),
            client_id,
        );
        if let Err(err) = self.status_update_sender.send(update) {
            error!(
                "Could not send streamdeck devices config reloaded message. {}.",
                err
            );
        }
        Ok(())
    }
}

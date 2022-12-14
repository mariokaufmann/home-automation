use std::sync::{Arc, RwLock};

use hidapi::HidApi;
use home_automation_client_lib::websocket::handler::AutomationStatusUpdateHandler;
use home_automation_common::action::AutomationStatusUpdate;
use home_automation_common::automodule::streamdeck::StreamdeckAutomationConfiguration;
use home_automation_common::config::ConfigurationManager;

use crate::config::Configuration;
use crate::StreamdeckClient;

pub struct StreamdeckAutomationClient {
    configuration: Configuration,
    streamdeck_client: StreamdeckClient,
    button_configuration_manager:
        Arc<RwLock<ConfigurationManager<StreamdeckAutomationConfiguration>>>,
}

impl StreamdeckAutomationClient {
    pub fn new(
        hid_api: Arc<std::sync::Mutex<HidApi>>,
        button_configuration_manager: Arc<
            RwLock<ConfigurationManager<StreamdeckAutomationConfiguration>>,
        >,
        configuration: Configuration,
    ) -> anyhow::Result<StreamdeckAutomationClient> {
        let streamdeck_client = StreamdeckClient::connect(hid_api)?;

        Ok(StreamdeckAutomationClient {
            configuration,
            streamdeck_client,
            button_configuration_manager,
        })
    }

    fn fill_streamdeck(&mut self) -> anyhow::Result<()> {
        let configuration_manager = self.button_configuration_manager.read().unwrap();

        let configuration = configuration_manager.get_configuration();

        for button_configuration in &configuration.button_configurations {
            self.streamdeck_client
                .set_button_text(button_configuration.key, &button_configuration.text)?;
        }

        Ok(())
    }
}

impl AutomationStatusUpdateHandler for StreamdeckAutomationClient {
    fn on_status_update(&mut self, status_update: AutomationStatusUpdate) {
        if let AutomationStatusUpdate::StreamdeckClientReloadedDevicesConfiguration(configuration) =
            status_update
        {
            match configuration
                .devices
                .into_iter()
                .find(|client_configuration| {
                    client_configuration
                        .device_id
                        .eq(&self.configuration.device_id)
                }) {
                Some(device_configuration) => {
                    let mut button_configuration_manager_guard =
                        self.button_configuration_manager.write().unwrap();
                    button_configuration_manager_guard
                        .set_configuration(device_configuration.configuration);

                    if let Err(err) = button_configuration_manager_guard.persist_configuration() {
                        error!(
                            "Could not persist new streamdeck button configuration: {}",
                            err
                        );
                    } else {
                        drop(button_configuration_manager_guard);
                        if let Err(err) = self.fill_streamdeck() {
                            error!("Could not fill streamdeck: {}.", err);
                        }
                    }
                }
                None => error!(
                    "Could not find configuration in server configuration with client id: {}",
                    &self.configuration.device_id
                ),
            }
        }
    }
}

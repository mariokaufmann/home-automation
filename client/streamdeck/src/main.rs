#![deny(clippy::all)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use anyhow::{anyhow, Context};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use hidapi::HidApi;
use home_automation_client_lib::websocket::{WebsocketClientInfo, WebsocketRunner};
use home_automation_common::automacro::AutomationMacro;
use home_automation_common::automodule::streamdeck::{
    StreamdeckAutomationConfiguration, StreamdeckButtonConfiguration,
};
use home_automation_common::config::ConfigurationManager;
use home_automation_common::websocket::dto::{AutomationMessage, ClientDeviceType};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

use crate::device::handler::{handle_button_presses, ButtonEvent};
use crate::device::StreamdeckClient;
use crate::handler::StreamdeckAutomationClient;

const CONFIG_FILE_NAME: &str = "automationStreamdeckClientConfig.json";
const BUTTON_CONFIG_FILE_NAME: &str = "buttonAutomationStreamdeckClientConfig.json";
const APPLICATION_NAME: &str = "automation-streamdeck-client";

mod config;
mod device;
mod handler;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let application_folder = home_automation_common::fs::get_application_folder(APPLICATION_NAME)
        .unwrap_or_else(|err| {
            panic!("Could not get application folder: {}", err);
        });

    home_automation_client_lib::logging::init_logger(APPLICATION_NAME);

    // client config
    let configuration_manager =
        ConfigurationManager::<config::Configuration>::load(&application_folder, CONFIG_FILE_NAME)
            .unwrap_or_else(|err| {
                panic!("Could not prepare configuration manager: {}", err);
            });
    let configuration = configuration_manager.get_configuration().clone();

    // button config
    let button_configuration_manager = Arc::new(RwLock::new(
        ConfigurationManager::<StreamdeckAutomationConfiguration>::load(
            &application_folder,
            BUTTON_CONFIG_FILE_NAME,
        )
        .unwrap_or_else(|err| {
            panic!(
                "Could not prepare configuration manager for button config: {}",
                err
            );
        }),
    ));

    let hid_api = Arc::new(std::sync::Mutex::new(HidApi::new().unwrap_or_else(|err| {
        panic!("Could not initialize HID API: {}.", err);
    })));
    print_hid_devices(&hid_api).unwrap_or_else(|err| {
        panic!("Could not print HID list: {}.", err);
    });

    let streamdeck_automation_client = StreamdeckAutomationClient::new(
        hid_api.clone(),
        button_configuration_manager.clone(),
        configuration.clone(),
    )
    .unwrap_or_else(|err| {
        panic!(
            "Could not initialize streamdeck automation client: {}.",
            err
        )
    });

    let message_handler = Arc::new(Mutex::new(streamdeck_automation_client));

    loop {
        info!("Connecting to automation server.");

        let ws_server_url = format!(
            "ws://{}:{}/ws",
            configuration.server_ip, configuration.server_port
        );

        let client_info = get_client_info(&button_configuration_manager);
        let websocket_runner =
            WebsocketRunner::new(client_info, ws_server_url, message_handler.clone());

        let (button_event_tx, button_event_rx) =
            tokio::sync::mpsc::unbounded_channel::<ButtonEvent>();

        handle_button_presses(hid_api.clone(), button_event_tx);
        tokio::spawn(handle_button_events(
            button_event_rx,
            websocket_runner.get_ws_sender(),
            button_configuration_manager.clone(),
        ));

        websocket_runner.stop().await;

        info!("Websocket terminated, reconnecting...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

fn get_client_info(
    button_configuration_manager: &Arc<
        RwLock<ConfigurationManager<StreamdeckAutomationConfiguration>>,
    >,
) -> WebsocketClientInfo {
    let locked_configuration_manager = button_configuration_manager.read().unwrap();
    let button_configuration = locked_configuration_manager.get_configuration();
    WebsocketClientInfo {
        client_name: button_configuration.device_name.clone(),
        client_type: ClientDeviceType::Streamdeck,
    }
}

fn find_button_configuration(
    configuration: &StreamdeckAutomationConfiguration,
    key: u8,
) -> Option<&StreamdeckButtonConfiguration> {
    configuration
        .button_configurations
        .iter()
        .find(|config| config.key == key)
}

async fn handle_button_events(
    mut receiver: UnboundedReceiver<ButtonEvent>,
    sender: UnboundedSender<AutomationMessage>,
    button_configuration_manager: Arc<
        RwLock<ConfigurationManager<StreamdeckAutomationConfiguration>>,
    >,
) {
    while let Some(event) = receiver.recv().await {
        let locked_configuration_manager = button_configuration_manager.read().unwrap();
        let configuration = locked_configuration_manager.get_configuration();
        match event {
            ButtonEvent::ButtonPressed(key) => {
                if let Some(button_config) = find_button_configuration(configuration, key) {
                    if let Err(err) = execute_macro(&sender, button_config.press_macro.clone()) {
                        warn!("Could not execute press macro: {}.", err);
                        break;
                    }
                }
            }
            ButtonEvent::ButtonReleased(key) => {
                if let Some(button_config) = find_button_configuration(configuration, key) {
                    if let Some(release_macro) = &button_config.release_macro {
                        if let Err(err) = execute_macro(&sender, release_macro.clone()) {
                            warn!("Could not execute release macro: {}.", err);
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn execute_macro(
    sender: &UnboundedSender<AutomationMessage>,
    auto_macro: AutomationMacro,
) -> anyhow::Result<()> {
    let message = AutomationMessage::ExecuteMacro { mac: auto_macro };

    sender
        .send(message)
        .context("Could not send websocket message.")?;
    Ok(())
}

fn print_hid_devices(hid_api: &Arc<std::sync::Mutex<HidApi>>) -> anyhow::Result<()> {
    let mut locked_hid = hid_api
        .lock()
        .map_err(|err| anyhow!("Could not lock mutex for HID API: {}", err))?;
    locked_hid
        .refresh_devices()
        .context("Could not refresh HID devices.")?;

    for device in locked_hid.device_list() {
        let product_string = device
            .product_string()
            .context("Could not product string of device.")?;
        let manufacturer_string = device
            .manufacturer_string()
            .context("Could not read manufacturer string of device.")?;
        info!(
            "Found device {} from {} with pid {}, vid {}.",
            product_string,
            manufacturer_string,
            device.product_id(),
            device.vendor_id()
        );
    }
    Ok(())
}

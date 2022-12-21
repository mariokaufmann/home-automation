use crate::automodule::philipshue::config::{
    PhilipsHueAutomationModuleConfiguration, CONFIG_FILE_NAME,
};
use crate::automodule::philipshue::dto::SetStatePayload;
use crate::automodule::AutomationModule;
use crate::websocket::dto::AutomationServerStatusUpdate;
use anyhow::{anyhow, Error};
use axum::{http, Router};
use home_automation_common::action::AutomationAction;
use home_automation_common::config::ConfigurationManager;
use hyper::client::HttpConnector;
use hyper::{Body, StatusCode};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

mod config;
mod dto;
mod routes;

pub struct PhilipsHueAutomationModule {
    request_sender: UnboundedSender<ConfigureHuePayload>,
}

enum HueAssetType {
    Light,
    Group,
}

struct ConfigureHuePayload {
    id: u32,
    asset_type: HueAssetType,
    on: bool,
}

impl PhilipsHueAutomationModule {
    async fn create_requester_task(
        mut request_receiver: UnboundedReceiver<ConfigureHuePayload>,
        configuration_manager: Arc<ConfigurationManager<PhilipsHueAutomationModuleConfiguration>>,
    ) {
        let client = hyper::Client::builder()
            .pool_max_idle_per_host(1)
            .build_http();
        let configuration = configuration_manager.get_configuration();
        while let Some(payload) = request_receiver.recv().await {
            match payload.asset_type {
                HueAssetType::Light => {
                    if let Err(err) =
                        Self::configure_light(payload.id, payload.on, configuration, &client).await
                    {
                        error!("Could not configure light on philips hue bridge: {}", err);
                    }
                }
                HueAssetType::Group => {
                    if let Err(err) =
                        Self::configure_group(payload.id, payload.on, configuration, &client).await
                    {
                        error!("Could not configure group on philips hue bridge: {}", err);
                    }
                }
            }
        }
    }

    async fn configure_light(
        light_id: u32,
        on: bool,
        configuration: &PhilipsHueAutomationModuleConfiguration,
        client: &hyper::Client<HttpConnector>,
    ) -> anyhow::Result<()> {
        let url_string = format!(
            "http://{}/api/{}/lights/{}/state",
            configuration.bridge_ip, configuration.username, light_id
        );

        Self::set_state(&url_string, on, client).await
    }

    async fn configure_group(
        group_id: u32,
        on: bool,
        configuration: &PhilipsHueAutomationModuleConfiguration,
        client: &hyper::Client<HttpConnector>,
    ) -> anyhow::Result<()> {
        let url_string = format!(
            "http://{}/api/{}/groups/{}/action",
            configuration.bridge_ip, configuration.username, group_id
        );

        Self::set_state(&url_string, on, client).await
    }

    async fn set_state(
        url: &str,
        on: bool,
        client: &hyper::Client<HttpConnector>,
    ) -> anyhow::Result<()> {
        let body = SetStatePayload { on };
        let serialized_body = serde_json::to_string(&body)?;

        let request = hyper::http::Request::builder()
            .method("PUT")
            .uri(url)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serialized_body));

        match request {
            Ok(request) => {
                match client.request(request).await {
                    Ok(response) => {
                        match response.status() {
                            StatusCode::OK => Ok(()),
                            other => Err(anyhow!("Received unexpected status code {} when making PUT request to Philips Hue Bridge.", other))
                        }
                    }
                    Err(err) => Err(anyhow!("Could not make PUT request to Philips Hue Bridge: {}", err))
                }
            }
            Err(err) => Err(anyhow!("Could not construct HTTP request to Philips Hue Bridge: {}.", err))
        }
    }

    fn send_request(
        &mut self,
        id: &u32,
        on: &bool,
        asset_type: HueAssetType,
    ) -> Result<bool, Error> {
        match self.request_sender.send(ConfigureHuePayload {
            id: *id,
            on: *on,
            asset_type,
        }) {
            Ok(()) => Ok(true),
            Err(err) => Err(anyhow!(
                "Could not send request to philips hue bridge request sender: {}.",
                err
            )),
        }
    }
}

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

        let (request_tx, request_rx) =
            tokio::sync::mpsc::unbounded_channel::<ConfigureHuePayload>();

        tokio::spawn(Self::create_requester_task(
            request_rx,
            configuration_manager,
        ));

        Ok(PhilipsHueAutomationModule {
            request_sender: request_tx,
        })
    }

    fn get_routes(&self) -> Option<Router> {
        axum::Router::nest            "/philipshue",
        Router::new()
            .route("/light", axum::routing::put()
        )
    }

    fn handle_action(&mut self, automation_action: &AutomationAction) -> anyhow::Result<bool> {
        match automation_action {
            AutomationAction::HueConfigureLight { id, on } => {
                self.send_request(id, on, HueAssetType::Light)
            }
            AutomationAction::HueConfigureGroup { id, on } => {
                self.send_request(id, on, HueAssetType::Group)
            }
            _ => Ok(false),
        }
    }

    fn send_initial_state(&self, _: usize) -> anyhow::Result<()> {
        Ok(())
    }
}

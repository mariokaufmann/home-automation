use crate::automodule::philipshue::config::PhilipsHueAutomationModuleConfiguration;
use anyhow::anyhow;
use axum::http;
use axum::http::StatusCode;
use hyper::client::HttpConnector;
use hyper::Body;
use hyper_tls::HttpsConnector;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct ApiClient {
    request_sender: UnboundedSender<ConfigureHueGroupedLightRequest>,
}

impl ApiClient {
    pub fn new(configuration: PhilipsHueAutomationModuleConfiguration) -> Self {
        let (request_tx, request_rx) =
            tokio::sync::mpsc::unbounded_channel::<ConfigureHueGroupedLightRequest>();

        tokio::spawn(Self::create_requester_task(request_rx, configuration));

        ApiClient {
            request_sender: request_tx,
        }
    }

    async fn create_requester_task(
        mut request_receiver: UnboundedReceiver<ConfigureHueGroupedLightRequest>,
        configuration: PhilipsHueAutomationModuleConfiguration,
    ) {
        let https_connector = HttpsConnector::new();
        let client = hyper::Client::builder()
            .pool_max_idle_per_host(1)
            .build(https_connector);
        while let Some(request) = request_receiver.recv().await {
            if let Err(err) = Self::configure_grouped_light(request, &configuration, &client).await
            {
                error!("Could not configure group on philips hue bridge: {}", err);
            }
        }
    }

    async fn configure_grouped_light(
        request: ConfigureHueGroupedLightRequest,
        configuration: &PhilipsHueAutomationModuleConfiguration,
        client: &hyper::Client<HttpsConnector<HttpConnector>>,
    ) -> anyhow::Result<()> {
        let url = format!(
            "https://{}/clip/v2/resource/grouped_light",
            configuration.bridge_ip
        );

        let body = PutConfiguredLightDto {
            id: request.id,
            on: request.on,
            dimming: PutDimmingDto {
                brightness: request.brightness,
            },
            color_temperature: PutColorTemperatureDto {
                mirek: request.color_temperature,
            },
        };
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

    pub fn request_sender(&self) -> UnboundedSender<ConfigureHueGroupedLightRequest> {
        self.request_sender.clone()
    }
}

pub struct ConfigureHueGroupedLightRequest {
    pub id: String,
    pub on: bool,
    pub brightness: u16,
    pub color_temperature: u16,
}

#[derive(Serialize)]
struct PutConfiguredLightDto {
    id: String,
    on: bool,
    dimming: PutDimmingDto,
    color_temperature: PutColorTemperatureDto,
}

#[derive(Serialize)]
struct PutDimmingDto {
    brightness: u16,
}

#[derive(Serialize)]
struct PutColorTemperatureDto {
    mirek: u16,
}

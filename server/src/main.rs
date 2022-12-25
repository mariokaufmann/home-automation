#![deny(clippy::all)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate core;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use log::LevelFilter;

use crate::automodule::philipshue::PhilipsHueAutomationModule;
use crate::automodule::streamdeck::StreamdeckAutomationModule;
use crate::automodule::{AutomationModule, CompositeAutomationModule};
use crate::services::ServicesContext;
use crate::websocket::dto::AutomationServerStatusUpdate;
use crate::websocket::server::WebsocketServer;

mod automodule;
mod logger;
mod services;
mod websocket;

const APPLICATION_NAME: &str = "home-automation-server";

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let application_folder = home_automation_common::fs::get_application_folder(APPLICATION_NAME)
        .unwrap_or_else(|err| {
            panic!("Could not get application folder: {}", err);
        });

    logger::init_logger(get_logging_level(&args));

    let (status_update_tx, status_update_rx) =
        tokio::sync::mpsc::unbounded_channel::<AutomationServerStatusUpdate>();

    let mut composite_module =
        CompositeAutomationModule::new(&application_folder, status_update_tx.clone()).unwrap();

    // Philips Hue Bridge
    let philips_hue_module =
        PhilipsHueAutomationModule::new(&application_folder, status_update_tx.clone())
            .unwrap_or_else(|err| panic!("Could not load Philips Hue module: {}.", err));
    composite_module.add_module(Box::new(philips_hue_module));

    // Streamdeck module
    let streamdeck_module =
        StreamdeckAutomationModule::new(&application_folder, status_update_tx.clone())
            .unwrap_or_else(|err| panic!("Could not load streamdeck module: {}.", err));
    composite_module.add_module(Box::new(streamdeck_module));

    let api_routes = composite_module.get_routes().unwrap();

    let services_context = Arc::new(ServicesContext {
        modules: Box::new(Mutex::new(composite_module)),
    });

    // websocket server
    let (websocket_event_tx, websocket_event_rx) = tokio::sync::mpsc::unbounded_channel();
    let websocket_server = Arc::new(tokio::sync::Mutex::new(WebsocketServer::new(
        websocket_event_tx,
    )));

    // routes (matched from bottom to top from more specific to less specific)
    let router = Router::new()
        .fallback_service(
            axum::routing::get_service(tower_http::services::ServeDir::new("static")).handle_error(
                |err| async move { error!("error occurred when serving static file: {}.", err) },
            ),
        )
        .merge(api_routes)
        // WS
        .route("/ws", axum::routing::get(websocket::route::ws_handler))
        .layer(axum::extract::Extension(services_context.clone()))
        .layer(axum::extract::Extension(websocket_server.clone()));

    websocket::setup(
        services_context,
        websocket_server,
        status_update_rx,
        websocket_event_rx,
    )
    .await;

    let port = 8091;
    info!("Starting home automation server on port {}.", port);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

fn get_logging_level(args: &[String]) -> LevelFilter {
    match args.get(1) {
        Some(arg) => match arg.as_str() {
            "--debug" => LevelFilter::Debug,
            "--trace" => LevelFilter::Trace,
            _ => LevelFilter::Debug,
        },
        None => LevelFilter::Info,
    }
}

use anyhow::{anyhow, Context};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use hidapi::HidApi;
use rusttype::{Font, Scale};
use streamdeck::{Colour, StreamDeck, TextOptions, TextPosition};

pub mod handler;

const DEFAULT_FONT: &[u8] = include_bytes!("../resources/Roboto-Regular.ttf");

pub struct StreamdeckClient {
    device: StreamDeck,
    font: Font<'static>,
    text_options: TextOptions,
}

impl StreamdeckClient {
    pub fn connect(hidapi: Arc<Mutex<HidApi>>) -> anyhow::Result<StreamdeckClient> {
        let device = connect_to_streamdeck(hidapi)?;

        let font = Font::try_from_bytes(DEFAULT_FONT).context("Could not load default font")?;

        let text_options = TextOptions::new(
            Colour::from_str("FFFFFF").unwrap(),
            Colour::from_str("1B5B88").unwrap(),
            Scale { x: 20.0, y: 20.0 },
            1.1,
        );

        Ok(StreamdeckClient {
            device,
            font,
            text_options,
        })
    }

    pub fn set_button_text(&mut self, index: u8, text: &str) -> anyhow::Result<()> {
        self.device
            .set_button_text(
                index as u8,
                &self.font,
                &TextPosition::Absolute { x: 5, y: 5 },
                text,
                &self.text_options,
            )
            .context("Could not set button text")?;
        Ok(())
    }
}

fn connect_to_streamdeck(hid_api: Arc<Mutex<HidApi>>) -> anyhow::Result<StreamDeck> {
    let locked_hid = hid_api
        .lock()
        .map_err(|err| anyhow!("Could not lock mutex for HID API: {}", err))?;

    match StreamDeck::connect_with_hid(&locked_hid, 0x0fd9, streamdeck::pids::ORIGINAL_V2, None) {
        Ok(streamdeck) => Ok(streamdeck),
        Err(_) => {
            warn!("Could not connect to streamdeck with V2 product ID, trying older pid.");
            let streamdeck =
                StreamDeck::connect_with_hid(&locked_hid, 0x0fd9, streamdeck::pids::ORIGINAL, None)
                    .context("Could not connect to streamdeck with older pid")?;
            Ok(streamdeck)
        }
    }
}

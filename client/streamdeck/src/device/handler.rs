use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use hidapi::HidApi;

use crate::device::connect_to_streamdeck;

const BUTTON_COUNT: usize = 15;

pub enum ButtonEvent {
    ButtonPressed(u8),
    ButtonReleased(u8),
}

pub fn handle_button_presses(
    hid_api: Arc<Mutex<HidApi>>,
    event_sender: tokio::sync::mpsc::UnboundedSender<ButtonEvent>,
) {
    std::thread::spawn(move || {
        if let Err(err) = handle_button_presses_internal(hid_api, event_sender) {
            error!("Could not handle streamdeck button presses: {}.", err);
        }
    });
}

fn handle_button_presses_internal(
    hid_api: Arc<Mutex<HidApi>>,
    event_sender: tokio::sync::mpsc::UnboundedSender<ButtonEvent>,
) -> anyhow::Result<()> {
    let mut streamdeck = connect_to_streamdeck(hid_api)?;
    let mut buttons_state: [bool; BUTTON_COUNT] = [false; BUTTON_COUNT];
    loop {
        let buttons_pressed = streamdeck
            .read_buttons(None)
            .context("Could not read pressed buttons")?;

        if buttons_pressed.len() != BUTTON_COUNT {
            return Err(anyhow!(
                "Button event count was {}, but expected {}.",
                buttons_pressed.len(),
                BUTTON_COUNT
            ));
        }

        for (i, button_state) in buttons_state.iter_mut().enumerate().take(BUTTON_COUNT) {
            let pressed = match buttons_pressed
                .get(i)
                .context("Pressed buttons did not contain expected item")?
            {
                0 => false,
                1 => true,
                val => {
                    return Err(anyhow!(
                        "Pressed button contained unexpected value: {}.",
                        val
                    ))
                }
            };

            if pressed && !(*button_state) {
                // button was not pressed and is now pressed
                let event = ButtonEvent::ButtonPressed(i as u8);
                *button_state = true;
                send_button_event(event, &event_sender)?;
            } else if !pressed && *button_state {
                // button was pressed and is now not pressed
                let event = ButtonEvent::ButtonReleased(i as u8);
                *button_state = false;
                send_button_event(event, &event_sender)?;
            }
        }
    }
}

fn send_button_event(
    event: ButtonEvent,
    event_sender: &tokio::sync::mpsc::UnboundedSender<ButtonEvent>,
) -> anyhow::Result<()> {
    event_sender
        .send(event)
        .map_err(|err| anyhow!("Could not send button event: {}.", err))
}

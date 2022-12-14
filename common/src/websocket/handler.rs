use crate::websocket::convert::parse_message_from_string;
use crate::websocket::dto::AutomationMessage;
use crate::websocket::MessageSender;

pub trait MessageHandler
where
    Self: Send,
{
    fn on_message(
        &mut self,
        client_id: usize,
        message: AutomationMessage,
        sender: &mut dyn MessageSender,
    );
    fn on_client_connected(&mut self, client_id: usize);
    fn on_client_disconnected(&mut self, client_id: usize);
}

pub fn handle_message_with_handler(
    client_id: usize,
    message_handler: &mut dyn MessageHandler,
    message_sender: &mut dyn MessageSender,
    text: &str,
) -> anyhow::Result<()> {
    match parse_message_from_string(text) {
        Ok(message) => {
            message_handler.on_message(client_id, message, message_sender);
        }
        Err(err) => {
            error!(
                "Could not parse text {} to websocket message: {}.",
                text, err
            );
        }
    }
    Ok(())
}

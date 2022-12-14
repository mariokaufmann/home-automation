use crate::websocket::dto::AutomationMessage;

pub mod convert;
pub mod dto;
pub mod handler;

pub trait MessageSender
where
    Self: Send,
{
    fn send_message(&mut self, message: AutomationMessage) -> anyhow::Result<()>;
}

use home_automation_common::websocket::convert::convert_message_to_text;
use home_automation_common::websocket::dto::AutomationMessage;

pub fn convert_message_to_ws_message(
    message: AutomationMessage,
) -> anyhow::Result<axum::extract::ws::Message> {
    let text = convert_message_to_text(message)?;
    Ok(axum::extract::ws::Message::Text(text))
}

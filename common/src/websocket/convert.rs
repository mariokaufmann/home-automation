use crate::websocket::dto::{AutomationMessage, MessageHeader};
use anyhow::anyhow;

const PROTOCOL_VERSION: u32 = 1;
const HEADER_MESSAGE_SEPARATOR: &str = "||";

pub fn parse_message_from_string(text: &str) -> anyhow::Result<AutomationMessage> {
    let mut header: Option<MessageHeader> = None;
    for part in text.split(HEADER_MESSAGE_SEPARATOR) {
        match header {
            None => {
                let parsed_header = serde_json::from_str::<MessageHeader>(part)?;
                header = Some(parsed_header)
            }
            Some(header) => {
                return parse_message_with_header(part, header);
            }
        }
    }
    Err(anyhow!("Message format was invalid, could not parse."))
}

fn parse_message_with_header(
    text: &str,
    header: MessageHeader,
) -> anyhow::Result<AutomationMessage> {
    match header.version {
        PROTOCOL_VERSION => {
            let message = serde_json::from_str::<AutomationMessage>(text)?;
            Ok(message)
        }
        _ => Err(anyhow!(
            "Could not parse message, unknown version number {}.",
            header.version
        )),
    }
}

pub fn convert_message_to_text(message: AutomationMessage) -> anyhow::Result<String> {
    let header = MessageHeader {
        version: PROTOCOL_VERSION,
    };
    let mut text = serialize_generic_message(header)?;
    text.push_str(HEADER_MESSAGE_SEPARATOR);
    let actual_message_text = serialize_generic_message(message)?;
    text.push_str(&actual_message_text);
    Ok(text)
}

fn serialize_generic_message<M>(message: M) -> anyhow::Result<String>
where
    M: serde::Serialize,
{
    let text = serde_json::to_string(&message)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use crate::action::AutomationStatusUpdate;

    use super::*;

    #[test]
    fn test_parse_message_with_payload() {
        let test_message = AutomationMessage::StatusUpdate {
            update: AutomationStatusUpdate::SoundPlayed {
                sound: "test".to_string(),
            },
        };

        let test_data = generate_test_data_with_message(true, &test_message, true);
        let result = do_message_parsing(&test_data);

        assert_eq!(test_message, result);
    }

    #[test]
    fn test_parse_message_without_payload() {
        let test_message = AutomationMessage::Ping;

        let test_data = generate_test_data_with_message(true, &test_message, true);
        let result = do_message_parsing(&test_data);

        assert_eq!(test_message, result);
    }

    #[test]
    #[should_panic]
    fn test_parse_message_no_separator() {
        let test_message = AutomationMessage::Ping;
        let test_data = generate_test_data_with_message(true, &test_message, false);
        do_message_parsing(&test_data);
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_header() {
        let test_message = AutomationMessage::Ping;
        let test_data = generate_test_data_with_message(false, &test_message, true);
        do_message_parsing(&test_data);
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_message() {
        let test_data = generate_test_data(true, "invalid_message", true);
        do_message_parsing(&test_data);
    }

    fn do_message_parsing(text: &str) -> AutomationMessage {
        parse_message_from_string(text).unwrap()
    }

    fn generate_test_data_with_message(
        valid_header: bool,
        test_message: &AutomationMessage,
        valid_separator: bool,
    ) -> String {
        let message_data = serde_json::to_string(test_message).unwrap();
        generate_test_data(valid_header, &message_data, valid_separator)
    }

    fn generate_test_data(valid_header: bool, message_text: &str, valid_separator: bool) -> String {
        let mut test_data = if valid_header {
            let test_header = MessageHeader {
                version: PROTOCOL_VERSION,
            };
            serde_json::to_string(&test_header).unwrap()
        } else {
            "invalid_header".to_owned()
        };

        if valid_separator {
            test_data.push_str(HEADER_MESSAGE_SEPARATOR);
        }

        test_data.push_str(message_text);

        test_data
    }
}

use super::widgets::logs::Log;
use crate::app::Message as AppMessage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Plaintext,
    SystemMessage,
    UserMessage,
}

pub(crate) trait Displayable {
    fn display(&self) -> String;
    fn format(&self) -> Format;
}

impl Displayable for String {
    fn display(&self) -> String {
        self.to_string()
    }

    fn format(&self) -> Format {
        Format::Plaintext
    }
}

impl Displayable for Log {
    fn display(&self) -> String {
        let formatted_log = format!("{:?}\n", self.response);
        formatted_log
    }

    fn format(&self) -> Format {
        Format::Plaintext
    }
}

impl Displayable for AppMessage {
    fn display(&self) -> String {
        match self {
            AppMessage::System(message) | AppMessage::User(message) => message.to_string(),
        }
    }

    fn format(&self) -> Format {
        match self {
            AppMessage::System(_) => Format::SystemMessage,
            AppMessage::User(_) => Format::UserMessage,
        }
    }
}

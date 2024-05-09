use crate::app::Message;

// TODO: add Json
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    // Json,
    Plaintext,
    SystemMessage,
    UserMessage,
}

pub(crate) trait Displayable {
    fn display(&self) -> &str;
    fn format(&self) -> Format;
}

impl Displayable for String {
    fn display(&self) -> &str {
        self
    }

    fn format(&self) -> Format {
        Format::Plaintext
    }
}

impl Displayable for Message {
    fn display(&self) -> &str {
        match self {
            Message::System(message) | Message::User(message) => message,
        }
    }

    fn format(&self) -> Format {
        match self {
            Message::System(_) => Format::SystemMessage,
            Message::User(_) => Format::UserMessage,
        }
    }
}

use crate::app::log::Log;
use crate::app::Message as AppMessage;

// TODO: look into standard Display trait

pub(crate) trait Displayable {
    fn display(&self) -> String;
}

impl Displayable for String {
    fn display(&self) -> String {
        self.to_string()
    }
}

impl Displayable for Log {
    fn display(&self) -> String {
        format!("{:?}: {:?}", self.logged_at, self.response)
    }
}

impl Displayable for AppMessage {
    fn display(&self) -> String {
        match self {
            AppMessage::SystemInternal(message) => message.to_string(),
            AppMessage::SystemPublic(message) => message.to_string(),
            AppMessage::User(message) => format!("{}: {}", message.username, message.content),
        }
    }
}

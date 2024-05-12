use super::widgets::logs::Log;
use crate::app::Message as AppMessage;
use crate::ui::symbols::*;
use ratatui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Plaintext,
    SystemInternalMessage,
    SystemPublicMessage,
    UserMessage,
}

pub(crate) trait Displayable {
    fn display(&self) -> String;
    fn format(&self) -> Format;

    fn to_line(&self) -> Line {
        Line::from(Span::raw(self.display()))
    }
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
            AppMessage::SystemInternal(message) => message.to_string(),
            AppMessage::SystemPublic(message) => message.to_string(),
            AppMessage::User(message) => format!("{}: {}", message.username, message.content),
        }
    }

    fn format(&self) -> Format {
        match self {
            AppMessage::SystemInternal(_) => Format::SystemInternalMessage,
            AppMessage::SystemPublic(_) => Format::SystemPublicMessage,
            AppMessage::User(_) => Format::UserMessage,
        }
    }

    fn to_line(&self) -> Line {
        match self {
            AppMessage::SystemInternal(message) => Line::from(Span::styled(
                format!("{} {INTERNAL_MESSAGE_SYMBOL}", message.display()),
                Style::default().italic().dim(),
            )),

            AppMessage::SystemPublic(message) => Line::from(Span::styled(
                format!("{}", message.display()),
                Style::default().light_blue().italic(),
            )),
            AppMessage::User(message) => {
                let username = message.username.clone();
                let content = message.content.clone();

                Line::from(vec![
                    Span::styled(format!("{}: ", username), Style::default().light_green()),
                    Span::raw(content),
                ])
            }
        }
    }
}

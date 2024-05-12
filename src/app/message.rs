use chrono::{DateTime, Utc};
use std::fmt;

// TODO: implement SystemError
#[derive(Clone, Debug)]
pub enum Message {
    SystemInternal(String),
    SystemPublic(String),
    // SystemError(String),
    User(UserMessage),
}

#[derive(Clone, Debug)]
pub struct UserMessage {
    pub username: String,
    pub content: String,
    pub sent_at: DateTime<Utc>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemInternal(message) => write!(f, "{}", message),
            Self::SystemPublic(message) => write!(f, "{}", message),
            Self::User(message) => write!(f, "{}: {}", message.username, message.content),
        }
    }
}

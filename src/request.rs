use serde_json::{json, Value as SerdeValue};

use crate::client::Refs;
use crate::message::Message;
use crate::user::User;

// This module contains the Request struct used to create requests to be sent to the server.

#[derive(Debug)]
enum Event {
    Join,
    Leave,
    Shout(String),
}

#[derive(Debug)]
pub struct Request {
    pub user: User,
    pub room: String,
    event: Event,
}

impl Request {
    pub fn join(room: String, user: User) -> Self {
        Self {
            event: Event::Join,
            room,
            user,
        }
    }

    pub fn leave(room: String, user: User) -> Self {
        Self {
            event: Event::Leave,
            room,
            user,
        }
    }

    pub fn shout(room: String, message: String, user: User) -> Self {
        Self {
            event: Event::Shout(message),
            room,
            user,
        }
    }

    pub fn to_payload(&self, refs: Refs) -> String {
        let event = self.event();
        let payload = self.payload();

        let message = Message {
            join_ref: Some(refs.get_join_ref()),
            message_ref: Some(refs.get_message_ref()),
            topic: format!("relay:{}", self.room),
            event,
            payload,
        };

        message
            .serialize_request()
            .expect("Problem serializing message")
    }

    fn event(&self) -> String {
        match self.event {
            Event::Shout(_) => "shout".to_string(),
            Event::Join => "phx_join".to_string(),
            Event::Leave => "phx_leave".to_string(),
        }
    }

    fn payload(&self) -> SerdeValue {
        match &self.event {
            Event::Shout(message) => json!({ "user": self.user, "message": message }),
            Event::Join => json!({ "user": self.user  }),
            Event::Leave => json!({}),
        }
    }
}

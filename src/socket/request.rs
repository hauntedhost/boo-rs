use serde_json::{json, Value as SerdeValue};

use crate::app::user::User;
use crate::socket::client::Refs;
use crate::socket::message::Message;

// This module contains the Request struct used to create requests to be sent to the server.

const TOPIC_PREFIX: &str = "relay:";

#[derive(Debug)]
enum Event {
    Heartbeat,
    // TODO: rewrite as Join(User) and remove user requirement from Request struct
    Join,
    Leave,
    Shout(String),
}

#[derive(Debug)]
pub struct Request {
    user: User,
    topic: String,
    event: Event,
}

impl Request {
    pub fn heartbeat(user: User) -> Self {
        Self {
            event: Event::Heartbeat,
            topic: "phoenix".to_string(),
            user,
        }
    }

    pub fn join(room: String, user: User) -> Self {
        Self {
            event: Event::Join,
            topic: room_to_topic(room),
            user,
        }
    }

    pub fn leave(room: String, user: User) -> Self {
        Self {
            event: Event::Leave,
            topic: room_to_topic(room),
            user,
        }
    }

    pub fn shout(room: String, message: String, user: User) -> Self {
        Self {
            event: Event::Shout(message),
            topic: room_to_topic(room),
            user,
        }
    }

    pub fn to_payload(&self, refs: Refs) -> String {
        let event = self.event();
        let payload = self.payload();

        let message = Message {
            join_ref: Some(refs.get_join_ref()),
            message_ref: Some(refs.get_message_ref()),
            topic: self.topic.clone(),
            event,
            payload,
        };

        message
            .serialize_request()
            .expect("Problem serializing message")
    }

    fn event(&self) -> String {
        match self.event {
            Event::Heartbeat => "heartbeat".to_string(),
            Event::Join => "phx_join".to_string(),
            Event::Leave => "phx_leave".to_string(),
            Event::Shout(_) => "shout".to_string(),
        }
    }

    fn payload(&self) -> SerdeValue {
        match &self.event {
            Event::Heartbeat => json!({}),
            Event::Join => json!({ "user": self.user  }),
            Event::Leave => json!({}),
            Event::Shout(message) => json!({ "user": self.user, "message": message }),
        }
    }
}

fn room_to_topic(room: String) -> String {
    format!("{TOPIC_PREFIX}{room}")
}

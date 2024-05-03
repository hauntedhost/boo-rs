use serde_json::{json, Value as SerdeValue};

use crate::app::user::User;
use crate::socket::client::Refs;
use crate::socket::message::Message;

// This module contains the Request struct used to create requests to be sent to the server.

const TOPIC_PREFIX: &str = "relay:";

#[derive(Debug)]
pub struct Request {
    topic: String,
    event: String,
    payload: SerdeValue,
}

impl Request {
    pub fn new_heartbeat() -> Self {
        Self {
            topic: "phoenix".to_string(),
            event: "heartbeat".to_string(),
            payload: json!({}),
        }
    }

    pub fn new_join(room: String, user: User) -> Self {
        Self {
            topic: room_to_topic(room),
            event: "phx_join".to_string(),
            payload: json!({ "user": user  }),
        }
    }

    pub fn new_leave(room: String) -> Self {
        Self {
            topic: room_to_topic(room),
            event: "phx_leave".to_string(),
            payload: json!({}),
        }
    }

    pub fn new_shout(room: String, message: String) -> Self {
        Self {
            topic: room_to_topic(room),
            event: "shout".to_string(),
            payload: json!({  "message": message }),
        }
    }

    pub fn to_payload(&self, refs: Refs) -> String {
        let message = Message {
            join_ref: Some(refs.get_join_ref()),
            message_ref: Some(refs.get_message_ref()),
            topic: self.topic.clone(),
            event: self.event.clone(),
            payload: self.payload.clone(),
        };

        message
            .serialize_request()
            .expect("Problem serializing message")
    }
}

fn room_to_topic(room: String) -> String {
    format!("{TOPIC_PREFIX}{room}")
}

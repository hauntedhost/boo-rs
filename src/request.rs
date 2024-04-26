use serde::Serialize;
use serde_json::json;

use crate::message::{serialize_message_array, MessageArray};
use crate::user::User;

const DEFAULT_TOPIC: &str = "relay:lobby";

#[derive(Debug)]
pub enum Request {
    Shout(Shout),
    Join(Join),
}

#[derive(Default, Serialize, Debug)]
pub struct Shout {
    pub user: User,
    pub message: String,
    // pub message_ref: Option<u32>,
}

#[derive(Default, Serialize, Debug)]
pub struct Join {
    pub user: User,
    // pub join_ref: Option<u32>,
    // pub message_ref: Option<u32>,
}

pub struct Refs {
    pub join_ref: u32,
    pub message_ref: u32,
}

pub fn build_request(request: Request, refs: Refs) -> String {
    match request {
        Request::Shout(shout) => {
            let Shout { user, message } = shout;

            let event = "shout";
            let payload = json!({ "user": user, "message": message });

            let array: MessageArray = (
                Some(refs.join_ref),
                Some(refs.message_ref),
                DEFAULT_TOPIC.to_string(),
                event.to_string(),
                payload,
            );

            serialize_message_array(&array).expect("Problem serializing message")
        }
        Request::Join(join) => {
            let Join { user } = join;

            let event = "phx_join";
            let payload = json!({ "user": user  });

            let array: MessageArray = (
                Some(refs.join_ref),
                Some(refs.message_ref),
                DEFAULT_TOPIC.to_string(),
                event.to_string(),
                payload,
            );

            serialize_message_array(&array).expect("Problem serializing message")
        }
    }
}

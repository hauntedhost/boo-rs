use serde::Serialize;
use serde_json::{json, Value as SerdeValue};

use crate::message::Message;
use crate::user::User;

const DEFAULT_ROOM: &str = "lobby";

#[derive(Debug)]
pub enum Request {
    Shout(Shout),
    Join(Join),
}

impl Request {
    fn event(&self) -> String {
        match self {
            Request::Shout(_) => "shout".to_string(),
            Request::Join(_) => "phx_join".to_string(),
        }
    }

    fn payload(&self) -> SerdeValue {
        match self {
            Request::Shout(shout) => json!({ "user": shout.user, "message": shout.message }),
            Request::Join(join) => json!({ "user": join.user  }),
        }
    }
}

#[derive(Default, Serialize, Debug)]
pub struct Shout {
    pub user: User,
    pub message: String,
}

#[derive(Default, Serialize, Debug)]
pub struct Join {
    pub user: User,
}

pub struct Refs {
    pub join_ref: u32,
    pub message_ref: u32,
}

pub fn build_request(request: Request, refs: Refs) -> String {
    let event = request.event();
    let payload = request.payload();

    let message = Message {
        join_ref: Some(refs.join_ref),
        message_ref: Some(refs.message_ref),
        topic: format!("relay:{DEFAULT_ROOM}"),
        event,
        payload,
    };

    message
        .serialize_request()
        .expect("Problem serializing message")
}

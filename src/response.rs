/// This module contains logic for parsing JSON from the server.
/// It exposes a single `parse_response` fn which takes a JSON string and returns a `Response` enum.
use serde::{Deserialize, Serialize};
use serde_json::{Result as SerdeResult, Value};
use std::collections::HashMap;

use crate::user::User;

// The response enum we will build based on the event type
#[derive(Default, Debug)]
pub enum Response {
    #[default]
    Null,
    Shout(Shout),
    PresenceDiff(PresenceDiff),
    PresenceState(PresenceState),
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Shout {
    pub user: User,
    pub message: String,
}

#[derive(Default, Debug)]
pub struct PresenceDiff {
    pub joins: Vec<User>,
    pub leaves: Vec<User>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct PresenceState {
    pub users: Vec<User>,
}

// The server sends messages as an array:
// [join_ref, message_ref, topic, event, payload]
type MessageArray = (Option<u32>, Option<u32>, String, String, Value);

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawPresenceDiff {
    joins: HashMap<String, UserPresence>,
    leaves: HashMap<String, UserPresence>,
}

type RawPresenceState = HashMap<String, UserPresence>;

#[derive(Serialize, Deserialize, Debug)]
struct UserPresence {
    metas: Vec<UserMeta>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserMeta {
    username: String,
}

pub fn parse_response(json_data: &str) -> Response {
    let Ok((_join_ref, _message_ref, _topic, event, payload)) = parse_message_array(json_data)
    else {
        return Response::Null;
    };

    match event.as_str() {
        "shout" => {
            let shout = serde_json::from_value::<Shout>(payload).unwrap();
            return Response::Shout(shout);
        }
        "presence_diff" => {
            let raw_diff = serde_json::from_value::<RawPresenceDiff>(payload).unwrap();
            let joins = extract_first_users(raw_diff.joins);
            let leaves = extract_first_users(raw_diff.leaves);
            return Response::PresenceDiff(PresenceDiff { joins, leaves });
        }
        "presence_state" => {
            let raw_state = serde_json::from_value::<RawPresenceState>(payload).unwrap();
            let users = extract_first_users(raw_state);
            return Response::PresenceState(PresenceState { users });
        }
        _ => {
            return Response::Null;
        }
    }
}

fn extract_first_users(joins: HashMap<String, UserPresence>) -> Vec<User> {
    let mut users = Vec::new();
    for (user_id, user_presence) in joins {
        if let Some(first_user) = user_presence.metas.get(0) {
            users.push(User {
                uuid: user_id,
                username: first_user.username.clone(),
            });
        }
    }

    users
}

fn parse_message_array(json_data: &str) -> SerdeResult<MessageArray> {
    let message_array: MessageArray = serde_json::from_str(json_data)?;
    Ok(message_array)
}

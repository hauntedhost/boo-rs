use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::message::Message;
use crate::room::Room;
use crate::user::User;

/// This module contains logic for parsing JSON from the server.
/// It exposes a single `parse_response` fn which takes a JSON string and returns a `Response` enum.

// The response enum we will build based on the event type
#[derive(Default, Debug)]
pub enum Response {
    #[default]
    Null,
    JoinReply(JoinReply),
    RoomsUpdate(RoomsUpdate),
    Shout(Shout),
    PresenceDiff(PresenceDiff),
    PresenceState(PresenceState),
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct JoinReply {
    pub user: User,
}

pub type RoomsUpdate = Vec<Room>;

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

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawReply {
    status: String,
    response: RawReplyResponse,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawReplyResponse {
    event: String,
    user: User,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawRoomsUpdate {
    rooms: Vec<RoomUpdateArray>,
}

type RoomUpdateArray = (
    String, // name
    u32,    // user count
);

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawPresenceDiff {
    joins: HashMap<String, UserPresence>,
    leaves: HashMap<String, UserPresence>,
}

type RawPresenceState = HashMap<String, UserPresence>;

#[derive(Serialize, Deserialize, Debug)]
struct UserPresence {
    metas: Vec<User>,
}

pub fn parse_response(json_data: &str) -> Response {
    let Ok(message) = Message::parse_response(json_data) else {
        return Response::Null;
    };

    match message.event.as_str() {
        "phx_reply" => {
            // currently only handling phx_join response.event
            if let Ok(reply) = serde_json::from_value::<RawReply>(message.payload) {
                if reply.response.event == "phx_join" {
                    return Response::JoinReply(JoinReply {
                        user: reply.response.user,
                    });
                }
            }
            return Response::Null;
        }
        "shout" => {
            let shout = serde_json::from_value::<Shout>(message.payload).unwrap();
            return Response::Shout(shout);
        }
        "rooms_update" => {
            let rooms_update = serde_json::from_value::<RawRoomsUpdate>(message.payload).unwrap();
            let rooms: Vec<Room> = rooms_update
                .rooms
                .iter()
                .map(|room_update| Room {
                    name: room_update.0.clone(),
                    user_count: room_update.1,
                })
                .collect();
            return Response::RoomsUpdate(rooms);
        }
        "presence_diff" => {
            let raw_diff = serde_json::from_value::<RawPresenceDiff>(message.payload).unwrap();
            let joins = extract_first_users(raw_diff.joins);
            let leaves = extract_first_users(raw_diff.leaves);
            return Response::PresenceDiff(PresenceDiff { joins, leaves });
        }
        "presence_state" => {
            let raw_state = serde_json::from_value::<RawPresenceState>(message.payload).unwrap();
            let users = extract_first_users(raw_state);
            return Response::PresenceState(PresenceState { users });
        }
        _ => {
            return Response::Null;
        }
    }
}

// A user can be "present" from multiple devices, we only care about the first one right now
fn extract_first_users(joins: HashMap<String, UserPresence>) -> Vec<User> {
    let mut users = Vec::new();
    for (_key, user_presence) in joins {
        if let Some(first_user) = user_presence.metas.get(0) {
            users.push(first_user.clone());
        }
    }
    users
}

use crate::app::room::Room;
use crate::app::user::User;
use crate::socket::message::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// This module contains logic for parsing JSON from the server.
/// Response struct exposes a single `new_from_json_string` fn which takes a JSON string and returns a `Response` enum.

// TODO: handle PhxReply messages:
// [null,3,"phoenix","phx_reply",{"status":"ok","response":{}}]

// The Response enum we will build based on the event type
#[derive(Clone, Default, Debug)]
pub enum Response {
    #[default]
    Unknown,
    JoinReply(JoinReply),
    RoomsUpdate(RoomsUpdate),
    Shout(Shout),
    PresenceDiff(PresenceDiff),
    PresenceState(PresenceState),
}

impl Response {
    pub fn new_from_json_string(json_data: &str) -> Self {
        let Ok(message) = Message::new_from_json_string(json_data) else {
            return Response::Unknown;
        };

        return match message.event.as_str() {
            "phx_reply" => {
                // currently only handling phx_join response.event
                if let Ok(reply) = serde_json::from_value::<RawReply>(message.payload) {
                    if reply.response.event == "phx_join" {
                        return Response::JoinReply(JoinReply {
                            user: reply.response.user,
                        });
                    }
                }
                Response::Unknown
            }
            "presence_diff" => {
                let raw_diff = serde_json::from_value::<RawPresenceDiff>(message.payload).unwrap();
                let joins = extract_first_users(raw_diff.joins);
                let leaves = extract_first_users(raw_diff.leaves);
                Response::PresenceDiff(PresenceDiff { joins, leaves })
            }
            "presence_state" => {
                let raw_state =
                    serde_json::from_value::<RawPresenceState>(message.payload).unwrap();
                let users = extract_first_users(raw_state);
                Response::PresenceState(PresenceState { users })
            }
            "rooms_update" => {
                let rooms_update =
                    serde_json::from_value::<RawRoomsUpdate>(message.payload).unwrap();
                let rooms: Vec<Room> = rooms_update
                    .rooms
                    .iter()
                    .map(|room_update| Room {
                        name: room_update.0.clone(),
                        user_count: room_update.1,
                    })
                    .collect();
                Response::RoomsUpdate(rooms)
            }
            "shout" => {
                let shout = serde_json::from_value::<Shout>(message.payload).unwrap();
                Response::Shout(shout)
            }
            _ => Response::Unknown,
        };
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct JoinReply {
    pub user: User,
}

#[derive(Clone, Default, Debug)]
pub struct PresenceDiff {
    pub joins: Vec<User>,
    pub leaves: Vec<User>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct PresenceState {
    pub users: Vec<User>,
}

pub type RoomsUpdate = Vec<Room>;

// Private

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Shout {
    pub user: User,
    pub message: String,
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

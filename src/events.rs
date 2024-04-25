use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

use crate::client::{self, Call};
use crate::user::User;

// The server sends messages as an array:
// [join_ref, message_ref, topic, event, payload]
type ServerMessage = (
    Option<i32>, // join_ref
    Option<i32>, // ref
    String,      // topic
    String,      // event
    Payload,     // payload
);

// Parsed message via custom serializer
#[allow(dead_code)]
#[derive(Default, Debug)]
struct Message {
    join_ref: Option<i32>,
    message_ref: Option<i32>,
    topic: String,
    event: String,
    payload: Payload,
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let server_message = ServerMessage::deserialize(deserializer)?;
        Ok(Message::from(server_message))
    }
}

impl From<ServerMessage> for Message {
    fn from(server_message: ServerMessage) -> Self {
        Message {
            join_ref: server_message.0,
            message_ref: server_message.1,
            topic: server_message.2,
            event: server_message.3,
            payload: server_message.4,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Payload {
    event: Option<String>,
    username: Option<String>,
    message: Option<String>,
    joins: Option<HashMap<String, UserPresence>>,
    leaves: Option<HashMap<String, UserPresence>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserPresence {
    metas: Vec<UserMeta>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserMeta {
    username: String,
}

// The response we will build and use internally
#[derive(Default, Debug)]
pub struct Response {
    pub event: String,
    pub username: String,
    pub message: String,
    pub joins: Vec<User>,
    pub leaves: Vec<User>,
}

fn parse_response(json_data: &str) -> Option<Response> {
    let message: Message = serde_json::from_str(json_data).unwrap();

    let response = Response {
        event: message.event,
        username: message.payload.username.unwrap_or_default(),
        message: message.payload.message.unwrap_or_default(),
        joins: extract_first_users(message.payload.joins),
        leaves: extract_first_users(message.payload.leaves),
    };

    Some(response)
}

fn extract_first_users(joins: Option<HashMap<String, UserPresence>>) -> Vec<User> {
    let Some(joins) = joins else {
        return vec![];
    };

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

pub fn handle_events(
    user: &mut User,
    input: &mut String,
    messages: &mut Vec<String>,
    logs: &mut Vec<String>,
    rx: &mut Receiver<String>,
    handle: &ezsockets::Client<client::Client>,
) -> std::io::Result<bool> {
    let username = &user.username;

    match rx.try_recv() {
        Ok(message_payload) => {
            logs.push(message_payload.clone());

            if let Some(response) = parse_response(&message_payload) {
                let Response {
                    event,
                    username,
                    message,
                    joins,
                    leaves,
                } = response;

                if event.eq("presence_diff") {
                    for user in joins {
                        let message = format!("{} has joined the chat!", user.username);
                        messages.push(message);
                    }

                    for user in leaves {
                        let message = format!("{} has left the chat!", user.username);
                        messages.push(message);
                    }
                } else if event.eq("shout") && !username.eq(&user.username) {
                    let message = format!("{username}: {message}");
                    messages.push(message);
                }
            };
        }
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
            // No messages, do nothing
        }
        Err(_e) => {
            // Error, do nothing for now
        }
    }

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Esc
                || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
            {
                return Ok(true);
            }

            if key.code == KeyCode::Enter {
                // TODO: fix username change logic
                //   1. push this message uniquely, e.g. "user x has changed their name to y"
                //   2. the server needs to handle the change too
                //   3. the rx.try_recv() also has to handle the name change broadcast
                // if input.starts_with("/username") {
                //     let prefix = "/username";
                //     let new_username = &input.trim()[prefix.len()..];
                //     if !new_username.is_empty() {
                //         *username = new_username.trim().to_string();
                //     } else {
                //         return Ok(false);
                //     }
                // }

                if input.len() > 0 {
                    let message = format!("{username}: {input}");
                    messages.push(message.clone());
                    handle
                        .call(Call::Shout(user.clone(), input.clone()))
                        .expect("call shout error");
                    input.clear();
                }
            } else if key.code == KeyCode::Backspace {
                input.pop();
            } else if let KeyCode::Char(c) = key.code {
                input.push(c);
            }
        }
    }

    Ok(false)
}

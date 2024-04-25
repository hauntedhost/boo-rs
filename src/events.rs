use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use serde_json::{Result as SerdeResult, Value};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

use crate::client::{self, Call};
use crate::user::User;

// The server sends messages as an array:
// [join_ref, message_ref, topic, event, payload]
type MessageArray = (Option<u32>, Option<u32>, String, String, Value);

// The response enum we will build based on the event type
#[derive(Default, Debug)]
enum Response {
    #[default]
    Null,
    Shout(Shout),
    PresenceDiff(PresenceDiff),
    PresenceState(PresenceState),
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct Shout {
    user: User,
    message: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawPresenceDiff {
    joins: HashMap<String, UserPresence>,
    leaves: HashMap<String, UserPresence>,
}

#[derive(Default, Debug)]
struct PresenceDiff {
    joins: Vec<User>,
    leaves: Vec<User>,
}

type RawPresenceState = HashMap<String, UserPresence>;

#[derive(Default, Serialize, Deserialize, Debug)]
struct PresenceState {
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserPresence {
    metas: Vec<UserMeta>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserMeta {
    username: String,
}

fn parse_message_array(json_data: &str) -> SerdeResult<MessageArray> {
    let message_array: MessageArray = serde_json::from_str(json_data)?;
    Ok(message_array)
}

fn parse_response(json_data: &str) -> Response {
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

pub fn handle_events(
    user: &mut User,
    users: &mut Vec<User>,
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

            match parse_response(&message_payload) {
                Response::Null => (),
                Response::Shout(shout) => {
                    log::info!("Shout={:?}", shout);

                    if !shout.user.uuid.eq(&user.uuid) {
                        let message = format!("{}: {}", shout.user.username, shout.message);
                        messages.push(message);
                    }
                }
                Response::PresenceDiff(diff) => {
                    log::info!("PresenceDiff={:?}", diff);

                    for user in diff.joins {
                        let message = format!("{} has joined the chat!", user.username);
                        messages.push(message);
                    }

                    for user in diff.leaves {
                        let message = format!("{} has left the chat!", user.username);
                        messages.push(message);
                    }
                }
                Response::PresenceState(state) => {
                    log::info!("PresenceState={:?}", state);
                    *users = state.users;
                }
            }
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

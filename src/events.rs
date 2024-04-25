use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io::Result;
use tokio::sync::mpsc::Receiver;

use crate::{
    client::{self, Call},
    user::User,
};

#[derive(Default, Debug)]
pub struct Response {
    pub event: String,
    pub username: String,
    pub message: String,
    pub joins: Vec<User>,
    pub leaves: Vec<User>,
}

pub fn handle_events(
    user: &mut User,
    input: &mut String,
    messages: &mut Vec<String>,
    logs: &mut Vec<String>,
    rx: &mut Receiver<String>,
    handle: &ezsockets::Client<client::Client>,
) -> Result<bool> {
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
        Err(e) => {
            println!("error={:?}", e);
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

// response is in the form of:
// [join_ref, message_ref, topic, event, payload]
fn parse_response(json: &String) -> Option<Response> {
    let Ok(value) = serde_json::from_str(json) else {
        return None;
    };

    if let serde_json::Value::Array(elements) = value {
        let Some(event) = elements.get(3)?.as_str() else {
            return None;
        };

        let Some(payload) = elements.get(4) else {
            return None;
        };

        let username = match payload["username"].as_str() {
            Some(username) => username,
            None => "",
        };

        let message = match payload["message"].as_str() {
            Some(message) => message,
            None => "",
        };

        // TODO: streamline joins/leaves parsing
        let mut joins: Vec<User> = Vec::new();
        if let Some(serde_json::Value::Object(joins_map)) = payload.get("joins") {
            for (user_id, metadata) in joins_map {
                if let Some(serde_json::Value::Array(metas)) = metadata.get("metas") {
                    if let Some(serde_json::Value::Object(meta)) = metas.get(0) {
                        let user = User {
                            uuid: user_id.clone(),
                            username: meta["username"].as_str().unwrap().to_string(),
                        };
                        joins.push(user);
                    }
                }
            }
        }

        let mut leaves: Vec<User> = Vec::new();
        if let Some(serde_json::Value::Object(leaves_map)) = payload.get("leaves") {
            for (user_id, metadata) in leaves_map {
                if let Some(serde_json::Value::Array(metas)) = metadata.get("metas") {
                    if let Some(serde_json::Value::Object(meta)) = metas.get(0) {
                        let user = User {
                            uuid: user_id.clone(),
                            username: meta["username"].as_str().unwrap().to_string(),
                        };
                        leaves.push(user);
                    }
                }
            }
        }

        return Some(Response {
            event: event.to_string(),
            username: username.to_string(),
            message: message.to_string(),
            joins,
            leaves,
        });
    }

    None
}

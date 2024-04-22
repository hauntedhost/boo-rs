use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io::Result;
use tokio::sync::mpsc::Receiver;

use crate::client::{self, Call};

#[derive(Default, Debug)]
pub struct Response {
    pub event: String,
    pub user: String,
    pub message: String,
}

pub fn handle_events(
    username: &mut String,
    input: &mut String,
    messages: &mut Vec<String>,
    logs: &mut Vec<String>,
    rx: &mut Receiver<String>,
    handle: &ezsockets::Client<client::Client>,
) -> Result<bool> {
    match rx.try_recv() {
        Ok(message_payload) => {
            logs.push(message_payload.clone());

            if let Some(response) = parse_response(&message_payload) {
                let Response {
                    event,
                    user,
                    message,
                } = response;

                if event.eq("joined") {
                    let message = format!("{user} has joined the chat!");
                    messages.push(message);
                } else if event.eq("shout") && !user.eq(username) {
                    let message = format!("{user}: {message}");
                    messages.push(message);
                }
            };
        }
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
            // No messages, do nothing
        }
        Err(_e) => {
            // Error, whatever
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
                        .call(Call::Shout(input.clone(), username.clone()))
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

fn parse_response(json: &String) -> Option<Response> {
    let Ok(value) = serde_json::from_str(json) else {
        return None;
    };

    if let serde_json::Value::Array(elements) = value {
        let Some(payload) = elements.get(4) else {
            return None;
        };

        return extract_response(payload);
    }
    None
}

fn extract_response(payload: &serde_json::Value) -> Option<Response> {
    let Some(event) = payload["event"].as_str() else {
        return None;
    };

    // TODO: let user be optional, e.g. for system messages
    let Some(user) = payload["user"].as_str() else {
        return None;
    };

    // TODO: let message be optional, e.g. for system broadcasts
    let Some(message) = payload["message"].as_str() else {
        return None;
    };

    Some(Response {
        event: event.to_string(),
        user: user.to_string(),
        message: message.to_string(),
    })
}

/// This module contains code for handling events within the main app loop.
/// It exposes a single `handle_events` function which handles both:
///   - incoming messages from the server
///   - keyboard input from the user
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc::Receiver;

use crate::client::{self, Call, Shout as CallShout};
use crate::response::{parse_response, Response};
use crate::user::User;

pub fn handle_events(
    handle: &ezsockets::Client<client::Client>,
    rx: &mut Receiver<String>,
    user: &mut User,
    users: &mut Vec<User>,
    input: &mut String,
    messages: &mut Vec<String>,
    logs: &mut Vec<String>,
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
                        .call(Call::Shout(CallShout {
                            user: user.clone(),
                            message: input.clone(),
                        }))
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

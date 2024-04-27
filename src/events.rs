/// This module contains code for handling events within the main app loop.
/// It exposes a single `handle_events` function which handles both:
///   - incoming messages from the server
///   - keyboard input from the user
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc::Receiver;

use crate::app::AppState;
use crate::client;
use crate::request::{Join, Request, Shout};
use crate::response::{parse_response, Response};

pub fn handle_events(
    handle: &ezsockets::Client<client::Client>,
    rx: &mut Receiver<String>,
    app: &mut AppState,
) -> std::io::Result<bool> {
    match rx.try_recv() {
        Ok(message_payload) => {
            app.append_log(message_payload.clone());

            match parse_response(&message_payload) {
                Response::Null => (),
                Response::Shout(shout) => {
                    log::info!("Shout={:?}", shout);

                    if !shout.user.uuid.eq(&app.user.uuid) {
                        let message = format!("{}: {}", shout.user.username, shout.message);
                        app.messages.push(message);
                    }
                }
                Response::PresenceDiff(diff) => {
                    log::info!("PresenceDiff={:?}", diff);

                    for user in diff.joins {
                        let message = format!("{} has joined the chat!", user.username);
                        app.messages.push(message);
                    }

                    for user in diff.leaves {
                        let message = format!("{} has left the chat!", user.username);
                        app.messages.push(message);
                    }
                }
                Response::PresenceState(state) => {
                    log::info!("PresenceState={:?}", state);
                    app.users = state.users;
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

            if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('s') {
                app.toggle_sidebar();
                return Ok(false);
            }

            if key.code == KeyCode::Enter {
                // Ignore empty messages
                if app.input.len() == 0 {
                    return Ok(false);
                }

                // Special case for handling the first message as a join request
                if !app.has_joined() {
                    app.user.username = app.input.clone();

                    let request = Request::Join(Join {
                        user: app.user.clone(),
                    });
                    handle.call(request).expect("join error");

                    app.input.clear();
                    app.set_has_joined();

                    return Ok(false);
                }

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

                // Handle normal messages
                let message = format!("{}: {}", &app.user.username, app.input);
                app.messages.push(message.clone());

                let request = Request::Shout(Shout {
                    user: app.user.clone(),
                    message: app.input.clone(),
                });
                handle.call(request).expect("call shout error");

                app.input.clear();
            } else if key.code == KeyCode::Backspace {
                // if user has not joined yet, and input is the guest username, clear the input on backspace
                if !app.has_joined() && app.input.clone() == app.user.username.clone() {
                    app.input.clear();
                } else {
                    app.input.pop();
                }
            } else if let KeyCode::Char(c) = key.code {
                app.input.push(c);
            }
        }
    }

    Ok(false)
}

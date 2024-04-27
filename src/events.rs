/// This module contains code for handling events within the main app loop.
/// It exposes a single `handle_events` function which handles both:
///   - incoming messages from the server
///   - keyboard input from the user
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use log::info;
use tokio::sync::mpsc::Receiver;

use crate::app::{AppState, Onboarding};
use crate::client;
use crate::names::{generate_room_name, generate_username};
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
                Response::JoinReply(reply) => {
                    info!("JoinReply={:?}", reply);
                    app.user.online_at = reply.user.online_at;
                }
                Response::Shout(shout) => {
                    info!("Shout={:?}", shout);

                    if !shout.user.uuid.eq(&app.user.uuid) {
                        let message = format!("{}: {}", shout.user.username, shout.message);
                        app.messages.push(message);
                    }
                }
                Response::PresenceDiff(diff) => {
                    info!("PresenceDiff={:?}", diff);

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
                    info!("PresenceState={:?}", state);
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

            if key.code == KeyCode::Up || key.code == KeyCode::Down {
                match app.onboarding {
                    Onboarding::ConfirmingUsername => {
                        app.user.username = generate_username();
                        app.input = app.user.username.clone();
                        return Ok(false);
                    }
                    Onboarding::ConfirmingRoomName => {
                        app.room = generate_room_name();
                        app.input = app.room.clone();
                        return Ok(false);
                    }
                    Onboarding::Completed => (),
                }
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

                match app.onboarding {
                    Onboarding::ConfirmingRoomName => {
                        // set room to input
                        app.room = app.input.clone();

                        // advance onboarding
                        app.advance_onboarding();
                    }
                    Onboarding::ConfirmingUsername => {
                        // set username to input
                        app.user.username = app.input.clone();

                        // send join request
                        let request = app.join_request();
                        handle.call(request).expect("join error");

                        // advance onboarding
                        app.advance_onboarding();
                    }
                    Onboarding::Completed => {
                        // Handle normal messages
                        let local_message = format!("{}: {}", &app.user.username, app.input);
                        app.messages.push(local_message);

                        // send shout request
                        let message = app.input.clone();
                        let request = app.shout_request(message);
                        handle.call(request).expect("shout request error");

                        // clear input
                        app.input.clear();
                    }
                }

                return Ok(false);

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
            } else if key.code == KeyCode::Backspace {
                // if confirming username and input is the username, clear entire input on backspace
                if app.onboarding == Onboarding::ConfirmingUsername
                    && app.input == app.user.username
                {
                    app.input.clear();
                }
                // if confirming room and input is room name, clear entire input on backspace
                else if app.onboarding == Onboarding::ConfirmingRoomName && app.input == app.room
                {
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

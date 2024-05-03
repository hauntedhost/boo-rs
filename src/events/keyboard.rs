use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{error, info};

use crate::app::{is_blank, AppState, Onboarding};
use crate::names::{generate_room_name, generate_username};
use crate::socket::client;

pub fn handle_key_event(
    app: &mut AppState,
    handle: &ezsockets::Client<client::Client>,
    key: KeyEvent,
) -> Result<bool, std::io::Error> {
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
        // ignore empty messages
        if !app.input_is_valid_message() {
            return Ok(false);
        }

        match app.onboarding {
            Onboarding::ConfirmingRoomName => {
                // set room to input and advance onboarding
                app.room = app.input.clone();
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
                // TODO: if app.input.starts_with("/") { delegate to command handler }
                // /join command
                if app.input.starts_with("/join") {
                    let prefix = "/join";
                    let room = app.input.trim()[prefix.len()..].to_string();
                    if !is_blank(&room) {
                        app.input.clear();
                        let new_room = room.trim().to_string();

                        let leave_request = app.leave_request();
                        info!("sending leave request={:?}", leave_request);
                        match handle.call(leave_request) {
                            Ok(_) => {
                                let join_request = app.join_new_request(new_room.clone());
                                handle.call(join_request).expect("join error");

                                app.room = new_room;
                            }
                            Err(error) => error!("leave error: {:?}", error),
                        }
                    }
                    return Ok(false);
                }

                // /quit command
                if app.input.eq("/quit") {
                    return Ok(true);
                }

                // TODO: handle /username to change username
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
                let local_message = format!("{}: {}", &app.user.username, app.input);
                app.add_message(local_message);

                // send shout request
                let message = app.input.clone();
                let request = app.shout_request(message);
                handle.call(request).expect("shout request error");

                // clear input
                app.input.clear();
            }
        }

        return Ok(false);
    } else if key.code == KeyCode::Backspace {
        // if confirming username and input is the username, clear entire input on backspace
        if app.onboarding == Onboarding::ConfirmingUsername && app.input == app.user.username {
            app.input.clear();
        }
        // if confirming room and input is room name, clear entire input on backspace
        else if app.onboarding == Onboarding::ConfirmingRoomName && app.input == app.room {
            app.input.clear();
        } else {
            app.input.pop();
        }
    } else if let KeyCode::Char(c) = key.code {
        app.input.push(c);
    }

    Ok(false)
}

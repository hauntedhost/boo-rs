use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{debug, error};

use crate::app::{is_blank, AppState, Onboarding};
use crate::names::{generate_room_name, generate_username};
use crate::socket::client;

enum KeyType {
    Backspace,
    Exit,
    Input(char),
    SubmitCommand,
    SubmitInput,
    ToggleSidebar,
    UpOrDown,
    Other,
}

pub fn handle_key_event(
    app: &mut AppState,
    handle: &ezsockets::Client<client::Client>,
    key: KeyEvent,
) -> Result<bool, std::io::Error> {
    match parse_key_type(app, key) {
        KeyType::Backspace => handle_backspace(app),
        KeyType::Exit => return Ok(true),
        KeyType::Input(c) => app.input.push(c),
        KeyType::SubmitCommand => return handle_command(app, handle),
        KeyType::SubmitInput => handle_submit_input(app, handle),
        KeyType::ToggleSidebar => app.toggle_sidebar(),
        KeyType::UpOrDown => handle_up_or_down_key(app),
        KeyType::Other => (),
    }

    Ok(false)
}

fn handle_backspace(app: &mut AppState) {
    if should_clear_all_input(app) {
        app.input.clear();
    } else {
        app.input.pop();
    }
}

// clear entire input on backspace if:
//   - confirming username and input is the username, or
//   - confirming room and input is room name
fn should_clear_all_input(app: &mut AppState) -> bool {
    (app.onboarding == Onboarding::ConfirmingUsername && app.input == app.user.username)
        || (app.onboarding == Onboarding::ConfirmingRoomName && app.input == app.room)
}

fn handle_command(
    app: &mut AppState,
    handle: &ezsockets::Client<client::Client>,
) -> Result<bool, std::io::Error> {
    // commands are only available after onboarding is completed
    if app.onboarding != Onboarding::Completed {
        return Ok(false);
    }

    // join command
    if app.input.starts_with("/join") {
        let prefix = "/join";
        let room = app.input.trim()[prefix.len()..].to_string();
        if !is_blank(&room) {
            app.input.clear();
            let new_room = room.trim().to_string();

            let leave_request = app.leave_request();
            debug!("sending leave request={:?}", leave_request);
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

    // quit command
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

    Ok(false)
}

fn handle_submit_input(app: &mut AppState, handle: &ezsockets::Client<client::Client>) {
    if !app.input_is_valid_message() {
        return;
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
}

fn handle_up_or_down_key(app: &mut AppState) {
    match app.onboarding {
        Onboarding::ConfirmingUsername => {
            app.user.username = generate_username();
            app.input = app.user.username.clone();
        }
        Onboarding::ConfirmingRoomName => {
            app.room = generate_room_name();
            app.input = app.room.clone();
        }
        Onboarding::Completed => (),
    }
}

fn parse_key_type(app: &mut AppState, key: KeyEvent) -> KeyType {
    if key.code == KeyCode::Backspace {
        return KeyType::Backspace;
    }

    if is_exit_key(key) {
        return KeyType::Exit;
    }

    if let KeyCode::Char(c) = key.code {
        return KeyType::Input(c);
    }

    if key.code == KeyCode::Enter {
        if app.input.starts_with("/") {
            return KeyType::SubmitCommand;
        } else {
            return KeyType::SubmitInput;
        }
    }

    if is_toggle_sidebar_key(key) {
        return KeyType::ToggleSidebar;
    }

    if is_up_or_down_key(key) {
        return KeyType::UpOrDown;
    }

    KeyType::Other
}

fn is_exit_key(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
}

fn is_toggle_sidebar_key(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('s')
}

fn is_up_or_down_key(key: KeyEvent) -> bool {
    key.code == KeyCode::Up || key.code == KeyCode::Down
}

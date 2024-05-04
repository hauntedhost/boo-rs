use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::{debug, error};

use crate::app::{is_blank, AppState, Focus, Onboarding};
use crate::names::{generate_room_name, generate_username};
use crate::socket::client;

#[derive(Debug, Default)]
enum Command {
    ChangeUsername,
    SwitchRoomsFromInput,
    SwitchRoomsFromSelected,
    #[default]
    Unknown,
}

#[derive(Debug, Default)]
enum KeyAction {
    AppendInputChar(char),
    ClearInput,
    ConfirmUsername,
    ConfirmRoomNameAndJoin,
    CycleFocus,
    DeleteLastInputChar,
    QuitApp,
    SelectNextRoom,
    SelectPrevRoom,
    SetInputToRandomRoom,
    SetInputToRandomUsername,
    SubmitCommand(Command),
    SubmitMessage,
    ToggleHelp,
    ToggleRightSidebar,
    #[default]
    Ignore,
}

pub fn handle_key_event(
    app: &mut AppState,
    handle: &ezsockets::Client<client::Client>,
    key: KeyEvent,
) {
    match parse_key_action(app, key) {
        KeyAction::AppendInputChar(c) => app.input.push(c),
        KeyAction::ClearInput => app.input.clear(),
        KeyAction::ConfirmRoomNameAndJoin => handle_confirm_room_name_and_join(app, handle),
        KeyAction::ConfirmUsername => handle_confirm_username(app),
        KeyAction::CycleFocus => app.cycle_focus(),
        KeyAction::DeleteLastInputChar => handle_delete_last_input_char(app),
        KeyAction::QuitApp => app.quit(),
        KeyAction::SelectNextRoom => app.select_next_room(),
        KeyAction::SelectPrevRoom => app.select_prev_room(),
        KeyAction::SetInputToRandomRoom => set_input_to_random_room(app),
        KeyAction::SetInputToRandomUsername => set_input_to_random_username(app),
        KeyAction::SubmitCommand(command) => handle_command(command, app, handle),
        KeyAction::SubmitMessage => handle_submit_message(app, handle),
        KeyAction::ToggleHelp => app.toggle_show_help(),
        KeyAction::ToggleRightSidebar => app.toggle_right_sidebar(),
        KeyAction::Ignore => (),
    }
}

// KeyAction parsing

fn parse_key_action(app: &mut AppState, key: KeyEvent) -> KeyAction {
    // Any key exits help
    if app.should_show_help() {
        return KeyAction::ToggleHelp;
    }

    if is_quit_key(key) {
        return KeyAction::QuitApp;
    }

    // Option + key actions
    if key.modifiers.contains(KeyModifiers::ALT) {
        return if key.code == KeyCode::Char('h') {
            KeyAction::ToggleHelp
        } else if key.code == KeyCode::Char('s') {
            KeyAction::ToggleRightSidebar
        } else {
            KeyAction::Ignore
        };
    }

    if key.code == KeyCode::Tab {
        return if app.onboarding == Onboarding::Completed {
            KeyAction::CycleFocus
        } else {
            KeyAction::Ignore
        };
    }

    if app.ui_focus_area == Focus::Rooms {
        return if key.code == KeyCode::Enter {
            KeyAction::SubmitCommand(Command::SwitchRoomsFromSelected)
        } else if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
            KeyAction::SelectPrevRoom
        } else if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
            KeyAction::SelectNextRoom
        } else {
            KeyAction::Ignore
        };
    }

    if key.code == KeyCode::Backspace {
        if should_clear_all_input(app) {
            return KeyAction::ClearInput;
        } else {
            return KeyAction::DeleteLastInputChar;
        }
    }

    if key.code == KeyCode::Enter {
        if app.input.starts_with("/") {
            return if app.onboarding == Onboarding::Completed {
                if app.input.starts_with("/help") || app.input.starts_with("/?") {
                    KeyAction::ToggleHelp
                } else if app.input.starts_with("/join") {
                    KeyAction::SubmitCommand(Command::SwitchRoomsFromInput)
                } else if app.input.starts_with("/quit") {
                    KeyAction::QuitApp
                } else if app.input.starts_with("/username") {
                    KeyAction::SubmitCommand(Command::ChangeUsername)
                } else {
                    KeyAction::SubmitCommand(Command::Unknown)
                }
            } else {
                KeyAction::SubmitCommand(Command::Unknown)
            };
        }

        return match app.onboarding {
            Onboarding::Completed => KeyAction::SubmitMessage,
            Onboarding::ConfirmingRoom => KeyAction::ConfirmRoomNameAndJoin,
            Onboarding::ConfirmingUsername => KeyAction::ConfirmUsername,
        };
    }

    if is_up_or_down_key(key) {
        return match app.onboarding {
            Onboarding::Completed => KeyAction::Ignore,
            Onboarding::ConfirmingRoom => KeyAction::SetInputToRandomRoom,
            Onboarding::ConfirmingUsername => KeyAction::SetInputToRandomUsername,
        };
    }

    if let KeyCode::Char(c) = key.code {
        return KeyAction::AppendInputChar(c);
    }

    KeyAction::Ignore
}

fn is_quit_key(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
}

fn is_up_or_down_key(key: KeyEvent) -> bool {
    key.code == KeyCode::Up || key.code == KeyCode::Down
}

// clear entire input on backspace if:
//   - confirming username and input is the username, or
//   - confirming room and input is room name
fn should_clear_all_input(app: &mut AppState) -> bool {
    (app.onboarding == Onboarding::ConfirmingRoom && app.input == app.room)
        || (app.onboarding == Onboarding::ConfirmingUsername && app.input == app.user.username)
}

// KeyAction handlers

fn handle_submit_message(app: &mut AppState, handle: &ezsockets::Client<client::Client>) {
    if !app.input_is_valid_message() {
        return;
    }

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

fn handle_delete_last_input_char(app: &mut AppState) {
    app.input.pop();
}

fn handle_command(
    command: Command,
    app: &mut AppState,
    handle: &ezsockets::Client<client::Client>,
) {
    match command {
        Command::ChangeUsername => {
            // TODO: change username
            //   1. push this message uniquely, e.g. "user x has changed their name to y"
            //   2. the server needs to handle the change too
            //   3. the rx.try_recv() also has to handle the name change broadcast
        }
        Command::SwitchRoomsFromInput => {
            // Note: similar logic to SwitchRoomsFromSelected, except:
            // - use room from input
            // - clear input
            let prefix = "/join ";
            let new_room = app.input.trim()[prefix.len()..].to_string();
            if !is_blank(&new_room) && new_room != app.room {
                app.input.clear();
                let leave_request = app.leave_request();
                debug!("sending leave request={:?}", leave_request);
                match handle.call(leave_request) {
                    Ok(_) => {
                        let join_request = app.join_new_room_request(new_room.clone());
                        handle.call(join_request).expect("join error");
                        app.room = new_room;
                        app.set_selected_to_current_room();
                    }
                    Err(error) => error!("leave error: {:?}", error),
                }
            }
        }
        Command::SwitchRoomsFromSelected => {
            // Note: similar logic to SwitchRoomsFromInput, except:
            // - use room selected in rooms list without clearing input
            // - change focus after join
            if let Some(new_room) = app.get_selected_room_name() {
                if new_room != app.room {
                    let leave_request = app.leave_request();
                    debug!("sending leave request={:?}", leave_request);
                    match handle.call(leave_request) {
                        Ok(_) => {
                            let join_request = app.join_new_room_request(new_room.clone());
                            handle.call(join_request).expect("join error");
                            app.room = new_room;
                            app.set_selected_to_current_room();
                            app.ui_focus_area = Focus::Input;
                        }
                        Err(error) => error!("leave error: {:?}", error),
                    }
                }
            }
        }
        Command::Unknown => (),
    }
}

// KeyAction handlers: Onboarding

// set room name to input, send join request and advance onboarding
fn handle_confirm_room_name_and_join(
    app: &mut AppState,
    handle: &ezsockets::Client<client::Client>,
) {
    app.room = app.input.clone();
    let request = app.join_request();
    handle.call(request).expect("join error");
    app.advance_onboarding();
}

// set username to input and advance onboarding
fn handle_confirm_username(app: &mut AppState) {
    app.user.username = app.input.clone();
    app.advance_onboarding();
}

fn set_input_to_random_room(app: &mut AppState) {
    app.room = generate_room_name();
    app.input = app.room.clone();
}

fn set_input_to_random_username(app: &mut AppState) {
    app.user.username = generate_username();
    app.input = app.user.username.clone();
}

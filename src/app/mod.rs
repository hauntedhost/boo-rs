pub mod room;
pub mod user;
use crate::app::{room::Room, user::User};
use crate::names::generate_valid_room_name;
use crate::socket::request::Request;
use crate::ui::widgets::logs::Log;
use chrono::{DateTime, Utc};
use ratatui::widgets::TableState;
use regex::Regex;
use std::env;
use std::time::{Duration, Instant};
use url::Url;

/// This module contains the AppState struct used to store the state of the application.

const HEARTBEAT_INTERVAL: Duration = Duration::new(30, 0); // 30 seconds
const SOCKET_ACTIVITY_DURATION: Duration = Duration::new(0, 500_000_000); // 0.5 seconds

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Onboarding {
    #[default]
    ConfirmingUsername, // Initial step of onboarding
    ConfirmingRoom, // Next step of onboarding
    Completed,      // Onboarding is complete
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    #[default]
    Input,
    Rooms,
}

#[derive(Debug, Default, PartialEq)]
pub enum RightSidebar {
    #[default]
    Rooms,
    Logs,
}

// TODO: implement SystemError
#[derive(Clone, Debug)]
pub enum Message {
    SystemInternal(String),
    SystemPublic(String),
    // SystemError(String),
    User(UserMessage),
}

#[derive(Clone, Debug)]
pub struct UserMessage {
    pub username: String,
    pub content: String,
    pub sent_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SocketStatus {
    #[default]
    Closed,
    Connected,
    ConnectFailed,
    Disconnected,
}

// TODO: store users and rooms as HashMap<String, User/Room> to allow for quick adds and removes
// TODO: nest ui state in a struct

#[derive(Debug)]
pub struct AppState {
    pub input: String,
    pub onboarding: Onboarding,
    pub room: String,
    pub socket_status: SocketStatus,
    pub socket_url: Option<String>,
    pub ui_focus_area: Focus,
    pub ui_input_width: u16,
    pub ui_messages_area_height: usize,
    pub ui_messages_line_length: usize,
    pub ui_messages_scrollbar_position: usize,
    pub ui_right_sidebar_view: RightSidebar,
    pub ui_room_table_state: TableState,
    pub user: User,
    last_heartbeat: Instant,
    logging_enabled: bool,
    logs: Vec<Log>,
    messages: Vec<Message>,
    quitting: bool,
    rooms: Vec<Room>,
    showing_help: bool,
    socket_activity: bool,
    socket_last_active: Instant,
    ui_selected_room_index: Option<usize>,
    users: Vec<User>,
}

impl Default for AppState {
    fn default() -> Self {
        let room = room_from_env_or_generate();
        let user = User::new_from_env_or_generate();

        // Initial input is set to username, to prefill for ConfirmingUsername onboarding step
        let initial_input = user.username.clone();

        AppState {
            input: initial_input,
            last_heartbeat: Instant::now(),
            logging_enabled: true,
            logs: Vec::new(),
            messages: Vec::new(),
            onboarding: Onboarding::default(),
            quitting: false,
            room: room.clone(),
            rooms: Vec::new(),
            showing_help: false,
            socket_activity: false,
            socket_last_active: Instant::now(),
            socket_status: SocketStatus::default(),
            socket_url: None,
            ui_focus_area: Focus::default(),
            ui_input_width: 0,
            ui_messages_area_height: 0,
            ui_messages_line_length: 0,
            ui_messages_scrollbar_position: 0,
            ui_right_sidebar_view: RightSidebar::default(),
            ui_room_table_state: TableState::default(),
            ui_selected_room_index: None,
            user: user.clone(),
            users: Vec::new(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    // heartbeat

    pub fn update_heartbeat_timer(&mut self) -> bool {
        if self.last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
            self.last_heartbeat = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn set_input_width(&mut self, width: u16) {
        self.ui_input_width = width;
    }

    // messages scrollbar

    pub fn get_scrollbar_position(&self) -> usize {
        self.ui_messages_scrollbar_position
    }

    pub fn set_messages_line_length_and_area_height(
        &mut self,
        line_length: usize,
        area_height: usize,
    ) {
        let scrollbar_was_at_bottom = self.is_messages_scrollbar_at_bottom();

        self.ui_messages_line_length = line_length;
        self.ui_messages_area_height = area_height;

        // TODO: add proportional scrollbar update

        if scrollbar_was_at_bottom {
            self.ui_messages_scrollbar_position = line_length.saturating_sub(area_height);
        }
    }

    fn is_messages_scrollbar_at_bottom(&self) -> bool {
        let bottom_position = self
            .ui_messages_line_length
            .saturating_sub(self.ui_messages_area_height);
        self.ui_messages_scrollbar_position == bottom_position
    }

    pub fn scroll_messages_up(&mut self) {
        self.update_scroll_position(-1)
    }

    pub fn scroll_messages_down(&mut self) {
        self.update_scroll_position(1)
    }

    fn update_scroll_position(&mut self, delta: isize) {
        let new_position = (self.ui_messages_scrollbar_position as isize + delta).max(0) as usize;
        let max_position = self
            .ui_messages_line_length
            .saturating_sub(self.ui_messages_area_height);
        self.ui_messages_scrollbar_position = new_position.min(max_position);
    }

    // socket activity

    pub fn is_socket_active(&self) -> bool {
        self.socket_activity
    }

    pub fn set_socket_activity(&mut self) {
        self.socket_activity = true;
        self.socket_last_active = Instant::now();
    }

    pub fn tick_socket_activity(&mut self) {
        if self.socket_activity && self.socket_last_active.elapsed() >= SOCKET_ACTIVITY_DURATION {
            self.socket_activity = false;
        }
    }

    // quitting app

    pub fn quitting(&self) -> bool {
        self.quitting
    }

    pub fn quit(&mut self) {
        self.quitting = true;
    }

    // socket_url

    pub fn set_socket_url(&mut self, url: Url) {
        self.socket_url = if let Some(host) = url.host_str() {
            Some(format!("{}://{}", url.scheme(), host))
        } else {
            None
        };
    }

    // onboarding

    pub fn advance_onboarding(&mut self) {
        match self.onboarding {
            Onboarding::ConfirmingUsername => {
                // advance to confirming room name, set input to initial room name
                self.onboarding = Onboarding::ConfirmingRoom;
                self.input = self.room.clone();
            }
            Onboarding::ConfirmingRoom => {
                // advance to completed, clear input
                self.onboarding = Onboarding::Completed;
                self.input.clear();
            }
            Onboarding::Completed => (),
        };
    }

    // input

    pub fn input_is_valid_command(&self) -> bool {
        let re = Regex::new(r"^\/[a-zA-Z0-9]{1,10}($| [a-zA-Z0-9\-]{1,30}$)").unwrap();
        re.is_match(&self.input)
    }

    pub fn is_valid_next_char_for_input_command(&self, c: char) -> bool {
        let new_input = format!("{}{}", self.input, c);
        let re = Regex::new(r"^\/[a-zA-Z0-9]{0,10}(| [a-zA-Z0-9\-]{0,30})$").unwrap();
        re.is_match(&new_input)
    }

    // message is not blank and is less than 200 characters
    pub fn input_is_valid_message(&self) -> bool {
        !self.input_is_blank() && self.input.len() <= 200
    }

    pub fn is_valid_next_char_for_input_message(&self, c: char) -> bool {
        let new_message = format!("{}{}", self.input, c);
        new_message.len() <= 200
    }

    // room name is alphanumeric and hyphens, between 3 and 20 characters
    pub fn input_is_valid_room_name(&self) -> bool {
        is_valid_room_or_username(&self.input)
    }

    pub fn is_valid_next_char_for_room_name(&self, c: char) -> bool {
        is_valid_room_or_username_with(&self.input, c)
    }

    // username is alphanumeric and hyphens, between 3 and 20 characters
    pub fn input_is_valid_username(&self) -> bool {
        is_valid_room_or_username(&self.input)
    }

    pub fn is_valid_next_char_for_username(&self, c: char) -> bool {
        is_valid_room_or_username_with(&self.input, c)
    }

    fn input_is_blank(&self) -> bool {
        is_blank(&self.input)
    }

    // rooms

    pub fn get_rooms(&self) -> Vec<Room> {
        self.rooms.clone()
    }

    pub fn set_rooms(&mut self, rooms: Vec<Room>) {
        let mut new_rooms = rooms.clone();
        new_rooms.sort_by_key(|room| room.name.clone());
        self.rooms = new_rooms;
        self.set_selected_to_current_room()
    }

    pub fn get_rooms_with_counts(&self) -> Vec<(String, u32)> {
        self.get_rooms()
            .iter()
            .map(|room| (room.name.clone(), room.user_count))
            .collect()
    }

    // UI

    pub fn cycle_focus(&mut self) {
        self.ui_focus_area = match self.ui_focus_area {
            Focus::Input => Focus::Rooms,
            Focus::Rooms => Focus::Input,
        };
    }

    pub fn showing_help(&self) -> bool {
        self.showing_help
    }

    pub fn toggle_show_help(&mut self) {
        if self.showing_help {
            self.showing_help = false;
        } else {
            self.showing_help = true;
        }
    }

    pub fn toggle_right_sidebar(&mut self) {
        match self.ui_right_sidebar_view {
            RightSidebar::Rooms => {
                self.ui_focus_area = Focus::Input;
                self.ui_right_sidebar_view = RightSidebar::Logs;
            }
            RightSidebar::Logs => self.ui_right_sidebar_view = RightSidebar::Rooms,
        };
    }

    // UI: ui_selected_room_index

    pub fn get_selected_or_current_room_index(&self) -> Option<usize> {
        self.ui_selected_room_index
            .or_else(|| self.get_current_room_index())
    }

    fn get_current_room_index(&self) -> Option<usize> {
        self.get_rooms()
            .iter()
            .position(|room| room.name == self.room)
    }

    pub fn get_selected_room_name(&self) -> Option<String> {
        match self.ui_selected_room_index {
            Some(index) => Some(self.get_rooms()[index].name.clone()),
            None => None,
        }
    }

    pub fn set_selected_to_current_room(&mut self) {
        self.ui_selected_room_index = self.get_rooms().iter().position(|r| r.name == self.room);
    }

    pub fn select_next_room(&mut self) {
        if let Some(index) = self.get_selected_or_current_room_index() {
            let next_index = (index + 1) % self.get_rooms().len();
            self.ui_selected_room_index = Some(next_index);
        }
    }

    pub fn select_prev_room(&mut self) {
        if let Some(index) = self.get_selected_or_current_room_index() {
            let num_rooms = self.get_rooms().len();
            let prev_index = (index + num_rooms - 1) % num_rooms;
            self.ui_selected_room_index = Some(prev_index);
        }
    }

    // users

    pub fn get_username(&self) -> String {
        self.user.username.clone()
    }

    pub fn add_user(&mut self, user: User) {
        if !self.users.iter().any(|u| u.uuid == user.uuid) {
            self.users.push(user);
        }
    }

    pub fn get_users(&self) -> Vec<User> {
        self.get_users_sorted()
    }

    pub fn get_uuid_username_pairs(&self) -> Vec<(String, String)> {
        self.get_users()
            .iter()
            .map(|u| (u.uuid.clone(), u.username.clone()))
            .collect()
    }

    pub fn remove_user(&mut self, user: User) {
        self.users.retain(|u| u.uuid != user.uuid);
    }

    pub fn set_users(&mut self, users: Vec<User>) {
        self.users = users;
    }

    fn get_users_sorted(&self) -> Vec<User> {
        let mut users = self.users.clone();
        users.sort_by_key(|user| user.username.clone());
        users
    }

    // messages

    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    pub fn add_user_message(&mut self, user: User, content: String) {
        self.add_message(Message::User(UserMessage {
            username: user.username.clone(),
            content,
            sent_at: Utc::now(),
        }));
    }

    pub fn add_system_internal_message(&mut self, message: String) {
        self.add_message(Message::SystemInternal(message.clone()));
        // self.maybe_scroll_messages_down()
    }

    pub fn add_system_public_message(&mut self, message: String) {
        self.add_message(Message::SystemPublic(message.clone()));
        // self.maybe_scroll_messages_down()
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        // self.maybe_scroll_messages_down()
    }

    // logs

    pub fn get_logs(&self) -> Vec<Log> {
        self.logs.clone()
    }

    pub fn append_log(&mut self, log: Log) {
        if self.logging_enabled {
            self.logs.push(log);
        }
    }

    // requests

    pub fn heartbeat_request(&mut self) -> Request {
        Request::new_heartbeat()
    }

    pub fn join_request(&mut self) -> Request {
        Request::new_join(self.room.clone(), self.user.clone())
    }

    pub fn join_new_room_request(&mut self, new_room: String) -> Request {
        Request::new_join(new_room, self.user.clone())
    }

    pub fn leave_request(&mut self) -> Request {
        Request::new_leave(self.room.clone())
    }

    pub fn shout_request(&mut self, message: String) -> Request {
        Request::new_shout(self.room.clone(), message)
    }
}

pub fn is_valid_room_or_username(name: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9\-]{3,20}$").unwrap();
    re.is_match(&name)
}

pub fn is_valid_room_or_username_with(name: &str, c: char) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9\-]{1,20}$").unwrap();
    let new_name = format!("{}{}", name, c);
    re.is_match(&new_name)
}

pub fn is_blank(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

// Get room name from ROOM env var, otherwise generate a room name
fn room_from_env_or_generate() -> String {
    match env::var("ROOM") {
        Ok(room) => room,
        Err(_) => generate_valid_room_name(),
    }
}

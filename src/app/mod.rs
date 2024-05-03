pub mod room;
pub mod user;

use ratatui::widgets::TableState;
use std::env;
use std::time::{Duration, Instant};

use crate::app::{room::Room, user::User};
use crate::names::generate_room_name;
use crate::socket::request::Request;

/// This module contains the AppState struct used to store the state of the application.

const HEARTBEAT_INTERVAL: Duration = Duration::new(30, 0);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Onboarding {
    #[default]
    ConfirmingRoomName,
    ConfirmingUsername,
    Completed,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    #[default]
    Input,
    Rooms,
}

#[derive(Debug, Default)]
pub enum Sidebar {
    #[default]
    Users,
    Logs,
}

#[derive(Debug)]
pub struct AppState {
    pub input: String,
    pub onboarding: Onboarding,
    pub room: String,
    pub user: User,
    // TODO: store users and rooms as HashMap<String, User/Room> to allow for quick adds and removes
    rooms: Vec<Room>,
    users: Vec<User>,
    // TODO: nest ui state in a struct
    pub ui_room_table_state: TableState,
    pub ui_sidebar_view: Sidebar,
    pub ui_focus_area: Focus,
    ui_selected_room_index: Option<usize>,
    last_heartbeat: Instant,
    logs: Vec<String>,
    logs_enabled: bool,
    messages: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let room = room_from_env_or_generate();

        // Initial input is set to the room name to prefill for onboarding
        let initial_input = room.clone();

        AppState {
            input: initial_input,
            last_heartbeat: Instant::now(),
            logs: Vec::new(),
            logs_enabled: true,
            messages: Vec::new(),
            onboarding: Onboarding::default(),
            room: room.clone(),
            ui_room_table_state: TableState::default(),
            ui_focus_area: Focus::default(),
            ui_sidebar_view: Sidebar::default(),
            rooms: Vec::new(),
            ui_selected_room_index: None,
            user: User::new_from_env_or_generate(),
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

    // onboarding

    pub fn advance_onboarding(&mut self) {
        match self.onboarding {
            Onboarding::ConfirmingRoomName => {
                // advance to confirming username, set input to initial username
                self.onboarding = Onboarding::ConfirmingUsername;
                self.input = self.user.username.clone();
            }
            Onboarding::ConfirmingUsername => {
                // advance to completed, clear input
                self.onboarding = Onboarding::Completed;
                self.input.clear();
            }
            Onboarding::Completed => (),
        };
    }

    // input

    pub fn input_is_valid_message(&self) -> bool {
        !self.input_is_blank()
    }

    pub fn input_is_blank(&self) -> bool {
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

    pub fn toggle_sidebar(&mut self) {
        self.ui_sidebar_view = match self.ui_sidebar_view {
            Sidebar::Users => Sidebar::Logs,
            Sidebar::Logs => Sidebar::Users,
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

    pub fn get_messages(&self) -> Vec<String> {
        self.messages.clone()
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    // logs

    pub fn get_logs(&self) -> Vec<String> {
        self.logs.clone()
    }

    pub fn append_log(&mut self, log: String) {
        if self.logs_enabled {
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

pub fn is_blank(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

// Get room name from ROOM env var, otherwise generate a room name
fn room_from_env_or_generate() -> String {
    match env::var("ROOM") {
        Ok(room) => room,
        Err(_) => generate_room_name(),
    }
}

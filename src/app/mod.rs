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

#[derive(Debug, Default)]
pub enum Sidebar {
    #[default]
    Users,
    Logs,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Onboarding {
    #[default]
    ConfirmingRoomName,
    ConfirmingUsername,
    Completed,
}

#[derive(Debug)]
pub struct AppState {
    pub input: String,
    pub onboarding: Onboarding,
    pub room: String,
    pub room_table_state: TableState,
    pub sidebar: Sidebar,
    pub user: User,
    last_heartbeat: Instant,
    logs: Vec<String>,
    logs_enabled: bool,
    messages: Vec<String>,
    // TODO: store users and rooms as HashMap<String, User/Room> to allow for quick adds and removes
    rooms: Vec<Room>,
    users: Vec<User>,
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
            room_table_state: TableState::default(),
            rooms: Vec::new(),
            sidebar: Sidebar::default(),
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
        self.get_rooms_sorted()
    }

    pub fn set_rooms(&mut self, rooms: Vec<Room>) {
        self.rooms = rooms;
    }

    pub fn get_rooms_with_counts(&self) -> Vec<(String, u32)> {
        self.get_rooms()
            .iter()
            .map(|room| (room.name.clone(), room.user_count))
            .collect()
    }

    pub fn get_room_index(&self) -> Option<usize> {
        self.get_rooms()
            .iter()
            .position(|room| room.name == self.room)
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

    pub fn toggle_sidebar(&mut self) {
        self.sidebar = match self.sidebar {
            Sidebar::Users => Sidebar::Logs,
            Sidebar::Logs => Sidebar::Users,
        };
    }

    // requests

    pub fn heartbeat_request(&mut self) -> Request {
        Request::heartbeat(self.user.clone())
    }

    pub fn join_request(&mut self) -> Request {
        Request::join(self.room.clone(), self.user.clone())
    }

    pub fn join_new_request(&mut self, new_room: String) -> Request {
        Request::join(new_room, self.user.clone())
    }

    pub fn leave_request(&mut self) -> Request {
        Request::leave(self.room.clone(), self.user.clone())
    }

    pub fn shout_request(&mut self, message: String) -> Request {
        Request::shout(self.room.clone(), message, self.user.clone())
    }

    // private

    fn get_users_sorted(&self) -> Vec<User> {
        let mut users = self.users.clone();
        users.sort_by_key(|user| user.username.clone());
        users
    }

    fn get_rooms_sorted(&self) -> Vec<Room> {
        let mut rooms = self.rooms.clone();
        rooms.sort_by_key(|room| (!room.name.eq(&self.room), room.name.clone()));
        rooms
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

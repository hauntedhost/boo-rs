use std::env;

/// This module contains the AppState struct used to store the state of the application.
use crate::names::generate_room_name;
use crate::request::Request;
use crate::room::Room;
use crate::user::User;

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
    pub user: User,
    // TODO: store users as a HashMap<String, User> to allow for quick adds and removes
    pub users: Vec<User>,
    pub room: String,
    // TODO: store rooms as a HashMap<String, User> to allow for quick adds and removes
    pub rooms: Vec<Room>,
    pub messages: Vec<String>,
    pub input: String,
    pub sidebar: Sidebar,
    pub onboarding: Onboarding,
    logs: Vec<String>,
    logs_enabled: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let room = room_from_env_or_generate();

        // Initial input is set to the room name to prefill for onboarding
        let initial_input = room.clone();

        AppState {
            user: User::new_from_env_or_generate(),
            users: Vec::new(),
            room: room.clone(),
            rooms: Vec::new(),
            messages: Vec::new(),
            input: initial_input,
            sidebar: Sidebar::default(),
            onboarding: Onboarding::default(),
            logs: Vec::new(),
            logs_enabled: true,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn input_is_valid_message(&self) -> bool {
        !is_blank(&self.input)
    }

    pub fn get_rooms_with_counts(&self) -> Vec<(String, u32)> {
        let rooms_with_counts = self
            .rooms
            .iter()
            .map(|room| (room.name.clone(), room.user_count))
            .collect();

        rooms_with_counts
    }

    pub fn add_user(&mut self, user: User) {
        if !self.users.iter().any(|u| u.uuid == user.uuid) {
            self.users.push(user);
        }
    }

    pub fn remove_user(&mut self, user: User) {
        self.users.retain(|u| u.uuid != user.uuid);
    }

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

    // Return a vector of username strings
    pub fn get_usernames(&self) -> Vec<String> {
        self.users.iter().map(|u| u.username.clone()).collect()
    }

    // Build a join request
    pub fn join_request(&mut self) -> Request {
        Request::join(self.room.clone(), self.user.clone())
    }

    // Build a shout request
    pub fn shout_request(&mut self, message: String) -> Request {
        Request::shout(self.room.clone(), message, self.user.clone())
    }
}

// Get room name from ROOM env var, otherwise generate a room name
fn room_from_env_or_generate() -> String {
    match env::var("ROOM") {
        Ok(room) => room,
        Err(_) => generate_room_name(),
    }
}

fn is_blank(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

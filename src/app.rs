// This module contains the AppState struct used to store the state of the application.
use crate::user::User;

#[derive(Debug)]
pub struct AppState {
    pub user: User,
    pub users: Vec<User>,
    pub input: String,
    pub messages: Vec<String>,
    pub logs: Vec<String>,
    pub has_joined: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            user: User::default(),
            users: Vec::new(),
            input: String::new(),
            messages: Vec::new(),
            logs: Vec::new(),
            has_joined: false,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let user = User::new_from_env_or_guest();
        let input = user.username.clone();

        Self {
            user,
            input,
            ..Default::default()
        }
    }
}

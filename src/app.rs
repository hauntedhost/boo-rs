// This module contains the AppState struct used to store the state of the application.
use crate::user::User;

#[derive(Debug, Default)]
pub struct AppState {
    pub user: User,
    pub users: Vec<User>,
    pub input: String,
    pub messages: Vec<String>,
    pub logs: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user: User::new_from_env_or_guest(),
            ..Default::default()
        }
    }
}

// This module contains the AppState struct used to store the state of the application.
use crate::user::User;

#[derive(Debug, Default)]
pub enum Sidebar {
    #[default]
    Users,
    Logs,
}

#[derive(Debug)]
pub struct AppState {
    pub user: User,
    pub users: Vec<User>,
    pub input: String,
    pub messages: Vec<String>,
    pub sidebar: Sidebar,
    logs: Vec<String>,
    logs_enabled: bool,
    has_joined: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            user: User::default(),
            users: Vec::new(),
            input: String::new(),
            messages: Vec::new(),
            logs: Vec::new(),
            sidebar: Sidebar::default(),
            logs_enabled: true,
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

    pub fn get_usernames(&self) -> Vec<String> {
        self.users.iter().map(|u| u.username.clone()).collect()
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar = match self.sidebar {
            Sidebar::Users => Sidebar::Logs,
            Sidebar::Logs => Sidebar::Users,
        };
    }

    pub fn has_joined(&self) -> bool {
        self.has_joined
    }

    pub fn set_has_joined(&mut self) {
        self.has_joined = true;
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.logs.clone()
    }

    pub fn append_log(&mut self, log: String) {
        if self.logs_enabled {
            self.logs.push(log);
        }
    }
}

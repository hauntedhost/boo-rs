use crate::names::generate_username;
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub online_at: u64,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            username,
            ..Default::default()
        }
    }

    // Create a new user from USERNAME env var otherwise generate a guest username
    pub fn new_from_env_or_generate() -> Self {
        match env::var("NAME") {
            Ok(username) => Self::new(username),
            Err(_) => Self::new(generate_username()),
        }
    }

    // Display name is username plus first four characters of uuid
    pub fn display_name(&self) -> String {
        let uuid_bit = self.uuid[0..4].to_string();
        format!("{}#{}", self.username, uuid_bit)
    }
}

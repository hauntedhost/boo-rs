use std::env;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub uuid: String,
    pub online_at: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            uuid: Uuid::new_v4().to_string(),
            ..Default::default()
        }
    }

    pub fn new_from_env_or_guest() -> Self {
        match env::var("NAME") {
            Ok(username) => Self::new(username),
            Err(_) => Self::new_guest(),
        }
    }

    pub fn new_guest() -> Self {
        Self::new(generate_guest_username())
    }

    pub fn display_name(&self) -> String {
        let bit = &self.uuid[0..4];
        format!("{}#{bit}", self.username)
    }
}

fn generate_guest_username() -> String {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(1..10_000);
    format!("guest{n}")
}

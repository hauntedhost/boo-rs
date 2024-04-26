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

    pub fn display_name(&self) -> String {
        let bit = &self.uuid[0..4];
        format!("{}#{bit}", self.username)
    }
}

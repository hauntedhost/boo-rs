use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub uuid: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

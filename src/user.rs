use std::env;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub online_at: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            username,
            ..Default::default()
        }
    }

    // Create a new user from the NAME env var or generate a guest username
    pub fn new_from_env_or_guest() -> Self {
        match env::var("NAME") {
            Ok(username) => Self::new(username),
            Err(_) => Self::new(generate_guest_username()),
        }
    }

    // Display name is username plus first four characters of uuid
    pub fn display_name(&self) -> String {
        let uuid_bit = self.uuid[0..4].to_string();
        format!("{}#{}", self.username, uuid_bit)
    }
}

// Guest username is a combination of two random words plus a random number
fn generate_guest_username() -> String {
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(10..99);
    format!("{}-{}-{}", random_a(), random_b(), n)
}

fn random_a() -> String {
    get_random(vec![
        "ancient", "barren", "chilly", "distant", "eerie", "frozen", "haunted", "hidden", "hollow",
        "lonely", "misty", "moody", "mystic", "quiet", "secret", "silent", "shrouded", "stark",
        "subtle", "sullen", "veiled", "velvet", "windy",
    ])
}

fn random_b() -> String {
    get_random(vec![
        "ash", "beam", "blossom", "castle", "cliff", "cloud", "crow", "crypt", "dust", "field",
        "flame", "fog", "frost", "ghost", "gloom", "glow", "grave", "leaf", "marsh", "mist",
        "moon", "night", "path", "raven", "root", "ruin", "sage", "shade", "snow", "star", "stone",
        "storm", "stream", "thorn", "wolf",
    ])
}

fn get_random(list: Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..list.len());
    list[index].to_string()
}

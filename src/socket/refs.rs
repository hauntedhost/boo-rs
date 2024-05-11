use std::sync::atomic::{AtomicUsize, Ordering};

use crate::names::generate_uuid;

pub struct Refs {
    pub join_ref: String,
    pub message_ref: AtomicUsize,
}

impl Default for Refs {
    fn default() -> Self {
        Self {
            join_ref: generate_uuid(),
            message_ref: AtomicUsize::new(1),
        }
    }
}

impl Refs {
    pub fn get_join_ref(&self) -> String {
        self.join_ref.clone()
    }

    pub fn get_message_ref(&self) -> usize {
        self.message_ref.load(Ordering::SeqCst)
    }
}

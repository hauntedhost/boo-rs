use crate::socket::response::Response as SocketResponse;
use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Clone, Default, Debug)]
pub struct Log {
    pub response: SocketResponse,
    pub logged_at: DateTime<Utc>,
}

impl Log {
    pub fn new(response: SocketResponse) -> Self {
        Self {
            response,
            logged_at: Utc::now(),
        }
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.logged_at, self.response)
    }
}

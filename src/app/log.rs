use chrono::{DateTime, Utc};

use crate::socket::response::Response as SocketResponse;

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

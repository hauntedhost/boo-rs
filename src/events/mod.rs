mod keyboard;
mod socket;

use crossterm::event::{self, Event};
use log::error;
use tokio::sync::mpsc::{self, Receiver};

use crate::app::AppState;
use crate::socket::client;

use self::keyboard::handle_key_event;
use self::socket::handle_message_event;

/// This module contains code for handling events within the main app loop.
/// It exposes a single `handle_events` function which handles both:
///   - incoming messages from the server
///   - keyboard input from the user

pub fn handle_events(
    handle: &ezsockets::Client<client::Client>,
    rx: &mut Receiver<String>,
    app: &mut AppState,
) -> std::io::Result<()> {
    app.tick_socket_socket_activity();

    if app.update_heartbeat_timer() {
        let heartbeat_request = app.heartbeat_request();
        app.set_socket_activity();
        handle.call(heartbeat_request).expect("heartbeat error");
    }

    // Handle incoming messages from the socket
    match rx.try_recv() {
        Ok(message_payload) => handle_message_event(app, message_payload),
        Err(mpsc::error::TryRecvError::Empty) => (),
        Err(error) => error!("rx.try_recv error: {:?}", error),
    }

    // Handle keyboard input from the user
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            handle_key_event(app, handle, key);
        }
    }

    Ok(())
}

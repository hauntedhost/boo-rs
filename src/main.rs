mod client;
mod events;
mod logging;
mod message;
mod request;
mod response;
mod socket;
mod ui;
mod user;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use std::env;
use std::io::{self, stdout};

use crate::events::handle_events;
use crate::logging::setup_logging;
use crate::request::{Join, Request};
use crate::socket::{connect_socket, create_channel};
use crate::user::User;

#[tokio::main]
async fn main() -> io::Result<()> {
    // init app state
    let mut user = User::new(env::var("NAME").ok());
    let mut users: Vec<User> = vec![];
    let mut input = "".to_string();
    let mut messages: Vec<String> = vec![];
    let mut logs: Vec<String> = vec![];
    let mut should_quit = false;

    // init logging
    setup_logging(user.display_name().clone()).expect("failed to initialize logging");
    log::info!("app started");

    // connect websocket
    let (tx, mut rx) = create_channel();
    let (handle, future) = connect_socket(tx).await;

    tokio::spawn(async move {
        future.await.unwrap();
    });

    // ui setup
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // join server
    let request = Request::Join(Join { user: user.clone() });
    handle.call(request).expect("join error");

    // main loop
    while !should_quit {
        // draw ui
        terminal.draw(|f| ui::render(f, &input, &messages, &logs, &users))?;

        // handle events
        should_quit = handle_events(
            &handle,
            &mut rx,
            &mut user,
            &mut users,
            &mut input,
            &mut messages,
            &mut logs,
        )?;
    }

    // cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

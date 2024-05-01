mod app;
mod client;
mod events;
mod logging;
mod message;
mod names;
mod request;
mod response;
mod room;
mod socket;
mod ui;
mod user;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use log::info;
use ratatui::prelude::*;
use std::io::{self, stdout};

use crate::app::AppState;
use crate::events::handle_events;
use crate::logging::setup_logging;
use crate::socket::{connect_socket, create_channel};

#[tokio::main]
async fn main() -> io::Result<()> {
    // init app state
    let mut app = AppState::new();

    // init logging
    setup_logging(app.user.display_name().clone()).expect("failed to initialize logging");
    info!("app started");

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

    // main loop
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui::render(f, &mut app))?; // draw ui
        should_quit = handle_events(&handle, &mut rx, &mut app)?; // handle input/server events
    }

    // cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

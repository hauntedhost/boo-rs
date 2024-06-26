mod app;
mod events;
mod logging;
mod names;
mod socket;
mod ui;
use crate::app::AppState;
use crate::events::handle_events;
use crate::logging::setup_logging;
use crate::socket::{close_socket, connect_socket, create_channel};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use log::info;
use ratatui::prelude::*;
use std::io::stdout;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // init app state
    let mut app = AppState::new();

    // init logging
    setup_logging(app.user.display_name().clone()).expect("failed to initialize logging");
    info!("app started");

    // connect websocket
    app.set_socket_activity();
    let (tx, mut rx) = create_channel();
    let (handle, future) = connect_socket(tx, &mut app).await;

    tokio::spawn(async move {
        future.await.unwrap();
    });

    // ui setup
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // main loop
    while !app.quitting() {
        terminal.draw(|f| ui::render(f, &mut app))?;
        handle_events(&handle, &mut rx, &mut app)?;
    }

    // cleanup
    close_socket(handle)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

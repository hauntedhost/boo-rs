mod client;
mod events;
mod message;
mod request;
mod response;
mod ui;
mod user;

use chrono::Local;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ezsockets::ClientConfig;
use fern::Dispatch;
use rand::Rng;
use ratatui::prelude::*;
use std::env;
use std::io::{self, stdout};
use tokio::sync::mpsc;
use url::Url;

use crate::client::Client;
use crate::events::handle_events;
use crate::request::{Join, Request};
use crate::user::User;

// TODO: move to user.rs
fn get_username() -> String {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(1..10_000);
    let username = env::var("NAME").unwrap_or(format!("guest{n}"));
    username
}

// TODO: where to move all this connect setup logic?
const DEFAULT_BASE_URL: &str = "ws://localhost:4000";

fn get_relay_url() -> Url {
    let mut base_url = env::var("RELAY_URL")
        .unwrap_or(DEFAULT_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string();
    base_url.push_str("/socket/websocket?vsn=2.0.0");
    Url::parse(&base_url).expect("Invalid relay URL")
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut user = User::new(get_username());

    setup_logging(user.display_name().clone()).expect("failed to initialize logging");
    log::info!("app started");

    let relay_url = get_relay_url();
    let config = ClientConfig::new(relay_url);
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let (handle, future) = ezsockets::connect(|handle| Client::new(handle, tx), config).await;

    tokio::spawn(async move {
        future.await.unwrap();
    });

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut users: Vec<User> = vec![];
    let mut input = "".to_string();
    let mut messages: Vec<String> = vec![];
    let mut logs: Vec<String> = vec![];
    let mut should_quit = false;

    let request = Request::Join(Join { user: user.clone() });
    handle.call(request).expect("join error");

    while !should_quit {
        terminal.draw(|f| ui::render(f, &input, &messages, &logs, &users))?;
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

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn setup_logging(username: String) -> Result<(), fern::InitError> {
    let log_file = "logs/app.log";

    // file based logging
    let file_config = Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                username,
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_file)?);

    file_config.apply()?;

    Ok(())
}

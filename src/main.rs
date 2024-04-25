mod client;
mod events;
mod message;
mod ui;
mod user;

use client::{Call, Client};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use events::handle_events;
use ezsockets::ClientConfig;
use rand::Rng;
use ratatui::prelude::*;
use std::env;
use std::io::{self, stdout};
use tokio::sync::mpsc;
use url::Url;
use user::User;

const DEFAULT_BASE_URL: &str = "ws://localhost:4000";

fn get_username() -> String {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(1..10_000);
    let username = env::var("NAME").unwrap_or(format!("guest{n}"));
    username
}

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
    // tracing_subscriber::fmt().init();
    tracing_subscriber::fmt().with_env_filter("off").init();

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

    let mut user = User::new(get_username());
    let mut input = "".to_string();
    let mut messages: Vec<String> = vec![];
    let mut logs: Vec<String> = vec![];
    let mut should_quit = false;

    handle.call(Call::Join(user.clone())).expect("join error");

    while !should_quit {
        terminal.draw(|f| ui::render(f, &input, &messages, &logs))?;
        should_quit = handle_events(
            &mut user,
            &mut input,
            &mut messages,
            &mut logs,
            &mut rx,
            &handle,
        )?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

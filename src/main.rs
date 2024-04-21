mod client;
mod message;

use client::{Call, Client};
use ezsockets::ClientConfig;
use std::{env, io::BufRead};
use url::Url;

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
async fn main() {
    tracing_subscriber::fmt::init();

    let relay_url = get_relay_url();
    let config = ClientConfig::new(relay_url);
    let (handle, future) = ezsockets::connect(|handle| Client { handle }, config).await;

    tokio::spawn(async move {
        future.await.unwrap();
    });

    handle.call(Call::Join).expect("call join error");

    let message = "Hello world!".to_string();
    handle.call(Call::Shout(message)).expect("call shout error");

    let stdin = std::io::stdin();
    let lines = stdin.lock().lines();

    for line in lines {
        let line = line.unwrap();
        tracing::info!("sending {line}");
        handle.call(Call::Shout(line)).expect("call shout error");
    }
}

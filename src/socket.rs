use ezsockets::ClientConfig;
use std::env;
use std::future::Future;
use tokio::sync::mpsc;
use url::Url;

use crate::client::Client;

const DEFAULT_BASE_URL: &str = "ws://localhost:4000";

fn get_relay_url() -> Url {
    let mut base_url = env::var("RELAY_URL")
        .unwrap_or(DEFAULT_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string();
    base_url.push_str("/socket/websocket?vsn=2.0.0");
    Url::parse(&base_url).expect("Invalid relay URL")
}

pub fn create_channel() -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    mpsc::channel::<String>(32)
}

pub async fn connect_socket(
    tx: mpsc::Sender<String>,
) -> (
    ezsockets::Client<Client>,
    impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
) {
    let relay_url = get_relay_url();
    let config = ClientConfig::new(relay_url);

    ezsockets::connect(|handle| Client::new(handle, tx), config).await
}

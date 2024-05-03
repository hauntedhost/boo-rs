pub mod client;
pub mod message;
pub mod request;
pub mod response;

use ezsockets::ClientConfig;
use log::info;
use std::env;
use std::future::Future;
use tokio::sync::mpsc;
use url::Url;

use crate::socket::client::Client;

const DEFAULT_URL: &str = "wss://chat.haunted.host";
const DEV_URL: &str = "ws://localhost:4000";

pub fn create_channel() -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    mpsc::channel::<String>(32)
}

pub async fn connect_socket(
    tx: mpsc::Sender<String>,
) -> (
    ezsockets::Client<Client>,
    impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
) {
    let socket_url = get_socket_url();
    let config = ClientConfig::new(socket_url.clone());

    info!("connecting to websocket {} ...", socket_url);

    ezsockets::connect(|handle| Client::new(handle, tx), config).await
}

fn get_socket_url() -> Url {
    let base_url = if env::var("DEV").unwrap_or_default() == "true" {
        DEV_URL.to_string()
    } else if let Ok(custom_url) = env::var("URL") {
        custom_url
    } else {
        DEFAULT_URL.to_string()
    };

    let mut url = Url::parse(&base_url).expect("Error parsing URL");

    url.set_path("/socket/websocket");
    url.set_query(Some("vsn=2.0.0"));

    // default to secure wss if not specified
    match url.scheme() {
        "ws" | "wss" => (),
        _ => url.set_scheme("wss").unwrap(),
    }

    url
}

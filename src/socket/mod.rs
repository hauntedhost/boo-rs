pub mod client;
pub mod message;
pub mod refs;
pub mod request;
pub mod response;

use crate::app::AppState;
use crate::socket::client::Client;
use ezsockets::ClientConfig;
use std::env;
use std::future::Future;
use tokio::sync::mpsc;
use url::Url;

use self::client::SocketEvent;

const DEFAULT_URL: &str = "wss://chat.haunted.host";
const DEV_URL: &str = "ws://localhost:4000";

pub fn create_channel() -> (mpsc::Sender<SocketEvent>, mpsc::Receiver<SocketEvent>) {
    mpsc::channel::<SocketEvent>(32)
}

pub async fn connect_socket(
    tx: mpsc::Sender<SocketEvent>,
    app: &mut AppState,
) -> (
    ezsockets::Client<Client>,
    impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
) {
    let socket_url = get_socket_url();
    log::info!("connecting to websocket {} ...", socket_url);

    let config = ClientConfig::new(socket_url.clone());
    app.set_socket_url(socket_url.clone());
    ezsockets::connect(|handle| Client::new(handle, tx), config).await
}

pub fn close_socket(handle: ezsockets::Client<Client>) -> std::io::Result<()> {
    log::info!("closing websocket");

    match handle.close(None) {
        Ok(_) => {
            log::info!("websocket closed");
            Ok(())
        }
        Err(e) => {
            log::error!("failed to close socket={:?}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to close socket={:?}", e),
            ))
        }
    }
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

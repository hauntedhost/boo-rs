use crate::message::{self, message_text, Message};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct Client {
    pub handle: ezsockets::Client<Self>,
    pub tx: mpsc::Sender<String>,
}

pub enum Call {
    Join(String),
    Shout(String, String),
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        tracing::info!("received message: {text}");

        if let Err(e) = self.tx.send(text).await {
            tracing::error!("Error sending message to terminal: {e}");
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        tracing::info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        match call {
            Call::Join(join_ref) => {
                tracing::info!("sending join request");

                let text = message_text(Message {
                    event: message::Event::Join,
                    join_ref: Some(join_ref),
                    ..Default::default()
                });

                self.handle.text(text).expect("send join error");
            }

            Call::Shout(message, message_ref) => {
                tracing::info!("sending shout message: {message}");

                let text = message_text(Message {
                    event: message::Event::Shout,
                    payload: message::Payload::Wrap(message),
                    message_ref: Some(message_ref),
                    ..Default::default()
                });

                self.handle.text(text).expect("send text error");
            }
        }

        Ok(())
    }
}

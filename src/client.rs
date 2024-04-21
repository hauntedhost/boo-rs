use crate::message::{self, message_text, Message};
use async_trait::async_trait;

pub struct Client {
    pub handle: ezsockets::Client<Self>,
}

pub enum Call {
    Join,
    Shout(String),
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        tracing::info!("received message: {text}");
        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        tracing::info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        match call {
            Call::Join => {
                tracing::info!("sending join request");

                let text = message_text(Message {
                    event: message::Event::Join,
                    ..Default::default()
                });

                self.handle.text(text).expect("send join error");
            }
            Call::Shout(message) => {
                tracing::info!("sending shout message: {message}");

                let text = message_text(Message {
                    event: message::Event::Shout,
                    payload: message::Payload::Wrap(message),
                    ..Default::default()
                });

                self.handle.text(text).expect("send text error");
            }
        }

        Ok(())
    }
}

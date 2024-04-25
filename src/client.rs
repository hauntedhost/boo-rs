use crate::message::{self, message_text, Message};
use crate::user::User;
use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

pub struct Client {
    pub handle: ezsockets::Client<Self>,
    pub tx: mpsc::Sender<String>,
    join_ref_counter: AtomicUsize,
    ref_counter: AtomicUsize,
}

impl Client {
    pub fn new(handle: ezsockets::Client<Self>, tx: mpsc::Sender<String>) -> Self {
        Self {
            handle,
            tx,
            join_ref_counter: AtomicUsize::new(1),
            ref_counter: AtomicUsize::new(1),
        }
    }

    fn generate_join_ref(&self) -> usize {
        self.join_ref_counter.fetch_add(1, Ordering::SeqCst)
    }

    fn generate_ref(&self) -> usize {
        self.ref_counter.fetch_add(1, Ordering::SeqCst)
    }
}

pub enum Call {
    Join(User),
    Shout(User, String),
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
            Call::Join(user) => {
                tracing::info!("sending join request for {}", user.username.clone());

                let payload = json!({"user": user});

                let text = message_text(Message {
                    event: message::Event::Join,
                    payload: message::Payload::Raw(payload),
                    join_ref: Some(self.generate_join_ref()),
                    message_ref: self.generate_ref(),
                    ..Default::default()
                });

                self.handle.text(text).expect("send join error");
            }

            Call::Shout(user, message) => {
                tracing::info!(
                    "sending shout message for {}: {message}",
                    user.username.clone()
                );

                let text = message_text(Message {
                    event: message::Event::Shout,
                    payload: message::Payload::Wrap(message),
                    message_ref: self.generate_ref(),
                    ..Default::default()
                });

                self.handle.text(text).expect("send text error");
            }
        }

        Ok(())
    }
}

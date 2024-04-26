/// This module contains the `Client` struct and ezsockets client implementation.
/// It handles internal calls and relays messages to the server.
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
    Join(Join),
    Shout(Shout),
}

pub struct Join {
    pub user: User,
}

pub struct Shout {
    pub user: User,
    pub message: String,
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        log::info!("received message: {text}");

        if let Err(e) = self.tx.send(text).await {
            log::error!("Error sending message to terminal: {e}");
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        log::info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        match call {
            Call::Join(join) => {
                log::info!("sending join request for {}", join.user.username.clone());

                let payload = json!({"user": join.user});

                let text = message_text(Message {
                    event: message::Event::Join,
                    payload: message::Payload::Raw(payload),
                    join_ref: Some(self.generate_join_ref()),
                    message_ref: self.generate_ref(),
                    ..Default::default()
                });

                self.handle.text(text).expect("send join error");
            }

            Call::Shout(shout) => {
                log::info!(
                    "sending shout message for {}: {}",
                    shout.user.username,
                    shout.message
                );

                let text = message_text(Message {
                    event: message::Event::Shout,
                    payload: message::Payload::Wrap(shout.message),
                    message_ref: self.generate_ref(),
                    ..Default::default()
                });

                self.handle.text(text).expect("send text error");
            }
        }

        Ok(())
    }
}

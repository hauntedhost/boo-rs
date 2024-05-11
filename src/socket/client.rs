use crate::socket::request::Request;
use async_trait::async_trait;
use log::{error, info};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
use uuid::Uuid;

/// This module contains the `Client` struct and ezsockets client implementation.
/// It handles internal calls and relays messages to the server.

pub struct Refs {
    join_ref: String,
    message_ref: AtomicUsize,
}

impl Default for Refs {
    fn default() -> Self {
        Self {
            join_ref: Uuid::new_v4().to_string()[..8].to_owned(),
            message_ref: AtomicUsize::new(1),
        }
    }
}

impl Refs {
    pub fn get_join_ref(&self) -> String {
        self.join_ref.clone()
    }

    pub fn get_message_ref(&self) -> usize {
        self.message_ref.load(Ordering::SeqCst)
    }
}

pub struct Client {
    pub handle: ezsockets::Client<Self>,
    pub tx: mpsc::Sender<String>,
    refs: Refs,
}

impl Client {
    pub fn new(handle: ezsockets::Client<Self>, tx: mpsc::Sender<String>) -> Self {
        Self {
            handle,
            tx,
            refs: Refs::default(),
        }
    }

    pub fn next_refs(&self) -> Refs {
        let new_message_ref = self.refs.message_ref.fetch_add(1, Ordering::SeqCst);

        Refs {
            join_ref: self.refs.join_ref.clone(),
            message_ref: AtomicUsize::new(new_message_ref + 1),
        }
    }
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Request;

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        if let Err(e) = self.tx.send(text).await {
            error!("Error sending message to terminal: {e}");
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, request: Request) -> Result<(), ezsockets::Error> {
        let request_payload = request.to_payload(self.next_refs());

        info!("sending request: {request_payload}");

        self.handle
            .text(request_payload)
            .expect("error sending request");

        Ok(())
    }
}

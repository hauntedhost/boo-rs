/// This module contains the `Client` struct and ezsockets client implementation.
/// It handles internal calls and relays messages to the server.
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

use crate::request::{build_request, Refs, Request};

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

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Request;

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

    async fn on_call(&mut self, request: Request) -> Result<(), ezsockets::Error> {
        let request_payload = build_request(
            request,
            Refs {
                join_ref: self.generate_join_ref() as u32,
                message_ref: self.generate_ref() as u32,
            },
        );

        self.handle
            .text(request_payload)
            .expect("error sending request");

        Ok(())
    }
}

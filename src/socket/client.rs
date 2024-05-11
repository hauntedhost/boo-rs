use super::{refs::Refs, response::Response};
use crate::socket::request::Request;
use async_trait::async_trait;
use ezsockets::{client::ClientCloseMode, CloseFrame, Error as SocketError, WSError};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

/// This module contains the `Client` struct and ezsockets client implementation.
/// It handles internal calls and relays messages to the server.

pub struct Client {
    pub handle: ezsockets::Client<Self>,
    pub tx: mpsc::Sender<SocketEvent>,
    refs: Refs,
}

impl Client {
    pub fn new(handle: ezsockets::Client<Self>, tx: mpsc::Sender<SocketEvent>) -> Self {
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

pub enum SocketEvent {
    Close,
    Connect,
    ConnectFail,
    Disconnect,
    Response(Response),
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Request;

    async fn on_text(&mut self, text: String) -> Result<(), SocketError> {
        log::debug!("on_text={text}");

        // Relay message from server to channel
        let response = SocketEvent::Response(Response::new_from_json_string(&text));
        if let Err(e) = self.tx.send(response).await {
            log::error!("error sending message to channel: {e}");
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), SocketError> {
        log::debug!("on_binary={bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, request: Request) -> Result<(), SocketError> {
        log::debug!("on_call={:?}", request);

        let request_payload = request.to_payload(self.next_refs());
        log::info!("sending request: {request_payload}");

        self.handle
            .text(request_payload)
            .expect("error sending request");

        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), SocketError> {
        log::debug!("on_connect");

        if let Err(e) = self.tx.send(SocketEvent::Connect).await {
            log::error!("error sending message to channel: {e}");
        }

        Ok(())
    }

    async fn on_connect_fail(&mut self, _error: WSError) -> Result<ClientCloseMode, SocketError> {
        log::error!("on_connect_fail");

        if let Err(e) = self.tx.send(SocketEvent::ConnectFail).await {
            log::error!("error sending message to channel: {e}");
        }

        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_close(
        &mut self,
        _frame: Option<CloseFrame>,
    ) -> Result<ClientCloseMode, SocketError> {
        log::error!("on_close");

        if let Err(e) = self.tx.send(SocketEvent::Close).await {
            log::error!("error sending message to channel: {e}");
        }

        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_disconnect(&mut self) -> Result<ClientCloseMode, SocketError> {
        log::error!("on_disconnect");

        if let Err(e) = self.tx.send(SocketEvent::Disconnect).await {
            log::error!("error sending message to channel: {e}");
        }

        Ok(ClientCloseMode::Reconnect)
    }
}

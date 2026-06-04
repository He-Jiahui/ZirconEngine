use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;
use zircon_plugin_net_runtime::{
    WebSocketRuntimeBackend, WebSocketRuntimeConnection, WebSocketRuntimeListener,
};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetError, NetEvent, NetWebSocketConnectDescriptor,
    NetWebSocketListenerDescriptor,
};

mod client;
mod connection;
mod frame;
mod handshake;
mod listener;
mod reader;
mod security;
mod stream;

#[derive(Clone, Debug, Default)]
pub struct TungsteniteWebSocketBackend;

pub fn websocket_runtime_backend() -> Arc<dyn WebSocketRuntimeBackend> {
    Arc::new(TungsteniteWebSocketBackend)
}

impl WebSocketRuntimeBackend for TungsteniteWebSocketBackend {
    fn listen_websocket(
        &self,
        runtime: &Runtime,
        descriptor: NetWebSocketListenerDescriptor,
    ) -> Result<Box<dyn WebSocketRuntimeListener>, NetError> {
        listener::listen_websocket(runtime, descriptor)
            .map(|listener| Box::new(listener) as Box<dyn WebSocketRuntimeListener>)
    }

    fn connect_websocket(
        &self,
        runtime: &Runtime,
        connection: NetConnectionId,
        descriptor: NetWebSocketConnectDescriptor,
        events: Arc<Mutex<VecDeque<NetEvent>>>,
    ) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError> {
        client::connect_websocket(runtime, connection, descriptor, events)
    }
}

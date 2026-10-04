//! 把根网络 WS 后端协议连接到 Tungstenite client/listener，实现按 frame 的异步读写。
//! 连接对象和 runtime 由根 manager 持有，后端负责握手策略与异步任务。

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

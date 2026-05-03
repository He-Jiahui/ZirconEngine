use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetEvent, NetWebSocketFrame,
};

pub trait WebSocketRuntimeBackend: Send + Sync + std::fmt::Debug {
    fn listen_websocket(
        &self,
        runtime: &Runtime,
        bind: SocketAddr,
    ) -> Result<Box<dyn WebSocketRuntimeListener>, NetError>;

    fn connect_websocket(
        &self,
        runtime: &Runtime,
        connection: NetConnectionId,
        descriptor: zircon_runtime::core::framework::net::NetWebSocketConnectDescriptor,
        events: Arc<Mutex<VecDeque<NetEvent>>>,
    ) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError>;
}

pub trait WebSocketRuntimeListener: Send + Sync + std::fmt::Debug {
    fn local_endpoint(&self) -> NetEndpoint;

    fn accept_websocket(
        &self,
        runtime: &Runtime,
        connection: NetConnectionId,
        events: Arc<Mutex<VecDeque<NetEvent>>>,
        poll_timeout: Duration,
    ) -> Result<Option<(NetEndpoint, Box<dyn WebSocketRuntimeConnection>)>, NetError>;
}

pub trait WebSocketRuntimeConnection: Send + Sync + std::fmt::Debug {
    fn state(&self) -> NetConnectionState;
    fn set_state(&self, state: NetConnectionState);
    fn send(&self, runtime: &Runtime, frame: NetWebSocketFrame) -> Result<(), NetError>;
    fn drain_frames(&self, max_frames: usize) -> Vec<NetWebSocketFrame>;
}

#[derive(Debug)]
pub(crate) enum ManagedWebSocketConnection {
    Loopback(LoopbackWebSocketConnection),
    Network(Box<dyn WebSocketRuntimeConnection>),
}

#[derive(Debug)]
pub(crate) struct LoopbackWebSocketConnection {
    pub peer: NetConnectionId,
    pub state: NetConnectionState,
    pub inbound: VecDeque<NetWebSocketFrame>,
}

impl ManagedWebSocketConnection {
    pub(crate) fn state(&self) -> NetConnectionState {
        match self {
            Self::Loopback(connection) => connection.state,
            Self::Network(connection) => connection.state(),
        }
    }
}

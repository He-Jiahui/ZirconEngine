use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::runtime::{Builder, Runtime};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetEvent, NetListenerId, NetRouteId,
    NetRuntimeMode, NetSocketId,
};

use crate::http::{ManagedHttpListener, ManagedHttpRoute};
use crate::websocket::{
    ManagedWebSocketConnection, WebSocketRuntimeBackend, WebSocketRuntimeListener,
};
use crate::HttpRuntimeBackend;

#[derive(Debug)]
pub(crate) struct ManagedUdpSocket {
    pub(crate) socket: UdpSocket,
    pub(crate) local_endpoint: NetEndpoint,
}

#[derive(Debug)]
pub(crate) struct ManagedTcpListener {
    pub(crate) listener: TcpListener,
    pub(crate) local_endpoint: NetEndpoint,
}

#[derive(Debug)]
pub(crate) struct ManagedTcpConnection {
    pub(crate) stream: TcpStream,
    pub(crate) _local_endpoint: NetEndpoint,
    pub(crate) _remote_endpoint: NetEndpoint,
    pub(crate) state: NetConnectionState,
}

pub(crate) struct NetRuntimeState {
    pub(crate) runtime: Runtime,
    pub(crate) mode: NetRuntimeMode,
    pub(crate) next_socket_id: AtomicU64,
    pub(crate) next_listener_id: AtomicU64,
    pub(crate) next_connection_id: AtomicU64,
    pub(crate) next_route_id: AtomicU64,
    pub(crate) udp_sockets: Mutex<HashMap<NetSocketId, ManagedUdpSocket>>,
    pub(crate) tcp_listeners: Mutex<HashMap<NetListenerId, ManagedTcpListener>>,
    pub(crate) http_listeners: Mutex<HashMap<NetListenerId, ManagedHttpListener>>,
    pub(crate) websocket_listeners:
        Mutex<HashMap<NetListenerId, Box<dyn WebSocketRuntimeListener>>>,
    pub(crate) tcp_connections: Mutex<HashMap<NetConnectionId, ManagedTcpConnection>>,
    pub(crate) http_routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    pub(crate) websocket_connections: Mutex<HashMap<NetConnectionId, ManagedWebSocketConnection>>,
    pub(crate) http_backend: Mutex<Option<Arc<dyn HttpRuntimeBackend>>>,
    pub(crate) websocket_backend: Mutex<Option<Arc<dyn WebSocketRuntimeBackend>>>,
    pub(crate) events: Arc<Mutex<VecDeque<NetEvent>>>,
}

impl NetRuntimeState {
    pub(crate) fn new(mode: NetRuntimeMode) -> Self {
        Self {
            runtime: Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .thread_name("zircon-net-runtime")
                .build()
                .expect("failed to create net Tokio runtime"),
            mode,
            next_socket_id: AtomicU64::new(0),
            next_listener_id: AtomicU64::new(0),
            next_connection_id: AtomicU64::new(0),
            next_route_id: AtomicU64::new(0),
            udp_sockets: Mutex::new(HashMap::new()),
            tcp_listeners: Mutex::new(HashMap::new()),
            http_listeners: Mutex::new(HashMap::new()),
            websocket_listeners: Mutex::new(HashMap::new()),
            tcp_connections: Mutex::new(HashMap::new()),
            http_routes: Arc::new(Mutex::new(HashMap::new())),
            websocket_connections: Mutex::new(HashMap::new()),
            http_backend: Mutex::new(None),
            websocket_backend: Mutex::new(None),
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub(crate) fn push_event(&self, event: NetEvent) {
        self.events
            .lock()
            .expect("net events mutex poisoned")
            .push_back(event);
    }
}

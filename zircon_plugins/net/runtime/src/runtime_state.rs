use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::runtime::{Builder, Runtime};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetEvent, NetListenerId, NetRouteId,
    NetRuntimeMode, NetSocketId,
};

use crate::http::{ManagedHttpListener, ManagedHttpRoute};
use crate::websocket::{
    ManagedWebSocketConnection, WebSocketRuntimeBackend, WebSocketRuntimeListener,
};
use crate::worker::{NetWorker, NetWorkerShutdownReport};
use crate::HttpRuntimeBackend;

#[derive(Debug)]
pub(crate) struct ManagedUdpSocket {
    pub(crate) local_endpoint: NetEndpoint,
}

#[derive(Debug)]
pub(crate) struct ManagedTcpListener {
    pub(crate) local_endpoint: NetEndpoint,
}

#[derive(Debug)]
pub(crate) struct ManagedTcpConnection {
    pub(crate) state: NetConnectionState,
}

pub(crate) struct NetRuntimeState {
    pub(crate) runtime: Runtime,
    pub(crate) worker: NetWorker,
    pub(crate) mode: NetRuntimeMode,
    pub(crate) next_socket_id: AtomicU64,
    pub(crate) next_listener_id: AtomicU64,
    pub(crate) next_connection_id: Arc<AtomicU64>,
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
        let next_connection_id = Arc::new(AtomicU64::new(0));
        Self {
            runtime: Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .thread_name("zircon-net-runtime")
                .build()
                .expect("failed to create net Tokio runtime"),
            worker: NetWorker::spawn(next_connection_id.clone())
                .expect("failed to create net worker"),
            mode,
            next_socket_id: AtomicU64::new(0),
            next_listener_id: AtomicU64::new(0),
            next_connection_id,
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

    pub(crate) fn next_connection_id(&self) -> NetConnectionId {
        NetConnectionId::new(self.next_connection_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    pub(crate) fn push_event(&self, event: NetEvent) {
        self.events
            .lock()
            .expect("net events mutex poisoned")
            .push_back(event);
    }

    pub(crate) fn poll_worker_ingress(&self, max_events: usize) -> usize {
        let events = self.worker.drain_ingress(max_events);
        let count = events.len();
        if count == 0 {
            return 0;
        }

        let mut queue = self.events.lock().expect("net events mutex poisoned");
        queue.extend(events);
        count
    }

    #[cfg(test)]
    pub(crate) fn shutdown_worker_for_tests(&self) -> NetWorkerShutdownReport {
        self.worker
            .shutdown()
            .expect("net worker shutdown should succeed")
    }
}

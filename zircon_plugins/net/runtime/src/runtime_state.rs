use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::runtime::{Builder, Runtime};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetEvent, NetListenerId, NetRouteId,
    NetRuntimeMode, NetSocketId,
};

use crate::http::{ManagedHttpListener, ManagedHttpRoute};
use crate::poison_recovery::lock_recover;
use crate::websocket::{
    ManagedWebSocketConnection, WebSocketRuntimeBackend, WebSocketRuntimeListener,
};
use crate::worker::NetWorker;
#[cfg(test)]
use crate::worker::NetWorkerShutdownReport;
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
        Mutex<HashMap<NetListenerId, Arc<dyn WebSocketRuntimeListener>>>,
    pub(crate) tcp_connections: Mutex<HashMap<NetConnectionId, ManagedTcpConnection>>,
    pub(crate) http_routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    pub(crate) websocket_connections: Mutex<HashMap<NetConnectionId, ManagedWebSocketConnection>>,
    pub(crate) http_backend: Mutex<Option<Arc<dyn HttpRuntimeBackend>>>,
    pub(crate) websocket_backend: Mutex<Option<Arc<dyn WebSocketRuntimeBackend>>>,
    pub(crate) events: Arc<Mutex<VecDeque<NetEvent>>>,
    pub(crate) outbound_bytes: AtomicU64,
    pub(crate) inbound_bytes: AtomicU64,
    pub(crate) last_observed_latency_ms: AtomicU64,
    pub(crate) latency_observed: AtomicBool,
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
            outbound_bytes: AtomicU64::new(0),
            inbound_bytes: AtomicU64::new(0),
            last_observed_latency_ms: AtomicU64::new(0),
            latency_observed: AtomicBool::new(false),
        }
    }

    pub(crate) fn next_connection_id(&self) -> NetConnectionId {
        NetConnectionId::new(self.next_connection_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    pub(crate) fn push_event(&self, event: NetEvent) {
        lock_recover(&self.events).push_back(event);
    }

    pub(crate) fn record_outbound_bytes(&self, bytes: usize) {
        self.outbound_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_inbound_bytes(&self, bytes: usize) {
        self.inbound_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_latency_ms(&self, latency_ms: u64) {
        self.last_observed_latency_ms
            .store(latency_ms, Ordering::Relaxed);
        self.latency_observed.store(true, Ordering::Relaxed);
    }

    pub(crate) fn diagnostic_counters(&self) -> (u64, u64, Option<u64>) {
        let latency = self
            .latency_observed
            .load(Ordering::Relaxed)
            .then(|| self.last_observed_latency_ms.load(Ordering::Relaxed));
        (
            self.outbound_bytes.load(Ordering::Relaxed),
            self.inbound_bytes.load(Ordering::Relaxed),
            latency,
        )
    }

    pub(crate) fn poll_worker_ingress(&self, max_events: usize) -> usize {
        let events = self.worker.drain_ingress(max_events);
        let count = events.len();
        if count == 0 {
            return 0;
        }

        let mut queue = lock_recover(&self.events);
        queue.extend(events);
        count
    }

    #[cfg(test)]
    pub(crate) fn poison_events_for_test(&self) {
        let events = Arc::clone(&self.events);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&events);
            panic!("poison net events for recovery coverage");
        });
    }

    #[cfg(test)]
    pub(crate) fn shutdown_worker_for_tests(&self) -> NetWorkerShutdownReport {
        self.worker
            .shutdown()
            .expect("net worker shutdown should succeed")
    }

    #[cfg(test)]
    pub(crate) fn shutdown_worker_result_for_tests(
        &self,
    ) -> Result<NetWorkerShutdownReport, zircon_runtime::core::framework::net::NetError> {
        self.worker.shutdown()
    }

    #[cfg(test)]
    pub(crate) fn poison_worker_thread_for_test(&self) {
        self.worker.poison_thread_for_test();
    }

    #[cfg(test)]
    pub(crate) fn fail_next_worker_shutdown_after_submit_for_test(&self) {
        self.worker.fail_next_shutdown_after_submit_for_test();
    }
}

use std::sync::{Mutex, MutexGuard};

use zircon_runtime::core::framework::net::NetError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetSharedState {
    UdpSockets,
    TcpListeners,
    HttpListeners,
    WebSocketListeners,
    TcpConnections,
    HttpRoutes,
    WebSocketConnections,
    HttpBackend,
    WebSocketBackend,
    WorkerThread,
}

impl NetSharedState {
    fn resource(self) -> &'static str {
        match self {
            Self::UdpSockets => "net.udp_sockets",
            Self::TcpListeners => "net.tcp_listeners",
            Self::HttpListeners => "net.http_listeners",
            Self::WebSocketListeners => "net.websocket_listeners",
            Self::TcpConnections => "net.tcp_connections",
            Self::HttpRoutes => "net.http_routes",
            Self::WebSocketConnections => "net.websocket_connections",
            Self::HttpBackend => "net.http_backend",
            Self::WebSocketBackend => "net.websocket_backend",
            Self::WorkerThread => "net.worker_thread",
        }
    }
}

/// Recovers the last valid state when a network worker panic poisons shared state.
pub(crate) fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Preserves the typed failure boundary for manager operations that can report errors.
pub(crate) fn lock_or_error<T>(
    mutex: &Mutex<T>,
    state: NetSharedState,
) -> Result<MutexGuard<'_, T>, NetError> {
    mutex.lock().map_err(|_| NetError::SharedStatePoisoned {
        resource: state.resource().to_string(),
    })
}

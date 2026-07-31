mod connections;
mod diagnostics;
mod http_routes;
mod listeners;
mod tcp;
mod udp;
mod websocket;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetDiagnostics, NetEndpoint, NetError, NetEvent,
    NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetListenerId,
    NetPacket, NetRouteId, NetRuntimeMode, NetSocketId, NetWebSocketConnectDescriptor,
    NetWebSocketFrame, NetWebSocketListenerDescriptor,
};

use crate::poison_recovery::lock_recover;
use crate::runtime_state::NetRuntimeState;
use crate::websocket::WebSocketRuntimeBackend;
use crate::HttpRuntimeBackend;

#[derive(Clone, Debug, Default)]
pub struct NetDriver;

#[derive(Clone)]
pub struct DefaultNetManager {
    pub(in crate::service_types) state: Arc<NetRuntimeState>,
}

pub type NetRuntimeManager = DefaultNetManager;

impl DefaultNetManager {
    pub fn for_mode(mode: NetRuntimeMode) -> Self {
        Self {
            state: Arc::new(NetRuntimeState::new(mode)),
        }
    }

    pub fn with_http_backend(self, backend: Arc<dyn HttpRuntimeBackend>) -> Self {
        *lock_recover(&self.state.http_backend) = Some(backend);
        self
    }

    pub fn with_websocket_backend(self, backend: Arc<dyn WebSocketRuntimeBackend>) -> Self {
        *lock_recover(&self.state.websocket_backend) = Some(backend);
        self
    }

    pub(in crate::service_types) fn next_socket_id(&self) -> NetSocketId {
        NetSocketId::new(self.state.next_socket_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    pub(in crate::service_types) fn next_listener_id(&self) -> NetListenerId {
        NetListenerId::new(self.state.next_listener_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    pub(in crate::service_types) fn next_connection_id(&self) -> NetConnectionId {
        self.state.next_connection_id()
    }

    pub(in crate::service_types) fn next_route_id(&self) -> NetRouteId {
        NetRouteId::new(self.state.next_route_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    #[cfg(test)]
    pub(crate) fn shutdown_worker_for_tests(&self) -> crate::worker::NetWorkerShutdownReport {
        self.state.shutdown_worker_for_tests()
    }

    #[cfg(test)]
    pub(crate) fn worker_is_shutdown_for_tests(&self) -> bool {
        self.state.worker.is_shutdown()
    }

    #[cfg(test)]
    pub(crate) fn shutdown_worker_result_for_tests(
        &self,
    ) -> Result<crate::worker::NetWorkerShutdownReport, NetError> {
        self.state.shutdown_worker_result_for_tests()
    }

    #[cfg(test)]
    pub(crate) fn poison_worker_thread_for_test(&self) {
        self.state.poison_worker_thread_for_test();
    }

    #[cfg(test)]
    pub(crate) fn poison_events_for_test(&self) {
        self.state.poison_events_for_test();
    }

    #[cfg(test)]
    pub(crate) fn poison_udp_sockets_for_test(&self) {
        let state = Arc::clone(&self.state);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&state.udp_sockets);
            panic!("poison net UDP sockets for typed-error coverage");
        });
    }

    #[cfg(test)]
    pub(crate) fn poison_tcp_connections_for_test(&self) {
        let state = Arc::clone(&self.state);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&state.tcp_connections);
            panic!("poison net TCP connections for typed-error coverage");
        });
    }

    #[cfg(test)]
    pub(crate) fn poison_http_listeners_for_test(&self) {
        let state = Arc::clone(&self.state);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&state.http_listeners);
            panic!("poison net HTTP listeners for post-callback coverage");
        });
    }

    #[cfg(test)]
    pub(crate) fn poison_websocket_listeners_for_test(&self) {
        let state = Arc::clone(&self.state);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&state.websocket_listeners);
            panic!("poison net WebSocket listeners for post-callback coverage");
        });
    }

    #[cfg(test)]
    pub(crate) fn poison_websocket_connections_for_test(&self) {
        let state = Arc::clone(&self.state);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&state.websocket_connections);
            panic!("poison net WebSocket connections for post-callback coverage");
        });
    }

    #[cfg(test)]
    pub(crate) fn fail_next_worker_shutdown_after_submit_for_test(&self) {
        self.state.fail_next_worker_shutdown_after_submit_for_test();
    }
}

impl Default for DefaultNetManager {
    fn default() -> Self {
        Self::for_mode(NetRuntimeMode::Client)
    }
}

impl std::fmt::Debug for DefaultNetManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetRuntimeManager")
            .field("mode", &self.state.mode)
            .finish_non_exhaustive()
    }
}

impl zircon_runtime::core::framework::net::NetManager for DefaultNetManager {
    fn backend_name(&self) -> String {
        self.backend_name_impl()
    }

    fn runtime_mode(&self) -> NetRuntimeMode {
        self.state.mode
    }

    fn bind_udp(&self, bind: &NetEndpoint) -> Result<NetSocketId, NetError> {
        self.bind_udp_impl(bind)
    }

    fn local_endpoint(&self, socket: NetSocketId) -> Result<NetEndpoint, NetError> {
        self.local_endpoint_impl(socket)
    }

    fn send_udp(
        &self,
        socket: NetSocketId,
        destination: &NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError> {
        self.send_udp_impl(socket, destination, payload)
    }

    fn poll_udp(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        self.poll_udp_impl(socket, max_packets)
    }

    fn close_socket(&self, socket: NetSocketId) -> Result<(), NetError> {
        self.close_socket_impl(socket)
    }

    fn listen_tcp(&self, bind: &NetEndpoint) -> Result<NetListenerId, NetError> {
        self.listen_tcp_impl(bind)
    }

    fn listener_endpoint(&self, listener: NetListenerId) -> Result<NetEndpoint, NetError> {
        self.listener_endpoint_impl(listener)
    }

    fn close_listener(&self, listener: NetListenerId) -> Result<(), NetError> {
        self.close_listener_impl(listener)
    }

    fn accept_tcp(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError> {
        self.accept_tcp_impl(listener, max_connections)
    }

    fn connect_tcp(&self, remote: &NetEndpoint) -> Result<NetConnectionId, NetError> {
        self.connect_tcp_impl(remote)
    }

    fn connection_state(
        &self,
        connection: NetConnectionId,
    ) -> Result<NetConnectionState, NetError> {
        self.connection_state_impl(connection)
    }

    fn send_tcp(&self, connection: NetConnectionId, payload: &[u8]) -> Result<usize, NetError> {
        self.send_tcp_impl(connection, payload)
    }

    fn poll_tcp(&self, connection: NetConnectionId, max_bytes: usize) -> Result<Vec<u8>, NetError> {
        self.poll_tcp_impl(connection, max_bytes)
    }

    fn close_connection(&self, connection: NetConnectionId) -> Result<(), NetError> {
        self.close_connection_impl(connection)
    }

    fn register_http_route(
        &self,
        route: NetHttpRouteDescriptor,
        response: NetHttpResponseDescriptor,
    ) -> Result<NetRouteId, NetError> {
        self.register_http_route_impl(route, response)
    }

    fn unregister_http_route(&self, route: NetRouteId) -> Result<(), NetError> {
        self.unregister_http_route_impl(route)
    }

    fn listen_http(&self, bind: &NetEndpoint) -> Result<NetListenerId, NetError> {
        self.listen_http_impl(bind)
    }

    fn send_http_request(
        &self,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        self.send_http_request_impl(request)
    }

    fn connect_websocket(
        &self,
        descriptor: NetWebSocketConnectDescriptor,
    ) -> Result<NetConnectionId, NetError> {
        self.connect_websocket_impl(descriptor)
    }

    fn listen_websocket(
        &self,
        descriptor: NetWebSocketListenerDescriptor,
    ) -> Result<NetListenerId, NetError> {
        self.listen_websocket_impl(descriptor)
    }

    fn accept_websocket(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError> {
        self.accept_websocket_impl(listener, max_connections)
    }

    fn open_websocket_loopback(&self) -> Result<(NetConnectionId, NetConnectionId), NetError> {
        self.open_websocket_loopback_impl()
    }

    fn send_websocket_frame(
        &self,
        connection: NetConnectionId,
        frame: NetWebSocketFrame,
    ) -> Result<(), NetError> {
        self.send_websocket_frame_impl(connection, frame)
    }

    fn poll_websocket_frames(
        &self,
        connection: NetConnectionId,
        max_frames: usize,
    ) -> Result<Vec<NetWebSocketFrame>, NetError> {
        self.poll_websocket_frames_impl(connection, max_frames)
    }

    fn drain_events(&self, max_events: usize) -> Vec<NetEvent> {
        self.drain_events_impl(max_events)
    }

    fn diagnostics(&self) -> NetDiagnostics {
        self.diagnostics_impl()
    }
}

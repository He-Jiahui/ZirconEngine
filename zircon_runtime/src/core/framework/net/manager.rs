use super::{
    NetConnectionId, NetConnectionState, NetDiagnostics, NetEndpoint, NetError, NetEvent,
    NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetListenerId,
    NetPacket, NetRouteId, NetRuntimeMode, NetSocketId, NetWebSocketFrame,
};

pub trait NetManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn runtime_mode(&self) -> NetRuntimeMode;
    fn bind_udp(&self, bind: &NetEndpoint) -> Result<NetSocketId, NetError>;
    fn local_endpoint(&self, socket: NetSocketId) -> Result<NetEndpoint, NetError>;
    fn send_udp(
        &self,
        socket: NetSocketId,
        destination: &NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError>;
    fn poll_udp(&self, socket: NetSocketId, max_packets: usize)
        -> Result<Vec<NetPacket>, NetError>;
    fn close_socket(&self, socket: NetSocketId) -> Result<(), NetError>;
    fn listen_tcp(&self, bind: &NetEndpoint) -> Result<NetListenerId, NetError>;
    fn listener_endpoint(&self, listener: NetListenerId) -> Result<NetEndpoint, NetError>;
    fn accept_tcp(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError>;
    fn connect_tcp(&self, remote: &NetEndpoint) -> Result<NetConnectionId, NetError>;
    fn connection_state(&self, connection: NetConnectionId)
        -> Result<NetConnectionState, NetError>;
    fn send_tcp(&self, connection: NetConnectionId, payload: &[u8]) -> Result<usize, NetError>;
    fn poll_tcp(&self, connection: NetConnectionId, max_bytes: usize) -> Result<Vec<u8>, NetError>;
    fn close_connection(&self, connection: NetConnectionId) -> Result<(), NetError>;
    fn register_http_route(
        &self,
        route: NetHttpRouteDescriptor,
        response: NetHttpResponseDescriptor,
    ) -> Result<NetRouteId, NetError>;
    fn unregister_http_route(&self, route: NetRouteId) -> Result<(), NetError>;
    fn send_http_request(
        &self,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError>;
    fn open_websocket_loopback(&self) -> Result<(NetConnectionId, NetConnectionId), NetError>;
    fn send_websocket_frame(
        &self,
        connection: NetConnectionId,
        frame: NetWebSocketFrame,
    ) -> Result<(), NetError>;
    fn poll_websocket_frames(
        &self,
        connection: NetConnectionId,
        max_frames: usize,
    ) -> Result<Vec<NetWebSocketFrame>, NetError>;
    fn drain_events(&self, max_events: usize) -> Vec<NetEvent>;
    fn diagnostics(&self) -> NetDiagnostics;
}

use std::collections::{HashMap, VecDeque};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::runtime::{Builder, Runtime};
use tokio::time::timeout;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetDiagnostics, NetEndpoint, NetError, NetEvent,
    NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetListenerId,
    NetPacket, NetRouteId, NetRuntimeMode, NetSocketId, NetTransportKind, NetWebSocketCloseReason,
    NetWebSocketFrame,
};

const TCP_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

#[derive(Clone, Debug, Default)]
pub struct NetDriver;

#[derive(Debug)]
struct ManagedUdpSocket {
    socket: UdpSocket,
    local_endpoint: NetEndpoint,
}

#[derive(Debug)]
struct ManagedTcpListener {
    listener: TcpListener,
    local_endpoint: NetEndpoint,
}

#[derive(Debug)]
struct ManagedTcpConnection {
    stream: TcpStream,
    _local_endpoint: NetEndpoint,
    _remote_endpoint: NetEndpoint,
    state: NetConnectionState,
}

#[derive(Debug)]
struct ManagedHttpRoute {
    route: NetHttpRouteDescriptor,
    response: NetHttpResponseDescriptor,
}

#[derive(Debug)]
struct ManagedWebSocketConnection {
    peer: NetConnectionId,
    state: NetConnectionState,
    inbound: VecDeque<NetWebSocketFrame>,
}

struct NetRuntimeState {
    runtime: Runtime,
    mode: NetRuntimeMode,
    next_socket_id: AtomicU64,
    next_listener_id: AtomicU64,
    next_connection_id: AtomicU64,
    next_route_id: AtomicU64,
    udp_sockets: Mutex<HashMap<NetSocketId, ManagedUdpSocket>>,
    tcp_listeners: Mutex<HashMap<NetListenerId, ManagedTcpListener>>,
    tcp_connections: Mutex<HashMap<NetConnectionId, ManagedTcpConnection>>,
    http_routes: Mutex<HashMap<NetRouteId, ManagedHttpRoute>>,
    websocket_connections: Mutex<HashMap<NetConnectionId, ManagedWebSocketConnection>>,
    events: Mutex<VecDeque<NetEvent>>,
}

impl NetRuntimeState {
    fn new(mode: NetRuntimeMode) -> Self {
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
            tcp_connections: Mutex::new(HashMap::new()),
            http_routes: Mutex::new(HashMap::new()),
            websocket_connections: Mutex::new(HashMap::new()),
            events: Mutex::new(VecDeque::new()),
        }
    }

    fn push_event(&self, event: NetEvent) {
        self.events
            .lock()
            .expect("net events mutex poisoned")
            .push_back(event);
    }
}

#[derive(Clone)]
pub struct DefaultNetManager {
    state: Arc<NetRuntimeState>,
}

pub type NetRuntimeManager = DefaultNetManager;

impl DefaultNetManager {
    pub fn for_mode(mode: NetRuntimeMode) -> Self {
        Self {
            state: Arc::new(NetRuntimeState::new(mode)),
        }
    }

    fn next_socket_id(&self) -> NetSocketId {
        NetSocketId::new(self.state.next_socket_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn next_listener_id(&self) -> NetListenerId {
        NetListenerId::new(self.state.next_listener_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn next_connection_id(&self) -> NetConnectionId {
        NetConnectionId::new(
            self.state
                .next_connection_id
                .fetch_add(1, Ordering::Relaxed)
                + 1,
        )
    }

    fn next_route_id(&self) -> NetRouteId {
        NetRouteId::new(self.state.next_route_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn endpoint_from_addr(addr: SocketAddr) -> NetEndpoint {
        NetEndpoint::from(addr)
    }

    fn path_from_http_url(url: &str) -> String {
        if let Some((_, rest)) = url.split_once("://") {
            match rest.find('/') {
                Some(index) => rest[index..].to_string(),
                None => "/".to_string(),
            }
        } else if url.starts_with('/') {
            url.to_string()
        } else {
            format!("/{url}")
        }
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
        "tokio-net".to_string()
    }

    fn runtime_mode(&self) -> NetRuntimeMode {
        self.state.mode
    }

    fn bind_udp(&self, bind: &NetEndpoint) -> Result<NetSocketId, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let socket = self
            .state
            .runtime
            .block_on(UdpSocket::bind(bind_addr))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = socket
            .local_addr()
            .map(Self::endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let socket_id = self.next_socket_id();
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .insert(
                socket_id,
                ManagedUdpSocket {
                    socket,
                    local_endpoint: local_endpoint.clone(),
                },
            );
        self.state.push_event(NetEvent::UdpSocketBound {
            socket: socket_id,
            endpoint: local_endpoint,
        });
        Ok(socket_id)
    }

    fn local_endpoint(&self, socket: NetSocketId) -> Result<NetEndpoint, NetError> {
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .get(&socket)
            .map(|entry| entry.local_endpoint.clone())
            .ok_or(NetError::UnknownSocket { socket })
    }

    fn send_udp(
        &self,
        socket: NetSocketId,
        destination: &NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError> {
        let destination = destination.to_socket_addr()?;
        let sockets = self
            .state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned");
        let entry = sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;
        entry
            .socket
            .try_send_to(payload, destination)
            .map_err(|error| NetError::Io(error.to_string()))
    }

    fn poll_udp(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        if max_packets == 0 {
            return Ok(Vec::new());
        }

        let sockets = self
            .state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned");
        let entry = sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;

        let mut packets = Vec::new();
        let mut buffer = vec![0_u8; u16::MAX as usize];
        while packets.len() < max_packets {
            match entry.socket.try_recv_from(&mut buffer) {
                Ok((received, source)) => packets.push(NetPacket {
                    source: Self::endpoint_from_addr(source),
                    payload: buffer[..received].to_vec(),
                }),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(NetError::Io(error.to_string())),
            }
        }

        Ok(packets)
    }

    fn close_socket(&self, socket: NetSocketId) -> Result<(), NetError> {
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .remove(&socket)
            .map(|_| ())
            .ok_or(NetError::UnknownSocket { socket })
    }

    fn listen_tcp(&self, bind: &NetEndpoint) -> Result<NetListenerId, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let listener = self
            .state
            .runtime
            .block_on(TcpListener::bind(bind_addr))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = listener
            .local_addr()
            .map(Self::endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let listener_id = self.next_listener_id();
        self.state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .insert(
                listener_id,
                ManagedTcpListener {
                    listener,
                    local_endpoint: local_endpoint.clone(),
                },
            );
        self.state.push_event(NetEvent::ListenerStarted {
            listener: listener_id,
            transport: NetTransportKind::Tcp,
            endpoint: local_endpoint,
        });
        Ok(listener_id)
    }

    fn listener_endpoint(&self, listener: NetListenerId) -> Result<NetEndpoint, NetError> {
        self.state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .get(&listener)
            .map(|entry| entry.local_endpoint.clone())
            .ok_or(NetError::UnknownListener { listener })
    }

    fn accept_tcp(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError> {
        if max_connections == 0 {
            return Ok(Vec::new());
        }

        let listeners = self
            .state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned");
        let entry = listeners
            .get(&listener)
            .ok_or(NetError::UnknownListener { listener })?;

        let mut accepted = Vec::new();
        while accepted.len() < max_connections {
            match self
                .state
                .runtime
                .block_on(timeout(TCP_ACCEPT_POLL_TIMEOUT, entry.listener.accept()))
            {
                Ok(Ok((stream, remote_addr))) => {
                    let local_endpoint = stream
                        .local_addr()
                        .map(Self::endpoint_from_addr)
                        .map_err(|error| NetError::Io(error.to_string()))?;
                    let remote_endpoint = Self::endpoint_from_addr(remote_addr);
                    let connection = self.next_connection_id();
                    self.state
                        .tcp_connections
                        .lock()
                        .expect("net TCP connections mutex poisoned")
                        .insert(
                            connection,
                            ManagedTcpConnection {
                                stream,
                                _local_endpoint: local_endpoint,
                                _remote_endpoint: remote_endpoint.clone(),
                                state: NetConnectionState::Open,
                            },
                        );
                    self.state.push_event(NetEvent::ConnectionAccepted {
                        listener,
                        connection,
                        remote: remote_endpoint,
                    });
                    self.state.push_event(NetEvent::ConnectionStateChanged {
                        connection,
                        transport: NetTransportKind::Tcp,
                        state: NetConnectionState::Open,
                    });
                    accepted.push(connection);
                }
                Ok(Err(error)) => return Err(NetError::Io(error.to_string())),
                Err(_) => break,
            }
        }

        Ok(accepted)
    }

    fn connect_tcp(&self, remote: &NetEndpoint) -> Result<NetConnectionId, NetError> {
        let remote_addr = remote.to_socket_addr()?;
        let stream = self
            .state
            .runtime
            .block_on(TcpStream::connect(remote_addr))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = stream
            .local_addr()
            .map(Self::endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let remote_endpoint = stream
            .peer_addr()
            .map(Self::endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let connection = self.next_connection_id();
        self.state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .insert(
                connection,
                ManagedTcpConnection {
                    stream,
                    _local_endpoint: local_endpoint,
                    _remote_endpoint: remote_endpoint,
                    state: NetConnectionState::Open,
                },
            );
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::Tcp,
            state: NetConnectionState::Open,
        });
        Ok(connection)
    }

    fn connection_state(
        &self,
        connection: NetConnectionId,
    ) -> Result<NetConnectionState, NetError> {
        if let Some(state) = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .get(&connection)
            .map(|entry| entry.state)
        {
            return Ok(state);
        }

        self.state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned")
            .get(&connection)
            .map(|entry| entry.state)
            .ok_or(NetError::UnknownConnection { connection })
    }

    fn send_tcp(&self, connection: NetConnectionId, payload: &[u8]) -> Result<usize, NetError> {
        let connections = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned");
        let entry = connections
            .get(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        loop {
            match entry.stream.try_write(payload) {
                Ok(written) => return Ok(written),
                Err(error) if error.kind() == ErrorKind::WouldBlock => self
                    .state
                    .runtime
                    .block_on(entry.stream.writable())
                    .map_err(|error| NetError::Io(error.to_string()))?,
                Err(error) => return Err(NetError::Io(error.to_string())),
            }
        }
    }

    fn poll_tcp(&self, connection: NetConnectionId, max_bytes: usize) -> Result<Vec<u8>, NetError> {
        if max_bytes == 0 {
            return Ok(Vec::new());
        }

        let mut connections = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned");
        let entry = connections
            .get_mut(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        let mut payload = vec![0_u8; max_bytes];
        match entry.stream.try_read(&mut payload) {
            Ok(0) => {
                entry.state = NetConnectionState::Closed;
                self.state
                    .push_event(NetEvent::ConnectionClosed { connection });
                Ok(Vec::new())
            }
            Ok(received) => {
                payload.truncate(received);
                Ok(payload)
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(Vec::new()),
            Err(error) => {
                entry.state = NetConnectionState::Failed;
                Err(NetError::Io(error.to_string()))
            }
        }
    }

    fn close_connection(&self, connection: NetConnectionId) -> Result<(), NetError> {
        if let Some(entry) = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .remove(&connection)
        {
            let _ = entry.stream;
            self.state
                .push_event(NetEvent::ConnectionClosed { connection });
            return Ok(());
        }

        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        let peer = websockets
            .remove(&connection)
            .map(|entry| entry.peer)
            .ok_or(NetError::UnknownConnection { connection })?;
        if let Some(peer_entry) = websockets.get_mut(&peer) {
            peer_entry.state = NetConnectionState::Closed;
            peer_entry.inbound.push_back(NetWebSocketFrame::Close(
                NetWebSocketCloseReason::normal("peer closed"),
            ));
        }
        self.state
            .push_event(NetEvent::ConnectionClosed { connection });
        Ok(())
    }

    fn register_http_route(
        &self,
        route: NetHttpRouteDescriptor,
        response: NetHttpResponseDescriptor,
    ) -> Result<NetRouteId, NetError> {
        let route_id = self.next_route_id();
        self.state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned")
            .insert(
                route_id,
                ManagedHttpRoute {
                    response,
                    route: route.clone(),
                },
            );
        self.state.push_event(NetEvent::HttpRouteRegistered {
            route: route_id,
            path: route.path,
            methods: route.methods,
        });
        Ok(route_id)
    }

    fn unregister_http_route(&self, route: NetRouteId) -> Result<(), NetError> {
        self.state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned")
            .remove(&route)
            .map(|_| ())
            .ok_or(NetError::UnknownRoute { route })
    }

    fn send_http_request(
        &self,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        let path = Self::path_from_http_url(&request.url);
        let routes = self
            .state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned");
        routes
            .values()
            .find(|entry| entry.route.path == path && entry.route.methods.contains(&request.method))
            .map(|entry| entry.response.clone().for_request(request.request))
            .ok_or_else(|| NetError::RouteNotFound {
                method: format!("{:?}", request.method),
                path,
            })
    }

    fn open_websocket_loopback(&self) -> Result<(NetConnectionId, NetConnectionId), NetError> {
        let client = self.next_connection_id();
        let server = self.next_connection_id();
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        websockets.insert(
            client,
            ManagedWebSocketConnection {
                peer: server,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            },
        );
        websockets.insert(
            server,
            ManagedWebSocketConnection {
                peer: client,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            },
        );
        self.state
            .push_event(NetEvent::WebSocketPairOpened { client, server });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection: client,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection: server,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok((client, server))
    }

    fn send_websocket_frame(
        &self,
        connection: NetConnectionId,
        frame: NetWebSocketFrame,
    ) -> Result<(), NetError> {
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        let peer = websockets
            .get(&connection)
            .map(|entry| entry.peer)
            .ok_or(NetError::UnknownConnection { connection })?;
        if let NetWebSocketFrame::Close(_) = frame {
            if let Some(entry) = websockets.get_mut(&connection) {
                entry.state = NetConnectionState::Closed;
            }
        }
        let peer_entry = websockets
            .get_mut(&peer)
            .ok_or(NetError::UnknownConnection { connection: peer })?;
        peer_entry.inbound.push_back(frame);
        let queued_frames = peer_entry.inbound.len();
        self.state.push_event(NetEvent::WebSocketFrameQueued {
            connection: peer,
            queued_frames,
        });
        Ok(())
    }

    fn poll_websocket_frames(
        &self,
        connection: NetConnectionId,
        max_frames: usize,
    ) -> Result<Vec<NetWebSocketFrame>, NetError> {
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        let entry = websockets
            .get_mut(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        let mut frames = Vec::new();
        while frames.len() < max_frames {
            match entry.inbound.pop_front() {
                Some(NetWebSocketFrame::Close(reason)) => {
                    entry.state = NetConnectionState::Closed;
                    frames.push(NetWebSocketFrame::Close(reason));
                }
                Some(frame) => frames.push(frame),
                None => break,
            }
        }
        Ok(frames)
    }

    fn drain_events(&self, max_events: usize) -> Vec<NetEvent> {
        let mut events = self.events().lock().expect("net events mutex poisoned");
        let mut drained = Vec::new();
        while drained.len() < max_events {
            match events.pop_front() {
                Some(event) => drained.push(event),
                None => break,
            }
        }
        drained
    }

    fn diagnostics(&self) -> NetDiagnostics {
        NetDiagnostics {
            backend_name: self.backend_name(),
            mode: self.state.mode,
            open_udp_sockets: self
                .state
                .udp_sockets
                .lock()
                .expect("net UDP sockets mutex poisoned")
                .len(),
            open_tcp_listeners: self
                .state
                .tcp_listeners
                .lock()
                .expect("net TCP listeners mutex poisoned")
                .len(),
            open_tcp_connections: self
                .state
                .tcp_connections
                .lock()
                .expect("net TCP connections mutex poisoned")
                .len(),
            open_http_routes: self
                .state
                .http_routes
                .lock()
                .expect("net HTTP routes mutex poisoned")
                .len(),
            open_websocket_connections: self
                .state
                .websocket_connections
                .lock()
                .expect("net WebSocket connections mutex poisoned")
                .len(),
            queued_events: self
                .state
                .events
                .lock()
                .expect("net events mutex poisoned")
                .len(),
        }
    }
}

impl DefaultNetManager {
    fn events(&self) -> &Mutex<VecDeque<NetEvent>> {
        &self.state.events
    }
}

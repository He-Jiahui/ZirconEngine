use std::collections::{HashMap, VecDeque};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::runtime::{Builder, Runtime};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetDiagnostics, NetEndpoint, NetError, NetEvent,
    NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetListenerId,
    NetPacket, NetRouteId, NetRuntimeMode, NetSocketId, NetTransportKind, NetWebSocketCloseReason,
    NetWebSocketConnectDescriptor, NetWebSocketFrame,
};

use crate::backend::{
    read_websocket_frames, LoopbackWebSocketConnection, ManagedWebSocketConnection,
    ManagedWebSocketListener, NetworkWebSocketConnection,
};
use crate::http_backend::{ManagedHttpListener, ManagedHttpRoute};

const TCP_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);
const WEBSOCKET_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

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

struct NetRuntimeState {
    runtime: Runtime,
    mode: NetRuntimeMode,
    next_socket_id: AtomicU64,
    next_listener_id: AtomicU64,
    next_connection_id: AtomicU64,
    next_route_id: AtomicU64,
    udp_sockets: Mutex<HashMap<NetSocketId, ManagedUdpSocket>>,
    tcp_listeners: Mutex<HashMap<NetListenerId, ManagedTcpListener>>,
    http_listeners: Mutex<HashMap<NetListenerId, ManagedHttpListener>>,
    websocket_listeners: Mutex<HashMap<NetListenerId, ManagedWebSocketListener>>,
    tcp_connections: Mutex<HashMap<NetConnectionId, ManagedTcpConnection>>,
    http_routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    websocket_connections: Mutex<HashMap<NetConnectionId, ManagedWebSocketConnection>>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
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
            http_listeners: Mutex::new(HashMap::new()),
            websocket_listeners: Mutex::new(HashMap::new()),
            tcp_connections: Mutex::new(HashMap::new()),
            http_routes: Arc::new(Mutex::new(HashMap::new())),
            websocket_connections: Mutex::new(HashMap::new()),
            events: Arc::new(Mutex::new(VecDeque::new())),
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
        if let Some(endpoint) = self
            .state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .get(&listener)
            .map(|entry| entry.local_endpoint.clone())
        {
            return Ok(endpoint);
        }

        if let Some(endpoint) = self
            .state
            .http_listeners
            .lock()
            .expect("net HTTP listeners mutex poisoned")
            .get(&listener)
            .map(|entry| entry.local_endpoint.clone())
        {
            return Ok(endpoint);
        }

        self.state
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned")
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
                .block_on(async { timeout(TCP_ACCEPT_POLL_TIMEOUT, entry.listener.accept()).await })
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
            .map(|entry| entry.state())
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
                    .block_on(async { entry.stream.writable().await })
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
        let entry = websockets
            .remove(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        match entry {
            ManagedWebSocketConnection::Loopback(entry) => {
                if let Some(ManagedWebSocketConnection::Loopback(peer_entry)) =
                    websockets.get_mut(&entry.peer)
                {
                    peer_entry.state = NetConnectionState::Closed;
                    peer_entry.inbound.push_back(NetWebSocketFrame::Close(
                        NetWebSocketCloseReason::normal("peer closed"),
                    ));
                }
            }
            ManagedWebSocketConnection::Network(entry) => {
                entry.set_state(NetConnectionState::Closed);
            }
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

    fn listen_http(&self, bind: &NetEndpoint) -> Result<NetListenerId, NetError> {
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
            .http_listeners
            .lock()
            .expect("net HTTP listeners mutex poisoned")
            .insert(
                listener_id,
                ManagedHttpListener {
                    local_endpoint: local_endpoint.clone(),
                },
            );
        self.state
            .runtime
            .spawn(crate::http_backend::serve_http_listener(
                listener,
                self.state.http_routes.clone(),
            ));
        self.state.push_event(NetEvent::ListenerStarted {
            listener: listener_id,
            transport: NetTransportKind::Http,
            endpoint: local_endpoint,
        });
        Ok(listener_id)
    }

    fn send_http_request(
        &self,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        let path = crate::http_backend::path_from_http_url(&request.url);
        let routes = self
            .state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned");
        if !crate::http_backend::url_has_explicit_port(&request.url) {
            if let Some(response) = routes
                .values()
                .find(|entry| {
                    entry.route.path == path && entry.route.methods.contains(&request.method)
                })
                .map(|entry| entry.response.clone().for_request(request.request))
            {
                return Ok(response);
            }
        }
        drop(routes);
        self.state
            .runtime
            .block_on(crate::http_backend::send_http_request(request))
    }

    fn connect_websocket(
        &self,
        descriptor: NetWebSocketConnectDescriptor,
    ) -> Result<NetConnectionId, NetError> {
        let connection = self.next_connection_id();
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Connecting,
        });
        let mut request = descriptor
            .url
            .as_str()
            .into_client_request()
            .map_err(|error| NetError::Io(error.to_string()))?;
        for (name, value) in &descriptor.headers {
            let name = tokio_tungstenite::tungstenite::http::header::HeaderName::from_bytes(
                name.as_bytes(),
            )
            .map_err(|error| NetError::Io(error.to_string()))?;
            let value = tokio_tungstenite::tungstenite::http::HeaderValue::from_str(value)
                .map_err(|error| NetError::Io(error.to_string()))?;
            request.headers_mut().insert(name, value);
        }
        if !descriptor.protocols.is_empty() {
            let protocols = descriptor.protocols.join(", ");
            let value = tokio_tungstenite::tungstenite::http::HeaderValue::from_str(&protocols)
                .map_err(|error| NetError::Io(error.to_string()))?;
            request.headers_mut().insert(
                tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL,
                value,
            );
        }
        let timeout_duration = Duration::from_millis(descriptor.timeout_ms);
        let (stream, _) = self
            .state
            .runtime
            .block_on(async {
                timeout(timeout_duration, tokio_tungstenite::connect_async(request)).await
            })
            .map_err(|_| NetError::Io("websocket connect timed out".to_string()))?
            .map_err(|error| NetError::Io(error.to_string()))?;
        let (sink, stream) = stream.split();
        let (network, read_half) = NetworkWebSocketConnection::client(sink, stream);
        let state = network.state.clone();
        let inbound = network.inbound.clone();
        self.state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned")
            .insert(connection, ManagedWebSocketConnection::Network(network));
        let events = self.state.events.clone();
        self.state.runtime.spawn(read_websocket_frames(
            connection, read_half, state, inbound, events,
        ));
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok(connection)
    }

    fn listen_websocket(&self, bind: &NetEndpoint) -> Result<NetListenerId, NetError> {
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
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned")
            .insert(
                listener_id,
                ManagedWebSocketListener {
                    listener,
                    local_endpoint: local_endpoint.clone(),
                },
            );
        self.state.push_event(NetEvent::ListenerStarted {
            listener: listener_id,
            transport: NetTransportKind::WebSocket,
            endpoint: local_endpoint,
        });
        Ok(listener_id)
    }

    fn accept_websocket(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError> {
        if max_connections == 0 {
            return Ok(Vec::new());
        }
        let listeners = self
            .state
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned");
        let listener_entry = listeners
            .get(&listener)
            .ok_or(NetError::UnknownListener { listener })?;
        let mut accepted = Vec::new();
        while accepted.len() < max_connections {
            let accept_result = self.state.runtime.block_on(async {
                timeout(
                    WEBSOCKET_ACCEPT_POLL_TIMEOUT,
                    listener_entry.listener.accept(),
                )
                .await
            });
            let (stream, remote_addr) = match accept_result {
                Ok(Ok(accepted)) => accepted,
                Ok(Err(error)) => return Err(NetError::Io(error.to_string())),
                Err(_) => break,
            };
            let websocket = self
                .state
                .runtime
                .block_on(tokio_tungstenite::accept_async(stream))
                .map_err(|error| NetError::Io(error.to_string()))?;
            let connection = self.next_connection_id();
            let (sink, stream) = websocket.split();
            let (network, read_half) = NetworkWebSocketConnection::server(sink, stream);
            let state = network.state.clone();
            let inbound = network.inbound.clone();
            self.state
                .websocket_connections
                .lock()
                .expect("net WebSocket connections mutex poisoned")
                .insert(connection, ManagedWebSocketConnection::Network(network));
            let events = self.state.events.clone();
            self.state.runtime.spawn(read_websocket_frames(
                connection, read_half, state, inbound, events,
            ));
            self.state.push_event(NetEvent::ConnectionAccepted {
                listener,
                connection,
                remote: Self::endpoint_from_addr(remote_addr),
            });
            self.state.push_event(NetEvent::ConnectionStateChanged {
                connection,
                transport: NetTransportKind::WebSocket,
                state: NetConnectionState::Open,
            });
            accepted.push(connection);
        }
        Ok(accepted)
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
            ManagedWebSocketConnection::Loopback(LoopbackWebSocketConnection {
                peer: server,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            }),
        );
        websockets.insert(
            server,
            ManagedWebSocketConnection::Loopback(LoopbackWebSocketConnection {
                peer: client,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            }),
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
        if let Some(ManagedWebSocketConnection::Network(entry)) = websockets.get(&connection) {
            return self.state.runtime.block_on(entry.send(frame));
        }
        let peer = websockets
            .get(&connection)
            .and_then(|entry| match entry {
                ManagedWebSocketConnection::Loopback(entry) => Some(entry.peer),
                ManagedWebSocketConnection::Network(_) => None,
            })
            .ok_or(NetError::UnknownConnection { connection })?;
        if let NetWebSocketFrame::Close(_) = frame {
            if let Some(ManagedWebSocketConnection::Loopback(entry)) =
                websockets.get_mut(&connection)
            {
                entry.state = NetConnectionState::Closed;
            }
        }
        let peer_entry = websockets
            .get_mut(&peer)
            .and_then(|entry| match entry {
                ManagedWebSocketConnection::Loopback(entry) => Some(entry),
                ManagedWebSocketConnection::Network(_) => None,
            })
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
        match entry {
            ManagedWebSocketConnection::Loopback(entry) => {
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
            ManagedWebSocketConnection::Network(entry) => Ok(entry.drain_frames(max_frames)),
        }
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
    fn events(&self) -> &Arc<Mutex<VecDeque<NetEvent>>> {
        &self.state.events
    }
}

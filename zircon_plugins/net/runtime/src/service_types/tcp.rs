use std::io::ErrorKind;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetEvent, NetListenerId,
    NetTransportKind,
};

use crate::runtime_state::{ManagedTcpConnection, ManagedTcpListener};

use super::DefaultNetManager;

const TCP_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

impl DefaultNetManager {
    pub(in crate::service_types) fn listen_tcp_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetListenerId, NetError> {
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

    pub(in crate::service_types) fn accept_tcp_impl(
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
                        transport: NetTransportKind::Tcp,
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

    pub(in crate::service_types) fn connect_tcp_impl(
        &self,
        remote: &NetEndpoint,
    ) -> Result<NetConnectionId, NetError> {
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

    pub(in crate::service_types) fn send_tcp_impl(
        &self,
        connection: NetConnectionId,
        payload: &[u8],
    ) -> Result<usize, NetError> {
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

    pub(in crate::service_types) fn poll_tcp_impl(
        &self,
        connection: NetConnectionId,
        max_bytes: usize,
    ) -> Result<Vec<u8>, NetError> {
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
                self.state.push_event(NetEvent::ConnectionClosed {
                    connection,
                    transport: NetTransportKind::Tcp,
                });
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
}

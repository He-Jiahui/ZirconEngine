mod dispatch;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::runtime::{Builder, Runtime};
use tokio::time::timeout;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetEvent, NetListenerId, NetPacket,
    NetSocketId, NetTransportKind,
};

use crate::transport::TransportStateMachine;

use super::egress::{AcceptedTcpConnection, TcpPollResult};
use super::ingress::NetIngress;
use super::shutdown::NetWorkerShutdownReport;

pub(super) use dispatch::run_worker;

const TCP_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

#[derive(Debug)]
struct WorkerUdpSocket {
    socket: UdpSocket,
}

#[derive(Debug)]
struct WorkerTcpListener {
    listener: TcpListener,
}

#[derive(Debug)]
struct WorkerTcpConnection {
    stream: TcpStream,
    state_machine: TransportStateMachine,
}

struct WorkerCore {
    runtime: Runtime,
    ingress: mpsc::SyncSender<NetIngress>,
    next_connection_id: Arc<AtomicU64>,
    udp_sockets: HashMap<NetSocketId, WorkerUdpSocket>,
    tcp_listeners: HashMap<NetListenerId, WorkerTcpListener>,
    tcp_connections: HashMap<NetConnectionId, WorkerTcpConnection>,
}

impl WorkerCore {
    fn new(
        ingress: mpsc::SyncSender<NetIngress>,
        next_connection_id: Arc<AtomicU64>,
    ) -> Result<Self, NetError> {
        Ok(Self {
            runtime: Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .thread_name("zircon-net-worker-runtime")
                .build()
                .map_err(|error| NetError::Io(error.to_string()))?,
            ingress,
            next_connection_id,
            udp_sockets: HashMap::new(),
            tcp_listeners: HashMap::new(),
            tcp_connections: HashMap::new(),
        })
    }

    fn bind_udp(
        &mut self,
        socket: NetSocketId,
        bind: NetEndpoint,
    ) -> Result<NetEndpoint, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let udp_socket = self
            .runtime
            .block_on(UdpSocket::bind(bind_addr))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = udp_socket
            .local_addr()
            .map(endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        self.udp_sockets
            .insert(socket, WorkerUdpSocket { socket: udp_socket });
        self.push_event(NetEvent::UdpSocketBound {
            socket,
            endpoint: local_endpoint.clone(),
        });
        Ok(local_endpoint)
    }

    fn send_udp(
        &self,
        socket: NetSocketId,
        destination: NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError> {
        let destination = destination.to_socket_addr()?;
        let entry = self
            .udp_sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;
        self.runtime
            .block_on(entry.socket.send_to(payload, destination))
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

        let entry = self
            .udp_sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;
        let mut packets = Vec::new();
        let mut buffer = vec![0_u8; u16::MAX as usize];
        while packets.len() < max_packets {
            match entry.socket.try_recv_from(&mut buffer) {
                Ok((received, source)) => packets.push(NetPacket {
                    source: endpoint_from_addr(source),
                    payload: buffer[..received].to_vec(),
                }),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(NetError::Io(error.to_string())),
            }
        }
        Ok(packets)
    }

    fn close_udp(&mut self, socket: NetSocketId) -> Result<(), NetError> {
        if self.udp_sockets.remove(&socket).is_none() {
            return Err(NetError::UnknownSocket { socket });
        }
        self.push_event(NetEvent::UdpSocketClosed { socket });
        Ok(())
    }

    fn listen_tcp(
        &mut self,
        listener: NetListenerId,
        bind: NetEndpoint,
    ) -> Result<NetEndpoint, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let tcp_listener = self
            .runtime
            .block_on(TcpListener::bind(bind_addr))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = tcp_listener
            .local_addr()
            .map(endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        self.tcp_listeners.insert(
            listener,
            WorkerTcpListener {
                listener: tcp_listener,
            },
        );
        self.push_event(NetEvent::ListenerStarted {
            listener,
            transport: NetTransportKind::Tcp,
            endpoint: local_endpoint.clone(),
        });
        Ok(local_endpoint)
    }

    fn accept_tcp(
        &mut self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<AcceptedTcpConnection>, NetError> {
        if max_connections == 0 {
            return Ok(Vec::new());
        }

        let mut accepted = Vec::new();
        while accepted.len() < max_connections {
            let accept_result = {
                let entry = self
                    .tcp_listeners
                    .get(&listener)
                    .ok_or(NetError::UnknownListener { listener })?;
                self.runtime.block_on(async {
                    timeout(TCP_ACCEPT_POLL_TIMEOUT, entry.listener.accept()).await
                })
            };
            match accept_result {
                Ok(Ok((stream, remote_addr))) => {
                    let remote_endpoint = endpoint_from_addr(remote_addr);
                    let connection = self.next_connection_id();
                    let mut state_machine = TransportStateMachine::new(
                        connection,
                        NetTransportKind::Tcp,
                        NetConnectionState::Connecting,
                    );
                    let open_event = state_machine.transition(NetConnectionState::Open);
                    self.tcp_connections.insert(
                        connection,
                        WorkerTcpConnection {
                            stream,
                            state_machine,
                        },
                    );
                    self.push_event(NetEvent::ConnectionAccepted {
                        listener,
                        connection,
                        transport: NetTransportKind::Tcp,
                        remote: remote_endpoint,
                    });
                    if let Some(event) = open_event {
                        self.push_event(event);
                    }
                    accepted.push(AcceptedTcpConnection { connection });
                }
                Ok(Err(error)) => return Err(NetError::Io(error.to_string())),
                Err(_) => break,
            }
        }
        Ok(accepted)
    }

    fn close_tcp_listener(&mut self, listener: NetListenerId) -> Result<(), NetError> {
        if self.tcp_listeners.remove(&listener).is_none() {
            return Err(NetError::UnknownListener { listener });
        }
        self.push_event(NetEvent::ListenerClosed {
            listener,
            transport: NetTransportKind::Tcp,
        });
        Ok(())
    }

    fn connect_tcp(
        &mut self,
        connection: NetConnectionId,
        remote: NetEndpoint,
    ) -> Result<(), NetError> {
        let remote_addr = remote.to_socket_addr()?;
        let mut state_machine = TransportStateMachine::new(
            connection,
            NetTransportKind::Tcp,
            NetConnectionState::Connecting,
        );
        self.push_event(state_machine.current_event());
        let stream = match self.runtime.block_on(TcpStream::connect(remote_addr)) {
            Ok(stream) => stream,
            Err(error) => {
                if let Some(event) = state_machine.transition(NetConnectionState::Failed) {
                    self.push_event(event);
                }
                return Err(NetError::Io(error.to_string()));
            }
        };
        let open_event = state_machine.transition(NetConnectionState::Open);
        self.tcp_connections.insert(
            connection,
            WorkerTcpConnection {
                stream,
                state_machine,
            },
        );
        if let Some(event) = open_event {
            self.push_event(event);
        }
        Ok(())
    }

    fn send_tcp(&self, connection: NetConnectionId, payload: &[u8]) -> Result<usize, NetError> {
        let entry = self
            .tcp_connections
            .get(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        loop {
            match entry.stream.try_write(payload) {
                Ok(written) => return Ok(written),
                Err(error) if error.kind() == ErrorKind::WouldBlock => self
                    .runtime
                    .block_on(async { entry.stream.writable().await })
                    .map_err(|error| NetError::Io(error.to_string()))?,
                Err(error) => return Err(NetError::Io(error.to_string())),
            }
        }
    }

    fn poll_tcp(
        &mut self,
        connection: NetConnectionId,
        max_bytes: usize,
    ) -> Result<TcpPollResult, NetError> {
        if max_bytes == 0 {
            return Ok(TcpPollResult {
                payload: Vec::new(),
                state: self.connection_state(connection)?,
            });
        }

        let mut payload = vec![0_u8; max_bytes];
        let mut state_event = None;
        let result = {
            let entry = self
                .tcp_connections
                .get_mut(&connection)
                .ok_or(NetError::UnknownConnection { connection })?;
            match entry.stream.try_read(&mut payload) {
                Ok(0) => {
                    state_event = entry.state_machine.transition(NetConnectionState::Closed);
                    Ok(TcpPollResult {
                        payload: Vec::new(),
                        state: NetConnectionState::Closed,
                    })
                }
                Ok(received) => {
                    payload.truncate(received);
                    Ok(TcpPollResult {
                        payload,
                        state: entry.state_machine.state(),
                    })
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(TcpPollResult {
                    payload: Vec::new(),
                    state: entry.state_machine.state(),
                }),
                Err(error) => {
                    state_event = entry.state_machine.transition(NetConnectionState::Failed);
                    Err(NetError::Io(error.to_string()))
                }
            }
        };

        if let Some(event) = state_event {
            self.push_event(event);
        }
        let result = result?;
        if result.state == NetConnectionState::Closed {
            self.push_event(NetEvent::ConnectionClosed {
                connection,
                transport: NetTransportKind::Tcp,
            });
        }

        Ok(result)
    }

    fn close_tcp(&mut self, connection: NetConnectionId) -> Result<(), NetError> {
        let Some(mut entry) = self.tcp_connections.remove(&connection) else {
            return Err(NetError::UnknownConnection { connection });
        };
        if let Some(event) = entry.state_machine.transition(NetConnectionState::Closing) {
            self.push_event(event);
        }
        if let Some(event) = entry.state_machine.transition(NetConnectionState::Closed) {
            self.push_event(event);
        }
        self.push_event(NetEvent::ConnectionClosed {
            connection,
            transport: NetTransportKind::Tcp,
        });
        Ok(())
    }

    fn connection_state(
        &self,
        connection: NetConnectionId,
    ) -> Result<NetConnectionState, NetError> {
        self.tcp_connections
            .get(&connection)
            .map(|entry| entry.state_machine.state())
            .ok_or(NetError::UnknownConnection { connection })
    }

    fn shutdown_report(&self, drained_egress_commands: usize) -> NetWorkerShutdownReport {
        NetWorkerShutdownReport {
            drained_egress_commands,
            open_udp_sockets_closed: self.udp_sockets.len(),
            open_tcp_listeners_closed: self.tcp_listeners.len(),
            open_tcp_connections_closed: self.tcp_connections.len(),
        }
    }

    fn next_connection_id(&self) -> NetConnectionId {
        NetConnectionId::new(self.next_connection_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn push_event(&self, event: NetEvent) {
        let _ = self.ingress.try_send(NetIngress::Event(event));
    }
}

fn endpoint_from_addr(addr: SocketAddr) -> NetEndpoint {
    NetEndpoint::from(addr)
}

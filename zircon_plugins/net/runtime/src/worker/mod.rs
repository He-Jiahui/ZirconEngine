mod egress;
mod ingress;
mod shutdown;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::runtime::{Builder, Runtime};
use tokio::time::timeout;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetEvent, NetListenerId, NetPacket,
    NetSocketId, NetTransportKind,
};

use crate::transport::TransportStateMachine;

pub(crate) use self::egress::{AcceptedTcpConnection, NetEgress, TcpPollResult, WorkerReply};
use self::ingress::NetIngress;
pub(crate) use self::shutdown::NetWorkerShutdownReport;

const DEFAULT_EGRESS_CAPACITY: usize = 1024;
const DEFAULT_INGRESS_CAPACITY: usize = 1024;
const COMMAND_REPLY_TIMEOUT: Duration = Duration::from_secs(2);
const TCP_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

#[derive(Debug)]
pub(crate) struct NetWorker {
    egress: mpsc::SyncSender<NetEgress>,
    ingress: Mutex<mpsc::Receiver<NetIngress>>,
    thread: Mutex<Option<JoinHandle<()>>>,
    shutdown: AtomicBool,
}

impl NetWorker {
    pub(crate) fn spawn(next_connection_id: Arc<AtomicU64>) -> Result<Self, NetError> {
        let (egress, worker_egress) = mpsc::sync_channel(DEFAULT_EGRESS_CAPACITY);
        let (worker_ingress, ingress) = mpsc::sync_channel(DEFAULT_INGRESS_CAPACITY);
        let thread = thread::Builder::new()
            .name("zircon-net-worker".to_string())
            .spawn(move || run_worker(worker_egress, worker_ingress, next_connection_id))
            .map_err(|error| NetError::Io(error.to_string()))?;

        Ok(Self {
            egress,
            ingress: Mutex::new(ingress),
            thread: Mutex::new(Some(thread)),
            shutdown: AtomicBool::new(false),
        })
    }

    pub(crate) fn bind_udp(
        &self,
        socket: NetSocketId,
        bind: NetEndpoint,
    ) -> Result<NetEndpoint, NetError> {
        self.request(|reply| NetEgress::BindUdp {
            socket,
            bind,
            reply,
        })
    }

    pub(crate) fn send_udp(
        &self,
        socket: NetSocketId,
        destination: NetEndpoint,
        payload: Vec<u8>,
    ) -> Result<usize, NetError> {
        self.request(|reply| NetEgress::SendUdp {
            socket,
            destination,
            payload,
            reply,
        })
    }

    pub(crate) fn poll_udp(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        self.request(|reply| NetEgress::PollUdp {
            socket,
            max_packets,
            reply,
        })
    }

    pub(crate) fn close_udp(&self, socket: NetSocketId) -> Result<(), NetError> {
        self.request(|reply| NetEgress::CloseUdp { socket, reply })
    }

    pub(crate) fn listen_tcp(
        &self,
        listener: NetListenerId,
        bind: NetEndpoint,
    ) -> Result<NetEndpoint, NetError> {
        self.request(|reply| NetEgress::ListenTcp {
            listener,
            bind,
            reply,
        })
    }

    pub(crate) fn accept_tcp(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<AcceptedTcpConnection>, NetError> {
        self.request(|reply| NetEgress::AcceptTcp {
            listener,
            max_connections,
            reply,
        })
    }

    pub(crate) fn close_tcp_listener(&self, listener: NetListenerId) -> Result<(), NetError> {
        self.request(|reply| NetEgress::CloseTcpListener { listener, reply })
    }

    pub(crate) fn connect_tcp(
        &self,
        connection: NetConnectionId,
        remote: NetEndpoint,
    ) -> Result<(), NetError> {
        self.request(|reply| NetEgress::ConnectTcp {
            connection,
            remote,
            reply,
        })
    }

    pub(crate) fn send_tcp(
        &self,
        connection: NetConnectionId,
        payload: Vec<u8>,
    ) -> Result<usize, NetError> {
        self.request(|reply| NetEgress::SendTcp {
            connection,
            payload,
            reply,
        })
    }

    pub(crate) fn poll_tcp(
        &self,
        connection: NetConnectionId,
        max_bytes: usize,
    ) -> Result<TcpPollResult, NetError> {
        self.request(|reply| NetEgress::PollTcp {
            connection,
            max_bytes,
            reply,
        })
    }

    pub(crate) fn close_tcp(&self, connection: NetConnectionId) -> Result<(), NetError> {
        self.request(|reply| NetEgress::CloseTcp { connection, reply })
    }

    pub(crate) fn drain_ingress(&self, max_events: usize) -> Vec<NetEvent> {
        let mut events = Vec::new();
        let ingress = self
            .ingress
            .lock()
            .expect("net worker ingress mutex poisoned");
        while events.len() < max_events {
            match ingress.try_recv() {
                Ok(ingress) => events.push(ingress.into_event()),
                Err(_) => break,
            }
        }
        events
    }

    pub(crate) fn shutdown(&self) -> Result<NetWorkerShutdownReport, NetError> {
        if self.shutdown.swap(true, Ordering::SeqCst) {
            return Ok(NetWorkerShutdownReport::default());
        }

        let (reply, receiver) = mpsc::sync_channel(1);
        self.egress
            .try_send(NetEgress::Shutdown { reply })
            .map_err(|error| NetError::Io(format!("net worker shutdown send failed: {error}")))?;
        let report = receiver
            .recv_timeout(COMMAND_REPLY_TIMEOUT)
            .map_err(|error| {
                NetError::Io(format!(
                    "net worker shutdown response timed out or closed: {error}"
                ))
            })??;
        if let Some(thread) = self
            .thread
            .lock()
            .expect("net worker thread mutex poisoned")
            .take()
        {
            thread
                .join()
                .map_err(|_| NetError::Io("net worker thread panicked during shutdown".into()))?;
        }
        Ok(report)
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    fn request<T>(&self, build: impl FnOnce(WorkerReply<T>) -> NetEgress) -> Result<T, NetError> {
        if self.is_shutdown() {
            return Err(NetError::Io("net worker is shut down".to_string()));
        }

        let (reply, receiver) = mpsc::sync_channel(1);
        self.egress
            .try_send(build(reply))
            .map_err(|error| NetError::Io(format!("net worker egress send failed: {error}")))?;
        receiver
            .recv_timeout(COMMAND_REPLY_TIMEOUT)
            .map_err(|error| {
                NetError::Io(format!(
                    "net worker command response timed out or closed: {error}"
                ))
            })?
    }
}

impl Drop for NetWorker {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

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
                        remote: remote_endpoint.clone(),
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

fn run_worker(
    egress: mpsc::Receiver<NetEgress>,
    ingress: mpsc::SyncSender<NetIngress>,
    next_connection_id: Arc<AtomicU64>,
) {
    let Ok(mut core) = WorkerCore::new(ingress, next_connection_id) else {
        return;
    };

    while let Ok(command) = egress.recv() {
        if handle_command(&mut core, command, &egress) {
            break;
        }
    }
}

fn handle_command(
    core: &mut WorkerCore,
    command: NetEgress,
    egress: &mpsc::Receiver<NetEgress>,
) -> bool {
    match command {
        NetEgress::BindUdp {
            socket,
            bind,
            reply,
        } => reply_result(reply, core.bind_udp(socket, bind)),
        NetEgress::SendUdp {
            socket,
            destination,
            payload,
            reply,
        } => reply_result(reply, core.send_udp(socket, destination, &payload)),
        NetEgress::PollUdp {
            socket,
            max_packets,
            reply,
        } => reply_result(reply, core.poll_udp(socket, max_packets)),
        NetEgress::CloseUdp { socket, reply } => reply_result(reply, core.close_udp(socket)),
        NetEgress::ListenTcp {
            listener,
            bind,
            reply,
        } => reply_result(reply, core.listen_tcp(listener, bind)),
        NetEgress::AcceptTcp {
            listener,
            max_connections,
            reply,
        } => reply_result(reply, core.accept_tcp(listener, max_connections)),
        NetEgress::CloseTcpListener { listener, reply } => {
            reply_result(reply, core.close_tcp_listener(listener))
        }
        NetEgress::ConnectTcp {
            connection,
            remote,
            reply,
        } => reply_result(reply, core.connect_tcp(connection, remote)),
        NetEgress::SendTcp {
            connection,
            payload,
            reply,
        } => reply_result(reply, core.send_tcp(connection, &payload)),
        NetEgress::PollTcp {
            connection,
            max_bytes,
            reply,
        } => reply_result(reply, core.poll_tcp(connection, max_bytes)),
        NetEgress::CloseTcp { connection, reply } => {
            reply_result(reply, core.close_tcp(connection))
        }
        NetEgress::Shutdown { reply } => {
            let report = core.shutdown_report(egress.try_iter().count());
            reply_result(reply, Ok(report));
            return true;
        }
    }
    false
}

fn reply_result<T>(reply: WorkerReply<T>, value: Result<T, NetError>) {
    let _ = reply.send(value);
}

fn endpoint_from_addr(addr: SocketAddr) -> NetEndpoint {
    NetEndpoint::from(addr)
}

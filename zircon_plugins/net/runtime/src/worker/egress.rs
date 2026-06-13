use std::sync::mpsc::SyncSender;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetListenerId, NetPacket,
    NetSocketId,
};

use super::shutdown::NetWorkerShutdownReport;

pub(crate) type WorkerReply<T> = SyncSender<Result<T, NetError>>;

#[derive(Debug)]
pub(crate) struct AcceptedTcpConnection {
    pub(crate) connection: NetConnectionId,
}

#[derive(Debug)]
pub(crate) struct TcpPollResult {
    pub(crate) payload: Vec<u8>,
    pub(crate) state: NetConnectionState,
}

#[derive(Debug)]
pub(crate) enum NetEgress {
    BindUdp {
        socket: NetSocketId,
        bind: NetEndpoint,
        reply: WorkerReply<NetEndpoint>,
    },
    SendUdp {
        socket: NetSocketId,
        destination: NetEndpoint,
        payload: Vec<u8>,
        reply: WorkerReply<usize>,
    },
    PollUdp {
        socket: NetSocketId,
        max_packets: usize,
        reply: WorkerReply<Vec<NetPacket>>,
    },
    CloseUdp {
        socket: NetSocketId,
        reply: WorkerReply<()>,
    },
    ListenTcp {
        listener: NetListenerId,
        bind: NetEndpoint,
        reply: WorkerReply<NetEndpoint>,
    },
    AcceptTcp {
        listener: NetListenerId,
        max_connections: usize,
        reply: WorkerReply<Vec<AcceptedTcpConnection>>,
    },
    CloseTcpListener {
        listener: NetListenerId,
        reply: WorkerReply<()>,
    },
    ConnectTcp {
        connection: NetConnectionId,
        remote: NetEndpoint,
        reply: WorkerReply<()>,
    },
    SendTcp {
        connection: NetConnectionId,
        payload: Vec<u8>,
        reply: WorkerReply<usize>,
    },
    PollTcp {
        connection: NetConnectionId,
        max_bytes: usize,
        reply: WorkerReply<TcpPollResult>,
    },
    CloseTcp {
        connection: NetConnectionId,
        reply: WorkerReply<()>,
    },
    Shutdown {
        reply: WorkerReply<NetWorkerShutdownReport>,
    },
}

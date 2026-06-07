use serde::{Deserialize, Serialize};

use super::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetHttpMethod, NetListenerId, NetRouteId,
    NetSocketId, NetTransportKind,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetEvent {
    UdpSocketBound {
        socket: NetSocketId,
        endpoint: NetEndpoint,
    },
    UdpSocketClosed {
        socket: NetSocketId,
    },
    ListenerStarted {
        listener: NetListenerId,
        transport: NetTransportKind,
        endpoint: NetEndpoint,
    },
    ListenerClosed {
        listener: NetListenerId,
        transport: NetTransportKind,
    },
    ConnectionStateChanged {
        connection: NetConnectionId,
        transport: NetTransportKind,
        state: NetConnectionState,
    },
    ConnectionAccepted {
        listener: NetListenerId,
        connection: NetConnectionId,
        transport: NetTransportKind,
        remote: NetEndpoint,
    },
    ConnectionClosed {
        connection: NetConnectionId,
        transport: NetTransportKind,
    },
    HttpRouteRegistered {
        route: NetRouteId,
        path: String,
        methods: Vec<NetHttpMethod>,
    },
    HttpRouteUnregistered {
        route: NetRouteId,
    },
    WebSocketPairOpened {
        client: NetConnectionId,
        server: NetConnectionId,
    },
    WebSocketFrameQueued {
        connection: NetConnectionId,
        queued_frames: usize,
    },
}

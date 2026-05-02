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
    ListenerStarted {
        listener: NetListenerId,
        transport: NetTransportKind,
        endpoint: NetEndpoint,
    },
    ConnectionStateChanged {
        connection: NetConnectionId,
        transport: NetTransportKind,
        state: NetConnectionState,
    },
    ConnectionAccepted {
        listener: NetListenerId,
        connection: NetConnectionId,
        remote: NetEndpoint,
    },
    ConnectionClosed {
        connection: NetConnectionId,
    },
    HttpRouteRegistered {
        route: NetRouteId,
        path: String,
        methods: Vec<NetHttpMethod>,
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

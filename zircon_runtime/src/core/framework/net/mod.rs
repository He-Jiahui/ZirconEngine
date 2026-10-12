//! Networking framework contracts for transport, session, RPC, sync, and download surfaces.
//! 本模块定义 Runtime 与网络插件共享的数据类型及 NetManager 服务契约；实际网络 I/O 由插件服务执行，HTTP/WebSocket 可通过可替换后端提供。

#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "download.rs"]
mod download;
#[path = "endpoint.rs"]
mod endpoint;
#[path = "error.rs"]
mod error;
#[path = "event.rs"]
mod event;
#[path = "http.rs"]
mod http;
#[path = "ids.rs"]
mod ids;
#[path = "manager.rs"]
mod manager;
#[path = "packet.rs"]
mod packet;
#[path = "reliable.rs"]
mod reliable;
#[path = "rpc.rs"]
mod rpc;
#[path = "session.rs"]
mod session;
#[path = "socket_id.rs"]
mod socket_id;
#[path = "sync.rs"]
mod sync;
#[path = "transport.rs"]
mod transport;
#[path = "websocket.rs"]
mod websocket;

pub use diagnostics::NetDiagnostics;
pub use download::{
    NetDownloadAttemptDescriptor, NetDownloadChunk, NetDownloadManifest, NetDownloadProgress,
    NetDownloadStatus,
};
pub use endpoint::NetEndpoint;
pub use error::NetError;
pub use event::NetEvent;
pub use http::{
    NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor,
};
pub use ids::{
    NetConnectionId, NetDownloadId, NetListenerId, NetObjectId, NetRequestId, NetRouteId,
    NetSessionId,
};
pub use manager::NetManager;
pub use packet::NetPacket;
pub use reliable::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramDeliveryReport,
    ReliableDatagramPacket, ReliableDatagramReceiveReport, ReliableDatagramReceiveStatus,
    ReliableDatagramRecoveryReport, ReliableDatagramRecoveryState, ReliableDatagramSendReport,
    ReliableDatagramSendStatus, ReliableDatagramSimulationProfile, ReliableDatagramStats,
};
pub use rpc::{
    RpcDescriptor, RpcDirection, RpcDispatchReport, RpcDispatchStatus, RpcInvocationDescriptor,
    RpcPayloadSchema, RpcPeerRole,
};
pub use session::{
    NetControlMessage, NetRuntimeMode, NetSessionControlReport, NetSessionHandshakePolicy,
    NetSessionHandshakeState, NetSessionInfo,
};
pub use socket_id::NetSocketId;
pub use sync::{
    NetworkIdentity, SyncAuthority, SyncComponentDescriptor, SyncDelta, SyncFieldDescriptor,
    SyncFieldValue, SyncInterestDescriptor, SyncObjectSnapshot, SyncReplicationBudget,
    SyncReplicationScheduleReport, SyncReplicationStrategy, SYNC_DEFAULT_COMPONENT_UPDATE_HZ,
    SYNC_DEFAULT_REPLICATION_PRIORITY, SYNC_REPLICATION_UNBOUNDED_BUDGET,
};
pub use transport::{
    NetCertificatePin, NetCertificateRoot, NetConnectionState, NetSecurityPolicy, NetTransportKind,
};
pub use websocket::{
    NetWebSocketCloseReason, NetWebSocketConnectDescriptor, NetWebSocketFrame,
    NetWebSocketListenerDescriptor,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

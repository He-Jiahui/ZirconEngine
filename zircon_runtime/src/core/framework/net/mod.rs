//! Networking framework contracts for transport, session, RPC, sync, and download surfaces.

mod diagnostics;
mod download;
mod endpoint;
mod error;
mod event;
mod http;
mod ids;
mod manager;
mod packet;
mod reliable;
mod rpc;
mod session;
mod socket_id;
mod sync;
mod transport;
mod websocket;

pub use diagnostics::NetDiagnostics;
pub use download::{NetDownloadChunk, NetDownloadManifest, NetDownloadProgress, NetDownloadStatus};
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
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramPacket,
    ReliableDatagramSendReport, ReliableDatagramSendStatus, ReliableDatagramStats,
};
pub use rpc::{
    RpcDescriptor, RpcDirection, RpcDispatchReport, RpcDispatchStatus, RpcInvocationDescriptor,
    RpcPeerRole,
};
pub use session::{
    NetControlMessage, NetRuntimeMode, NetSessionControlReport, NetSessionHandshakePolicy,
    NetSessionHandshakeState, NetSessionInfo,
};
pub use socket_id::NetSocketId;
pub use sync::{
    SyncAuthority, SyncComponentDescriptor, SyncDelta, SyncFieldDescriptor, SyncFieldValue,
    SyncInterestDescriptor, SyncObjectSnapshot,
};
pub use transport::{NetCertificatePin, NetConnectionState, NetSecurityPolicy, NetTransportKind};
pub use websocket::{NetWebSocketCloseReason, NetWebSocketConnectDescriptor, NetWebSocketFrame};

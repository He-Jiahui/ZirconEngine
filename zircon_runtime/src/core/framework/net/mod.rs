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
pub use download::{NetDownloadChunk, NetDownloadManifest, NetDownloadStatus};
pub use endpoint::NetEndpoint;
pub use error::NetError;
pub use event::NetEvent;
pub use http::{
    NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor,
};
pub use ids::{
    NetConnectionId, NetDownloadId, NetListenerId, NetRequestId, NetRouteId, NetSessionId,
};
pub use manager::NetManager;
pub use packet::NetPacket;
pub use reliable::{ReliableDatagramConfig, ReliableDatagramStats};
pub use rpc::{RpcDescriptor, RpcDirection};
pub use session::{NetControlMessage, NetRuntimeMode};
pub use socket_id::NetSocketId;
pub use sync::{SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor};
pub use transport::{NetConnectionState, NetSecurityPolicy, NetTransportKind};
pub use websocket::{NetWebSocketCloseReason, NetWebSocketFrame};

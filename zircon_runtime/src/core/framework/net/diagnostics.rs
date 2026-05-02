use serde::{Deserialize, Serialize};

use super::NetRuntimeMode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetDiagnostics {
    pub backend_name: String,
    pub mode: NetRuntimeMode,
    pub open_udp_sockets: usize,
    pub open_tcp_listeners: usize,
    pub open_tcp_connections: usize,
    pub open_http_routes: usize,
    pub open_websocket_connections: usize,
    pub queued_events: usize,
}

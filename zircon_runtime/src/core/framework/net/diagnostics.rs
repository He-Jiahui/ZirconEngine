use serde::{Deserialize, Serialize};

use super::NetRuntimeMode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 管理器在查询时返回的计数快照，用于观测后端、连接与事件队列，不承诺与随后的收发操作原子一致。
pub struct NetDiagnostics {
    pub backend_name: String,
    pub mode: NetRuntimeMode,
    #[serde(default)]
    pub outbound_bytes: u64,
    #[serde(default)]
    pub inbound_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_observed_latency_ms: Option<u64>,
    pub open_udp_sockets: usize,
    pub open_tcp_listeners: usize,
    pub open_http_listeners: usize,
    pub open_websocket_listeners: usize,
    pub open_tcp_connections: usize,
    pub open_http_routes: usize,
    pub open_websocket_connections: usize,
    pub queued_events: usize,
}

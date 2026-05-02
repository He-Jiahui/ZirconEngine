use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetTransportKind {
    Udp,
    Tcp,
    Http,
    WebSocket,
    ReliableUdp,
}

impl NetTransportKind {
    pub fn is_tcp(self) -> bool {
        matches!(self, Self::Tcp)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetConnectionState {
    Connecting,
    Open,
    Closing,
    Closed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetSecurityPolicy {
    pub tls_required: bool,
    pub certificate_pinning: bool,
    pub allow_insecure_loopback: bool,
}

impl NetSecurityPolicy {
    pub fn development() -> Self {
        Self {
            tls_required: false,
            certificate_pinning: false,
            allow_insecure_loopback: true,
        }
    }

    pub fn production_tls() -> Self {
        Self {
            tls_required: true,
            certificate_pinning: false,
            allow_insecure_loopback: false,
        }
    }
}

impl Default for NetSecurityPolicy {
    fn default() -> Self {
        Self::development()
    }
}

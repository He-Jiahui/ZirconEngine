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
pub struct NetCertificatePin {
    pub host: String,
    pub sha256: String,
}

impl NetCertificatePin {
    pub fn new(host: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            sha256: sha256.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetSecurityPolicy {
    pub tls_required: bool,
    pub certificate_pinning: bool,
    pub certificate_pins: Vec<NetCertificatePin>,
    pub allow_insecure_loopback: bool,
}

impl NetSecurityPolicy {
    pub fn development() -> Self {
        Self {
            tls_required: false,
            certificate_pinning: false,
            certificate_pins: Vec::new(),
            allow_insecure_loopback: true,
        }
    }

    pub fn production_tls() -> Self {
        Self {
            tls_required: true,
            certificate_pinning: false,
            certificate_pins: Vec::new(),
            allow_insecure_loopback: false,
        }
    }

    pub fn with_certificate_pin(
        mut self,
        host: impl Into<String>,
        sha256: impl Into<String>,
    ) -> Self {
        self.certificate_pinning = true;
        self.certificate_pins
            .push(NetCertificatePin::new(host, sha256));
        self
    }

    pub fn has_pin_for_host(&self, host: &str) -> bool {
        self.certificate_pins
            .iter()
            .any(|pin| pin.host.eq_ignore_ascii_case(host) && !pin.sha256.trim().is_empty())
    }
}

impl Default for NetSecurityPolicy {
    fn default() -> Self {
        Self::development()
    }
}

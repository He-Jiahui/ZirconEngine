use serde::{Deserialize, Serialize};

use super::{NetEndpoint, NetSecurityPolicy};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetWebSocketConnectDescriptor {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub protocols: Vec<String>,
    pub timeout_ms: u64,
    pub security: NetSecurityPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetWebSocketListenerDescriptor {
    pub bind: NetEndpoint,
    pub allowed_paths: Vec<String>,
    pub required_headers: Vec<(String, String)>,
    pub allowed_protocols: Vec<String>,
}

impl NetWebSocketListenerDescriptor {
    pub fn new(bind: NetEndpoint) -> Self {
        Self {
            bind,
            allowed_paths: Vec::new(),
            required_headers: Vec::new(),
            allowed_protocols: Vec::new(),
        }
    }

    pub fn with_allowed_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    pub fn with_required_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.required_headers.push((name.into(), value.into()));
        self
    }

    pub fn with_allowed_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.allowed_protocols.push(protocol.into());
        self
    }
}

impl NetWebSocketConnectDescriptor {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
            protocols: Vec::new(),
            timeout_ms: 30_000,
            security: NetSecurityPolicy::default(),
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocols.push(protocol.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetWebSocketFrame {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(NetWebSocketCloseReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetWebSocketCloseReason {
    pub code: u16,
    pub reason: String,
    pub clean: bool,
}

impl NetWebSocketCloseReason {
    pub fn normal(reason: impl Into<String>) -> Self {
        Self {
            code: 1000,
            reason: reason.into(),
            clean: true,
        }
    }
}

use std::fmt;
use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::NetError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Socket 端点的序列化表示。to_socket_addr 只接受 IP 字面量；需要 DNS 的调用方应在更高层解析主机名。
pub struct NetEndpoint {
    pub host: String,
    pub port: u16,
}

impl NetEndpoint {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn to_socket_addr(&self) -> Result<SocketAddr, NetError> {
        let literal_host = self
            .host
            .strip_prefix('[')
            .and_then(|host| host.strip_suffix(']'))
            .unwrap_or(self.host.as_str());
        let address = literal_host
            .parse::<IpAddr>()
            .map_err(|_| NetError::InvalidEndpoint {
                endpoint: self.to_string(),
            })?;
        Ok(SocketAddr::new(address, self.port))
    }
}

impl From<SocketAddr> for NetEndpoint {
    fn from(value: SocketAddr) -> Self {
        Self {
            host: value.ip().to_string(),
            port: value.port(),
        }
    }
}

impl fmt::Display for NetEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') && !(self.host.starts_with('[') && self.host.ends_with(']')) {
            return write!(f, "[{}]:{}", self.host, self.port);
        }
        write!(f, "{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
#[path = "tests/endpoint_optimization_batch_ez_tests.rs"]
mod optimization_batch_ez_tests;

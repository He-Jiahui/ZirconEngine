use std::fmt;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::NetError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|_| NetError::InvalidEndpoint {
                endpoint: self.to_string(),
            })
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
        write!(f, "{}:{}", self.host, self.port)
    }
}

use serde::{Deserialize, Serialize};

use super::NetEndpoint;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetPacket {
    pub source: NetEndpoint,
    pub payload: Vec<u8>,
}

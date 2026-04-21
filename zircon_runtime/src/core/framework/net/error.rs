use serde::{Deserialize, Serialize};

use super::NetSocketId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetError {
    InvalidEndpoint { endpoint: String },
    UnknownSocket { socket: NetSocketId },
    Io(String),
}

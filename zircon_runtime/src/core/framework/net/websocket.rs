use serde::{Deserialize, Serialize};

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

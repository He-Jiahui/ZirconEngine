use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcDirection {
    ClientToServer,
    ServerToClient,
    TargetClient,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcDescriptor {
    pub id: String,
    pub direction: RpcDirection,
    pub payload_schema: Option<String>,
    pub max_calls_per_second: Option<u32>,
    pub max_payload_bytes: Option<usize>,
}

impl RpcDescriptor {
    pub fn new(id: impl Into<String>, direction: RpcDirection) -> Self {
        Self {
            id: id.into(),
            direction,
            payload_schema: None,
            max_calls_per_second: None,
            max_payload_bytes: None,
        }
    }

    pub fn with_payload_schema(mut self, schema: impl Into<String>) -> Self {
        self.payload_schema = Some(schema.into());
        self
    }

    pub fn with_max_calls_per_second(mut self, limit: u32) -> Self {
        self.max_calls_per_second = Some(limit);
        self
    }

    pub fn with_max_payload_bytes(mut self, limit: usize) -> Self {
        self.max_payload_bytes = Some(limit);
        self
    }
}

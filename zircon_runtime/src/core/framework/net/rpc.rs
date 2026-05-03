use serde::{Deserialize, Serialize};

use super::{NetRequestId, NetSessionId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcDirection {
    ClientToServer,
    ServerToClient,
    TargetClient,
}

impl RpcDirection {
    pub fn allows_caller(self, caller: RpcPeerRole) -> bool {
        match self {
            Self::ClientToServer => caller == RpcPeerRole::Client,
            Self::ServerToClient | Self::TargetClient => caller == RpcPeerRole::Server,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcPeerRole {
    Server,
    Client,
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

    pub fn command(id: impl Into<String>) -> Self {
        Self::new(id, RpcDirection::ClientToServer)
    }

    pub fn client_rpc(id: impl Into<String>) -> Self {
        Self::new(id, RpcDirection::ServerToClient)
    }

    pub fn target_rpc(id: impl Into<String>) -> Self {
        Self::new(id, RpcDirection::TargetClient)
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcInvocationDescriptor {
    pub request: Option<NetRequestId>,
    pub rpc_id: String,
    pub direction: RpcDirection,
    pub source_session: Option<NetSessionId>,
    pub target_session: Option<NetSessionId>,
    pub payload: Vec<u8>,
    pub timeout_ms: Option<u64>,
    pub priority: u8,
}

impl RpcInvocationDescriptor {
    pub fn new(rpc_id: impl Into<String>, direction: RpcDirection, payload: Vec<u8>) -> Self {
        Self {
            request: None,
            rpc_id: rpc_id.into(),
            direction,
            source_session: None,
            target_session: None,
            payload,
            timeout_ms: None,
            priority: 0,
        }
    }

    pub fn with_request(mut self, request: NetRequestId) -> Self {
        self.request = Some(request);
        self
    }

    pub fn with_source_session(mut self, session: NetSessionId) -> Self {
        self.source_session = Some(session);
        self
    }

    pub fn with_target_session(mut self, session: NetSessionId) -> Self {
        self.target_session = Some(session);
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn payload_bytes(&self) -> usize {
        self.payload.len()
    }
}

/// Diagnostics-first result; real handler execution can be layered on accepted calls later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcDispatchStatus {
    Accepted,
    Queued,
    NoHandler,
    DirectionDenied,
    SchemaUnavailable,
    SchemaRejected,
    SessionUnavailable,
    QuotaExceeded,
    QueueFull,
    PayloadTooLarge,
    TimedOut,
    HandlerFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcDispatchReport {
    pub rpc_id: String,
    pub request: Option<NetRequestId>,
    pub direction: RpcDirection,
    pub status: RpcDispatchStatus,
    pub source_session: Option<NetSessionId>,
    pub target_session: Option<NetSessionId>,
    pub payload_bytes: usize,
    pub schema: Option<String>,
    pub diagnostic: Option<String>,
    pub response_payload: Option<Vec<u8>>,
}

impl RpcDispatchReport {
    pub fn for_invocation(invocation: &RpcInvocationDescriptor, status: RpcDispatchStatus) -> Self {
        Self {
            rpc_id: invocation.rpc_id.clone(),
            request: invocation.request,
            direction: invocation.direction,
            status,
            source_session: invocation.source_session,
            target_session: invocation.target_session,
            payload_bytes: invocation.payload_bytes(),
            schema: None,
            diagnostic: None,
            response_payload: None,
        }
    }

    pub fn with_schema(mut self, schema: Option<String>) -> Self {
        self.schema = schema;
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }

    pub fn with_response_payload(mut self, payload: Vec<u8>) -> Self {
        self.response_payload = Some(payload);
        self
    }
}

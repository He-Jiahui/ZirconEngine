use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetRequestId, NetSessionHandshakePolicy, NetSessionHandshakeState,
    NetSessionId, NetSessionInfo, RpcDescriptor, RpcInvocationDescriptor, RpcPeerRole,
};

use crate::feature::NET_RPC_FEATURE_CAPABILITY;

use super::{RpcHandler, RpcSchemaValidator, DEFAULT_RPC_QUEUE_DEPTH, RPC_PROTOCOL_VERSION};

#[derive(Clone)]
pub struct NetRpcRuntimeManager {
    pub(in crate::manager) state: Arc<Mutex<NetRpcRuntimeState>>,
}

pub(in crate::manager) struct NetRpcRuntimeState {
    pub(in crate::manager) next_session_id: u64,
    pub(in crate::manager) handshake_policy: NetSessionHandshakePolicy,
    pub(in crate::manager) sessions: HashMap<NetSessionId, NetRpcSessionState>,
    pub(in crate::manager) rpc_descriptors: HashMap<String, RpcDescriptor>,
    pub(in crate::manager) schema_validators: HashMap<String, RpcSchemaValidator>,
    pub(in crate::manager) rpc_handlers: HashMap<String, RpcHandler>,
    pub(in crate::manager) quota_windows: HashMap<RpcQuotaKey, RpcQuotaWindow>,
    pub(in crate::manager) netspeed_windows: HashMap<NetSessionId, NetSpeedWindow>,
    pub(in crate::manager) queued_invocations: Vec<QueuedRpcInvocation>,
    pub(in crate::manager) pending_requests: HashMap<NetRequestId, PendingRpcRequest>,
    pub(in crate::manager) next_queue_sequence: u64,
    pub(in crate::manager) max_queue_depth: usize,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct PendingRpcRequest {
    pub(in crate::manager) invocation: RpcInvocationDescriptor,
    pub(in crate::manager) started_at: Instant,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct QueuedRpcInvocation {
    pub(in crate::manager) invocation: RpcInvocationDescriptor,
    pub(in crate::manager) caller: RpcPeerRole,
    pub(in crate::manager) enqueued_at: Instant,
    pub(in crate::manager) sequence: u64,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct NetRpcSessionState {
    pub(in crate::manager) connection: Option<NetConnectionId>,
    pub(in crate::manager) handshake_state: NetSessionHandshakeState,
    pub(in crate::manager) player_id: Option<String>,
    pub(in crate::manager) netspeed_bytes_per_second: Option<u32>,
}

impl NetRpcSessionState {
    pub(in crate::manager) fn new(connection: Option<NetConnectionId>) -> Self {
        Self {
            connection,
            handshake_state: NetSessionHandshakeState::AwaitingHello,
            player_id: None,
            netspeed_bytes_per_second: None,
        }
    }

    pub(in crate::manager) fn info(&self, session: NetSessionId) -> NetSessionInfo {
        NetSessionInfo::new(
            session,
            self.connection,
            self.handshake_state,
            self.player_id.clone(),
            self.netspeed_bytes_per_second,
        )
    }
}

// Scope quotas per RPC and source session to isolate noisy clients.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::manager) struct RpcQuotaKey {
    pub(in crate::manager) rpc_id: String,
    pub(in crate::manager) source_session: Option<NetSessionId>,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct RpcQuotaWindow {
    pub(in crate::manager) started_at: Instant,
    pub(in crate::manager) calls: u32,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct NetSpeedWindow {
    pub(in crate::manager) started_at: Instant,
    pub(in crate::manager) bytes: usize,
}

impl NetRpcRuntimeManager {
    pub fn new() -> Self {
        Self::with_handshake_policy(
            NetSessionHandshakePolicy::new(RPC_PROTOCOL_VERSION)
                .with_required_feature(NET_RPC_FEATURE_CAPABILITY),
        )
    }

    pub fn with_handshake_policy(policy: NetSessionHandshakePolicy) -> Self {
        Self {
            state: Arc::new(Mutex::new(NetRpcRuntimeState {
                next_session_id: 0,
                handshake_policy: policy,
                sessions: HashMap::new(),
                rpc_descriptors: HashMap::new(),
                schema_validators: HashMap::new(),
                rpc_handlers: HashMap::new(),
                quota_windows: HashMap::new(),
                netspeed_windows: HashMap::new(),
                queued_invocations: Vec::new(),
                pending_requests: HashMap::new(),
                next_queue_sequence: 0,
                max_queue_depth: DEFAULT_RPC_QUEUE_DEPTH,
            })),
        }
    }

    pub fn with_max_queue_depth(max_queue_depth: usize) -> Self {
        let manager = Self::new();
        manager
            .state
            .lock()
            .expect("net RPC state mutex poisoned")
            .max_queue_depth = max_queue_depth;
        manager
    }
}

impl Default for NetRpcRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

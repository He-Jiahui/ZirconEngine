//! RPC manager clone 共享握手策略、session、回调注册、quota 窗口和优先级队列。
//! 状态锁只同步内存协议；连接、网络收发和玩家身份均需外层提供。

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetError, NetRequestId, NetSessionHandshakePolicy, NetSessionHandshakeState,
    NetSessionId, NetSessionInfo, RpcDescriptor, RpcInvocationDescriptor, RpcPeerRole,
};

use crate::feature::NET_RPC_FEATURE_CAPABILITY;

use super::{
    RpcChannelMessage, RpcHandler, RpcSchemaValidator, DEFAULT_RPC_QUEUE_DEPTH,
    RPC_PROTOCOL_VERSION,
};

/// Clones share one protocol state. Poisoning permanently makes that state
/// unavailable: fallible operations return SharedStatePoisoned, observations
/// return None/empty, and dispatch rejects without executing handlers. Construct
/// a new manager to establish fresh sessions; partial authentication is not reused.
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
    pub(in crate::manager) queued_invocations: BinaryHeap<QueuedRpcInvocation>,
    pub(in crate::manager) pending_requests: HashMap<NetRequestId, PendingRpcRequest>,
    pub(in crate::manager) next_queue_sequence: u64,
    pub(in crate::manager) max_queue_depth: usize,
    pub(in crate::manager) channel_queues: HashMap<u8, VecDeque<RpcChannelMessage>>,
    pub(in crate::manager) channel_sequences: HashMap<u8, u64>,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct PendingRpcRequest {
    pub(in crate::manager) invocation: RpcInvocationDescriptor,
    pub(in crate::manager) started_at: Instant,
    /// A synchronous handler cannot be cancelled; expiration must not invite a retry
    /// while its side effects may still be running.
    pub(in crate::manager) handler_executing: bool,
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct QueuedRpcInvocation {
    pub(in crate::manager) invocation: RpcInvocationDescriptor,
    pub(in crate::manager) caller: RpcPeerRole,
    pub(in crate::manager) enqueued_at: Instant,
    pub(in crate::manager) sequence: u64,
}

impl PartialEq for QueuedRpcInvocation {
    fn eq(&self, other: &Self) -> bool {
        self.invocation.priority == other.invocation.priority && self.sequence == other.sequence
    }
}

impl Eq for QueuedRpcInvocation {}

impl PartialOrd for QueuedRpcInvocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedRpcInvocation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.invocation
            .priority
            .cmp(&other.invocation.priority)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
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
        Self::with_max_queue_depth(DEFAULT_RPC_QUEUE_DEPTH)
    }

    pub fn with_handshake_policy(policy: NetSessionHandshakePolicy) -> Self {
        Self::with_configuration(policy, DEFAULT_RPC_QUEUE_DEPTH)
    }

    pub fn with_max_queue_depth(max_queue_depth: usize) -> Self {
        Self::with_configuration(
            NetSessionHandshakePolicy::new(RPC_PROTOCOL_VERSION)
                .with_required_feature(NET_RPC_FEATURE_CAPABILITY),
            max_queue_depth,
        )
    }

    fn with_configuration(policy: NetSessionHandshakePolicy, max_queue_depth: usize) -> Self {
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
                queued_invocations: BinaryHeap::new(),
                pending_requests: HashMap::new(),
                next_queue_sequence: 0,
                max_queue_depth,
                channel_queues: HashMap::new(),
                channel_sequences: HashMap::new(),
            })),
        }
    }

    // Authentication, quota and request bookkeeping may be partially updated.
    // Never recover or clear this poison, even for a later call through a clone.
    pub(in crate::manager) fn lock_state(
        &self,
    ) -> Result<MutexGuard<'_, NetRpcRuntimeState>, NetError> {
        self.state.lock().map_err(|_| NetError::SharedStatePoisoned {
            resource: "net.rpc_state".to_string(),
        })
    }
}

#[cfg(test)]
#[path = "../tests/poisoned_manager.rs"]
mod poisoned_manager_tests;

impl Default for NetRpcRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

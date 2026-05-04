use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetControlMessage, NetError, NetEvent, NetRequestId,
    NetSessionControlReport, NetSessionHandshakePolicy, NetSessionHandshakeState, NetSessionId,
    NetSessionInfo, RpcDescriptor, RpcDirection, RpcDispatchReport, RpcDispatchStatus,
    RpcInvocationDescriptor, RpcPeerRole,
};

use crate::feature::NET_RPC_FEATURE_CAPABILITY;

const RPC_PROTOCOL_VERSION: u32 = 1;
const RPC_QUOTA_WINDOW: Duration = Duration::from_secs(1);
const DEFAULT_RPC_QUEUE_DEPTH: usize = 256;

type RpcSchemaValidator = Arc<dyn Fn(&[u8]) -> bool + Send + Sync>;
type RpcHandler = Arc<dyn Fn(&RpcInvocationDescriptor) -> Result<Vec<u8>, String> + Send + Sync>;

#[derive(Clone)]
pub struct NetRpcRuntimeManager {
    state: Arc<Mutex<NetRpcRuntimeState>>,
}

struct NetRpcRuntimeState {
    next_session_id: u64,
    handshake_policy: NetSessionHandshakePolicy,
    sessions: HashMap<NetSessionId, NetRpcSessionState>,
    rpc_descriptors: HashMap<String, RpcDescriptor>,
    schema_validators: HashMap<String, RpcSchemaValidator>,
    rpc_handlers: HashMap<String, RpcHandler>,
    quota_windows: HashMap<RpcQuotaKey, RpcQuotaWindow>,
    netspeed_windows: HashMap<NetSessionId, NetSpeedWindow>,
    queued_invocations: Vec<QueuedRpcInvocation>,
    pending_requests: HashMap<NetRequestId, PendingRpcRequest>,
    next_queue_sequence: u64,
    max_queue_depth: usize,
}

#[derive(Clone, Debug)]
struct PendingRpcRequest {
    invocation: RpcInvocationDescriptor,
    started_at: Instant,
}

#[derive(Clone, Debug)]
struct QueuedRpcInvocation {
    invocation: RpcInvocationDescriptor,
    caller: RpcPeerRole,
    enqueued_at: Instant,
    sequence: u64,
}

#[derive(Clone, Debug)]
struct NetRpcSessionState {
    connection: Option<NetConnectionId>,
    handshake_state: NetSessionHandshakeState,
    player_id: Option<String>,
    netspeed_bytes_per_second: Option<u32>,
}

impl NetRpcSessionState {
    fn new(connection: Option<NetConnectionId>) -> Self {
        Self {
            connection,
            handshake_state: NetSessionHandshakeState::AwaitingHello,
            player_id: None,
            netspeed_bytes_per_second: None,
        }
    }

    fn info(&self, session: NetSessionId) -> NetSessionInfo {
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
struct RpcQuotaKey {
    rpc_id: String,
    source_session: Option<NetSessionId>,
}

#[derive(Clone, Debug)]
struct RpcQuotaWindow {
    started_at: Instant,
    calls: u32,
}

#[derive(Clone, Debug)]
struct NetSpeedWindow {
    started_at: Instant,
    bytes: usize,
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

    pub fn begin_handshake(&self) -> NetSessionId {
        self.begin_session(None)
    }

    pub fn begin_handshake_for_connection(&self, connection: NetConnectionId) -> NetSessionId {
        self.begin_session(Some(connection))
    }

    fn begin_session(&self, connection: Option<NetConnectionId>) -> NetSessionId {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        state.next_session_id += 1;
        let session = NetSessionId::new(state.next_session_id);
        state
            .sessions
            .insert(session, NetRpcSessionState::new(connection));
        session
    }

    pub fn handshake_state(
        &self,
        session: NetSessionId,
    ) -> Result<NetSessionHandshakeState, NetError> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .sessions
            .get(&session)
            .map(|session_state| session_state.handshake_state)
            .ok_or(NetError::UnknownSession { session })
    }

    pub fn session_info(&self, session: NetSessionId) -> Result<NetSessionInfo, NetError> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .sessions
            .get(&session)
            .map(|session_state| session_state.info(session))
            .ok_or(NetError::UnknownSession { session })
    }

    pub fn close_session(&self, session: NetSessionId) -> Result<NetSessionInfo, NetError> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let session_state = state
            .sessions
            .get_mut(&session)
            .ok_or(NetError::UnknownSession { session })?;
        session_state.handshake_state = NetSessionHandshakeState::Closed;
        Ok(session_state.info(session))
    }

    pub fn close_connection_sessions(&self, connection: NetConnectionId) -> Vec<NetSessionInfo> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        state
            .sessions
            .iter_mut()
            .filter_map(|(session, session_state)| {
                (session_state.connection == Some(connection)).then(|| {
                    session_state.handshake_state = NetSessionHandshakeState::Closed;
                    session_state.info(*session)
                })
            })
            .collect()
    }

    pub fn apply_transport_events(
        &self,
        events: impl IntoIterator<Item = NetEvent>,
    ) -> Vec<NetSessionInfo> {
        let mut closed = Vec::new();
        for event in events {
            match event {
                NetEvent::ConnectionClosed { connection }
                | NetEvent::ConnectionStateChanged {
                    connection,
                    state: NetConnectionState::Closed | NetConnectionState::Failed,
                    ..
                } => closed.extend(self.close_connection_sessions(connection)),
                _ => {}
            }
        }
        closed
    }

    pub fn process_control_message(
        &self,
        session: NetSessionId,
        message: NetControlMessage,
    ) -> Result<NetSessionControlReport, NetError> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let current = state
            .sessions
            .get(&session)
            .map(|session_state| session_state.handshake_state)
            .ok_or(NetError::UnknownSession { session })?;
        let login_player_id = match &message {
            NetControlMessage::Login { player_id, .. } => Some(player_id.clone()),
            _ => None,
        };
        let netspeed_bytes_per_second = match &message {
            NetControlMessage::NetSpeed { bytes_per_second } => Some(*bytes_per_second),
            _ => None,
        };
        let (next, response) =
            Self::advance_handshake(current, &state.handshake_policy, session, message);
        let session_state = state
            .sessions
            .get_mut(&session)
            .expect("session should exist after control lookup");
        session_state.handshake_state = next;
        if matches!(
            next,
            NetSessionHandshakeState::Welcomed | NetSessionHandshakeState::Joined
        ) {
            if let Some(player_id) = login_player_id {
                session_state.player_id = Some(player_id);
            }
            if let Some(bytes_per_second) = netspeed_bytes_per_second {
                session_state.netspeed_bytes_per_second = Some(bytes_per_second);
            }
        }
        Ok(NetSessionControlReport::new(session, next, response))
    }

    pub fn register_rpc(&self, descriptor: RpcDescriptor) -> Result<(), NetError> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .rpc_descriptors
            .insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn register_schema_validator(
        &self,
        schema: impl Into<String>,
        validator: impl Fn(&[u8]) -> bool + Send + Sync + 'static,
    ) {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .schema_validators
            .insert(schema.into(), Arc::new(validator));
    }

    pub fn register_rpc_handler(
        &self,
        descriptor: RpcDescriptor,
        handler: impl Fn(&RpcInvocationDescriptor) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    ) -> Result<(), NetError> {
        let rpc_id = descriptor.id.clone();
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        state.rpc_descriptors.insert(rpc_id.clone(), descriptor);
        state.rpc_handlers.insert(rpc_id, Arc::new(handler));
        Ok(())
    }

    pub fn rpc_descriptor(&self, id: &str) -> Option<RpcDescriptor> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .rpc_descriptors
            .get(id)
            .cloned()
    }

    pub fn pending_request(&self, request: NetRequestId) -> Option<RpcInvocationDescriptor> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .pending_requests
            .get(&request)
            .map(|pending| pending.invocation.clone())
    }

    pub fn dispatch_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        Self::validate_invocation(&mut state, &invocation, caller, true)
    }

    pub fn invoke_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        if Self::invocation_timed_out(&invocation, Instant::now()) {
            return RpcDispatchReport::for_invocation(&invocation, RpcDispatchStatus::TimedOut)
                .with_diagnostic("RPC invocation timed out before handler execution");
        }

        let (mut report, handler) = {
            let mut state = self.state.lock().expect("net RPC state mutex poisoned");
            let report = Self::validate_invocation(&mut state, &invocation, caller, true);
            let handler = if report.status == RpcDispatchStatus::Accepted {
                Self::track_pending_request(&mut state, &invocation);
                state.rpc_handlers.get(&invocation.rpc_id).cloned()
            } else {
                None
            };
            (report, handler)
        };

        let Some(handler) = handler else {
            if report.status == RpcDispatchStatus::Accepted {
                report.status = RpcDispatchStatus::NoHandler;
            }
            self.complete_pending_request(&invocation);
            return report;
        };

        let report = match handler(&invocation) {
            Ok(payload) => report.with_response_payload(payload),
            Err(error) => {
                report.status = RpcDispatchStatus::HandlerFailed;
                report.with_diagnostic(error)
            }
        };
        self.complete_pending_request(&invocation);
        report
    }

    pub fn enqueue_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let mut report = Self::validate_invocation(&mut state, &invocation, caller, false);
        if report.status != RpcDispatchStatus::Accepted {
            return report;
        }
        if state.queued_invocations.len() >= state.max_queue_depth {
            report.status = RpcDispatchStatus::QueueFull;
            return report.with_diagnostic("RPC queue depth exceeded");
        }
        report = Self::validate_invocation(&mut state, &invocation, caller, true);
        if report.status != RpcDispatchStatus::Accepted {
            return report;
        }
        let sequence = state.next_queue_sequence;
        state.next_queue_sequence += 1;
        state.queued_invocations.push(QueuedRpcInvocation {
            invocation,
            caller,
            enqueued_at: Instant::now(),
            sequence,
        });
        report.status = RpcDispatchStatus::Queued;
        report
    }

    pub fn drain_rpc_queue(&self, max_invocations: usize) -> Vec<RpcDispatchReport> {
        let queued = {
            let mut state = self.state.lock().expect("net RPC state mutex poisoned");
            state.queued_invocations.sort_by(|left, right| {
                right
                    .invocation
                    .priority
                    .cmp(&left.invocation.priority)
                    .then_with(|| left.sequence.cmp(&right.sequence))
            });
            let take = max_invocations.min(state.queued_invocations.len());
            state.queued_invocations.drain(0..take).collect::<Vec<_>>()
        };

        let now = Instant::now();
        queued
            .into_iter()
            .map(|queued| {
                if Self::queued_invocation_timed_out(&queued, now) {
                    RpcDispatchReport::for_invocation(
                        &queued.invocation,
                        RpcDispatchStatus::TimedOut,
                    )
                    .with_diagnostic("queued RPC invocation timed out")
                } else {
                    self.invoke_queued_rpc(queued.invocation, queued.caller)
                }
            })
            .collect()
    }

    fn invoke_queued_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        let (mut report, handler) = {
            let mut state = self.state.lock().expect("net RPC state mutex poisoned");
            let report = Self::validate_invocation(&mut state, &invocation, caller, false);
            let handler = if report.status == RpcDispatchStatus::Accepted {
                Self::track_pending_request(&mut state, &invocation);
                state.rpc_handlers.get(&invocation.rpc_id).cloned()
            } else {
                None
            };
            (report, handler)
        };

        let Some(handler) = handler else {
            if report.status == RpcDispatchStatus::Accepted {
                report.status = RpcDispatchStatus::NoHandler;
            }
            self.complete_pending_request(&invocation);
            return report;
        };

        let report = match handler(&invocation) {
            Ok(payload) => report.with_response_payload(payload),
            Err(error) => {
                report.status = RpcDispatchStatus::HandlerFailed;
                report.with_diagnostic(error)
            }
        };
        self.complete_pending_request(&invocation);
        report
    }

    pub fn expire_pending_requests(&self) -> Vec<RpcDispatchReport> {
        let now = Instant::now();
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let expired = state
            .pending_requests
            .iter()
            .filter_map(|(request, pending)| {
                pending.invocation.timeout_ms.and_then(|timeout_ms| {
                    (timeout_ms == 0
                        || now.duration_since(pending.started_at).as_millis() > timeout_ms as u128)
                        .then_some(*request)
                })
            })
            .collect::<Vec<_>>();
        expired
            .into_iter()
            .filter_map(|request| state.pending_requests.remove(&request))
            .map(|pending| {
                RpcDispatchReport::for_invocation(&pending.invocation, RpcDispatchStatus::TimedOut)
                    .with_diagnostic("pending RPC request timed out")
            })
            .collect()
    }

    fn validate_invocation(
        state: &mut NetRpcRuntimeState,
        invocation: &RpcInvocationDescriptor,
        caller: RpcPeerRole,
        account_limits: bool,
    ) -> RpcDispatchReport {
        let Some(descriptor) = state.rpc_descriptors.get(&invocation.rpc_id).cloned() else {
            return RpcDispatchReport::for_invocation(invocation, RpcDispatchStatus::NoHandler);
        };

        let mut report = RpcDispatchReport::for_invocation(invocation, RpcDispatchStatus::Accepted)
            .with_schema(descriptor.payload_schema.clone());

        if descriptor.direction != invocation.direction
            || !descriptor.direction.allows_caller(caller)
        {
            report.status = RpcDispatchStatus::DirectionDenied;
            return report;
        }

        if descriptor.direction == RpcDirection::ClientToServer {
            let Some(source_session) = invocation.source_session else {
                report.status = RpcDispatchStatus::SessionUnavailable;
                return report.with_diagnostic("client-to-server RPC requires a source session");
            };
            let Some(source_session_state) = state.sessions.get(&source_session) else {
                report.status = RpcDispatchStatus::SessionUnavailable;
                return report.with_diagnostic("source session is unknown");
            };
            match source_session_state.handshake_state {
                NetSessionHandshakeState::Joined => {}
                NetSessionHandshakeState::Closed => {
                    report.status = RpcDispatchStatus::SessionUnavailable;
                    return report.with_diagnostic("source session is closed");
                }
                _ => {
                    report.status = RpcDispatchStatus::SessionUnavailable;
                    return report.with_diagnostic("source session is not joined");
                }
            }
        }

        if descriptor
            .max_payload_bytes
            .is_some_and(|limit| invocation.payload_bytes() > limit)
        {
            report.status = RpcDispatchStatus::PayloadTooLarge;
            return report;
        }

        if let Some(schema) = descriptor.payload_schema.as_deref() {
            let Some(validator) = state.schema_validators.get(schema) else {
                report.status = RpcDispatchStatus::SchemaUnavailable;
                return report.with_diagnostic("schema validator unavailable");
            };
            if !validator(&invocation.payload) {
                report.status = RpcDispatchStatus::SchemaRejected;
                return report.with_diagnostic("schema validation rejected payload");
            }
        }

        if account_limits {
            let netspeed_budget = invocation.source_session.and_then(|source_session| {
                state
                    .sessions
                    .get(&source_session)
                    .and_then(|session| session.netspeed_bytes_per_second)
                    .map(|bytes_per_second| (source_session, bytes_per_second))
            });
            if let Some((source_session, bytes_per_second)) = netspeed_budget {
                if !Self::record_netspeed_bytes(
                    state,
                    source_session,
                    bytes_per_second,
                    invocation.payload_bytes(),
                ) {
                    report.status = RpcDispatchStatus::QuotaExceeded;
                    return report.with_diagnostic("source session NetSpeed byte budget exceeded");
                }
            }

            if descriptor
                .max_calls_per_second
                .is_some_and(|limit| !Self::record_quota_call(state, invocation, limit))
            {
                report.status = RpcDispatchStatus::QuotaExceeded;
                return report;
            }
        }

        report
    }

    fn invocation_timed_out(invocation: &RpcInvocationDescriptor, now: Instant) -> bool {
        let _ = now;
        invocation
            .timeout_ms
            .is_some_and(|timeout_ms| timeout_ms == 0)
    }

    fn track_pending_request(state: &mut NetRpcRuntimeState, invocation: &RpcInvocationDescriptor) {
        if let Some(request) = invocation.request {
            state.pending_requests.insert(
                request,
                PendingRpcRequest {
                    invocation: invocation.clone(),
                    started_at: Instant::now(),
                },
            );
        }
    }

    fn complete_pending_request(&self, invocation: &RpcInvocationDescriptor) {
        if let Some(request) = invocation.request {
            self.state
                .lock()
                .expect("net RPC state mutex poisoned")
                .pending_requests
                .remove(&request);
        }
    }

    fn queued_invocation_timed_out(queued: &QueuedRpcInvocation, now: Instant) -> bool {
        queued.invocation.timeout_ms.is_some_and(|timeout_ms| {
            timeout_ms == 0
                || now.duration_since(queued.enqueued_at).as_millis() > timeout_ms as u128
        })
    }

    fn advance_handshake(
        current: NetSessionHandshakeState,
        policy: &NetSessionHandshakePolicy,
        session: NetSessionId,
        message: NetControlMessage,
    ) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        match (current, message) {
            (
                NetSessionHandshakeState::AwaitingHello,
                NetControlMessage::Hello {
                    protocol_version,
                    runtime_features,
                },
            ) => Self::accept_hello(policy, protocol_version, &runtime_features),
            (
                NetSessionHandshakeState::AwaitingLogin,
                NetControlMessage::Login {
                    player_id,
                    challenge_response,
                },
            ) => Self::accept_login(policy, session, &player_id, &challenge_response),
            (NetSessionHandshakeState::Welcomed, NetControlMessage::NetSpeed { .. }) => {
                (NetSessionHandshakeState::Welcomed, None)
            }
            (NetSessionHandshakeState::Welcomed, NetControlMessage::Join) => {
                (NetSessionHandshakeState::Joined, None)
            }
            (NetSessionHandshakeState::Joined, NetControlMessage::NetSpeed { .. })
            | (NetSessionHandshakeState::Joined, NetControlMessage::Join) => {
                (NetSessionHandshakeState::Joined, None)
            }
            (NetSessionHandshakeState::Failed, _) => (NetSessionHandshakeState::Failed, None),
            (NetSessionHandshakeState::Closed, _) => (NetSessionHandshakeState::Closed, None),
            _ => Self::failure("unexpected control message"),
        }
    }

    fn accept_hello(
        policy: &NetSessionHandshakePolicy,
        protocol_version: u32,
        runtime_features: &[String],
    ) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        if protocol_version != policy.protocol_version {
            return Self::failure("protocol version mismatch");
        }

        if let Some(feature) = policy
            .required_features
            .iter()
            .find(|required| !runtime_features.contains(required))
        {
            return Self::failure(format!("missing required feature: {feature}"));
        }

        (
            NetSessionHandshakeState::AwaitingLogin,
            Some(NetControlMessage::Challenge {
                nonce: policy.challenge_nonce.clone(),
            }),
        )
    }

    fn accept_login(
        policy: &NetSessionHandshakePolicy,
        session: NetSessionId,
        player_id: &str,
        challenge_response: &str,
    ) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        if player_id.trim().is_empty() || challenge_response != policy.challenge_nonce {
            return Self::failure("challenge response rejected");
        }

        (
            NetSessionHandshakeState::Welcomed,
            Some(NetControlMessage::Welcome {
                session_id: session.raw().to_string(),
                map: policy.welcome_map.clone(),
            }),
        )
    }

    fn failure(reason: impl Into<String>) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        (
            NetSessionHandshakeState::Failed,
            Some(NetControlMessage::Failure {
                reason: reason.into(),
            }),
        )
    }

    fn record_quota_call(
        state: &mut NetRpcRuntimeState,
        invocation: &RpcInvocationDescriptor,
        max_calls: u32,
    ) -> bool {
        let now = Instant::now();
        let key = RpcQuotaKey {
            rpc_id: invocation.rpc_id.clone(),
            source_session: invocation.source_session,
        };
        let window = state.quota_windows.entry(key).or_insert(RpcQuotaWindow {
            started_at: now,
            calls: 0,
        });
        if now.duration_since(window.started_at) >= RPC_QUOTA_WINDOW {
            window.started_at = now;
            window.calls = 0;
        }
        if window.calls >= max_calls {
            return false;
        }
        window.calls += 1;
        true
    }

    fn record_netspeed_bytes(
        state: &mut NetRpcRuntimeState,
        session: NetSessionId,
        bytes_per_second: u32,
        payload_bytes: usize,
    ) -> bool {
        let max_bytes = bytes_per_second as usize;
        if payload_bytes > max_bytes {
            return false;
        }

        let now = Instant::now();
        let window = state
            .netspeed_windows
            .entry(session)
            .or_insert(NetSpeedWindow {
                started_at: now,
                bytes: 0,
            });
        if now.duration_since(window.started_at) >= RPC_QUOTA_WINDOW {
            window.started_at = now;
            window.bytes = 0;
        }
        if window.bytes.saturating_add(payload_bytes) > max_bytes {
            return false;
        }
        window.bytes += payload_bytes;
        true
    }
}

impl Default for NetRpcRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn net_rpc_runtime_manager() -> NetRpcRuntimeManager {
    NetRpcRuntimeManager::default()
}

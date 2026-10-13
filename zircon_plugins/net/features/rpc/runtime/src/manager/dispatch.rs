//! 对 RPC 调用执行方向、session、schema、quota 和 netspeed 检查，再调用 handler 或按优先级排队。
//! handler 在锁外执行以允许回调访问 manager；同步执行不能抢占，迟到完成保留真实结果并附诊断。

use std::time::{Duration, Instant};

use zircon_runtime::core::framework::net::{
    NetRequestId, NetSessionHandshakeState, RpcDispatchReport, RpcDispatchStatus,
    RpcInvocationDescriptor, RpcPeerRole,
};

use super::{
    state::{NetRpcRuntimeState, PendingRpcRequest, QueuedRpcInvocation},
    NetRpcRuntimeManager, RpcHandler,
};

impl NetRpcRuntimeManager {
    pub fn pending_request(&self, request: NetRequestId) -> Option<RpcInvocationDescriptor> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .pending_requests
            .get(&request)
            .map(|pending| pending.invocation.clone())
    }

    /// 仅做调用准入检查并记配额，不执行已注册 handler；需要执行使用 invoke_rpc，需排队使用 enqueue_rpc。
    pub fn dispatch_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        self.admit_invocation(&invocation, caller, true)
    }

    /// Invoke synchronously. A zero timeout is rejected before execution; a positive
    /// timeout measures handler execution, which cannot be cancelled. Late completions
    /// retain their real status/response and carry a diagnostic to prevent unsafe retries.
    pub fn invoke_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        if Self::invocation_timed_out(&invocation) {
            return RpcDispatchReport::for_invocation(&invocation, RpcDispatchStatus::TimedOut)
                .with_diagnostic("RPC invocation timed out before handler execution");
        }

        let (mut report, handler) = self.admit_invocation_for_execution(&invocation, caller, true);

        let Some(handler) = handler else {
            if report.status == RpcDispatchStatus::Accepted {
                report.status = RpcDispatchStatus::NoHandler;
            }
            self.complete_pending_request(&invocation);
            return report;
        };

        let _pending_guard = PendingRequestGuard::new(self, invocation.request);
        let report = Self::invoke_handler_with_timeout(&invocation, report, handler);
        report
    }

    pub fn enqueue_rpc(
        &self,
        invocation: RpcInvocationDescriptor,
        caller: RpcPeerRole,
    ) -> RpcDispatchReport {
        let mut report = self.admit_invocation(&invocation, caller, false);
        if report.status != RpcDispatchStatus::Accepted {
            return report;
        }
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
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

    /// 按优先级与同优先级 FIFO 限量执行已准入调用；重新检查 session/schema，避免排队后权限变化。
    /// 配额在入队时计费，出队执行不重复计费；同步 handler 的时间不受条数预算限制。
    pub fn drain_rpc_queue(&self, max_invocations: usize) -> Vec<RpcDispatchReport> {
        let queued = {
            let mut state = self.state.lock().expect("net RPC state mutex poisoned");
            take_queued_invocations(&mut state.queued_invocations, max_invocations)
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
        let (mut report, handler) = self.admit_invocation_for_execution(&invocation, caller, false);

        let Some(handler) = handler else {
            if report.status == RpcDispatchStatus::Accepted {
                report.status = RpcDispatchStatus::NoHandler;
            }
            self.complete_pending_request(&invocation);
            return report;
        };

        let _pending_guard = PendingRequestGuard::new(self, invocation.request);
        let report = Self::invoke_handler_with_timeout(&invocation, report, handler);
        report
    }

    /// Expire requests that are waiting outside synchronous handler execution. Active
    /// handlers remain pending until they return because this runtime cannot cancel them.
    pub fn expire_pending_requests(&self) -> Vec<RpcDispatchReport> {
        let now = Instant::now();
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let mut expired_reports = Vec::new();
        state.pending_requests.retain(|_, pending| {
            let expired = !pending.handler_executing
                && pending.invocation.timeout_ms.is_some_and(|timeout_ms| {
                    timeout_ms == 0
                        || now.duration_since(pending.started_at).as_millis() > timeout_ms as u128
                });
            if expired {
                expired_reports.push(
                    RpcDispatchReport::for_invocation(
                        &pending.invocation,
                        RpcDispatchStatus::TimedOut,
                    )
                    .with_diagnostic("pending RPC request timed out"),
                );
            }
            !expired
        });
        expired_reports
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
            .with_schema(
                descriptor
                    .payload_schema
                    .as_ref()
                    .map(|schema| schema.schema_id.clone()),
            );

        if !descriptor
            .direction
            .allows_invocation(invocation.direction, caller)
        {
            report.status = RpcDispatchStatus::DirectionDenied;
            return report;
        }

        // Session admission follows the origin of this invocation, including when the
        // registered descriptor is bidirectional.
        if caller == RpcPeerRole::Client {
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

        if let Some(schema) = descriptor.payload_schema.as_ref() {
            let Some(_validator) = state.schema_validators.get(schema.schema_id()) else {
                report.status = RpcDispatchStatus::SchemaUnavailable;
                return report.with_diagnostic("schema validator unavailable");
            };
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

    /// Validate against a stable descriptor/validator snapshot. User code runs with no
    /// manager lock held; a concurrent registration change rejects this admission.
    fn admit_invocation(
        &self,
        invocation: &RpcInvocationDescriptor,
        caller: RpcPeerRole,
        account_limits: bool,
    ) -> RpcDispatchReport {
        self.admit_invocation_inner(invocation, caller, account_limits, false)
            .0
    }

    fn admit_invocation_for_execution(
        &self,
        invocation: &RpcInvocationDescriptor,
        caller: RpcPeerRole,
        account_limits: bool,
    ) -> (RpcDispatchReport, Option<RpcHandler>) {
        self.admit_invocation_inner(invocation, caller, account_limits, true)
    }

    fn admit_invocation_inner(
        &self,
        invocation: &RpcInvocationDescriptor,
        caller: RpcPeerRole,
        account_limits: bool,
        capture_handler: bool,
    ) -> (RpcDispatchReport, Option<RpcHandler>) {
        let (preflight, descriptor, validator, handler) = {
            let mut state = self.state.lock().expect("net RPC state mutex poisoned");
            let report = Self::validate_invocation(&mut state, invocation, caller, false);
            if report.status != RpcDispatchStatus::Accepted {
                return (report, None);
            }
            let descriptor = state.rpc_descriptors.get(&invocation.rpc_id).cloned();
            let validator = descriptor
                .as_ref()
                .and_then(|d| d.payload_schema.as_ref())
                .and_then(|schema| state.schema_validators.get(schema.schema_id()))
                .cloned();
            let handler = state.rpc_handlers.get(&invocation.rpc_id).cloned();
            (report, descriptor, validator, handler)
        };

        if let Some(validator) = validator.as_ref() {
            if !validator(&invocation.payload) {
                let mut rejected = preflight;
                rejected.status = RpcDispatchStatus::SchemaRejected;
                return (
                    rejected.with_diagnostic("schema validation rejected payload"),
                    None,
                );
            }
        }

        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let current_descriptor = state.rpc_descriptors.get(&invocation.rpc_id);
        let descriptor_matches = current_descriptor == descriptor.as_ref();
        let current_validator = descriptor
            .as_ref()
            .and_then(|d| d.payload_schema.as_ref())
            .and_then(|schema| state.schema_validators.get(schema.schema_id()));
        let validator_matches = match (current_validator, validator.as_ref()) {
            (None, None) => true,
            (Some(current), Some(validated)) => std::sync::Arc::ptr_eq(current, validated),
            _ => false,
        };
        let current_handler = state.rpc_handlers.get(&invocation.rpc_id);
        let handler_matches = match (current_handler, handler.as_ref()) {
            (None, None) => true,
            (Some(current), Some(validated)) => std::sync::Arc::ptr_eq(current, validated),
            _ => false,
        };
        if !descriptor_matches || !validator_matches || (capture_handler && !handler_matches) {
            let mut changed = preflight;
            changed.status = RpcDispatchStatus::SessionUnavailable;
            return (
                changed.with_diagnostic("RPC admission policy changed during validation"),
                None,
            );
        }
        let report = Self::validate_invocation(&mut state, invocation, caller, account_limits);
        if report.status != RpcDispatchStatus::Accepted || !capture_handler {
            return (report, None);
        }
        if handler.is_some() {
            Self::track_pending_request(&mut state, invocation);
        }
        (report, handler)
    }

    fn invocation_timed_out(invocation: &RpcInvocationDescriptor) -> bool {
        invocation
            .timeout_ms
            .is_some_and(|timeout_ms| timeout_ms == 0)
    }

    /// `timeout_ms == 0` is rejected before execution. For positive values, synchronous
    /// handlers run to completion; an over-budget completion is returned with its real
    /// result and a diagnostic so callers do not retry potentially committed effects.
    fn invoke_handler_with_timeout(
        invocation: &RpcInvocationDescriptor,
        mut report: RpcDispatchReport,
        handler: RpcHandler,
    ) -> RpcDispatchReport {
        let started_at = Instant::now();
        let handler_result = handler(invocation);
        // A synchronous handler cannot be cancelled. Preserve its actual completion
        // result so the caller does not retry an operation whose side effects committed.
        if Self::handler_timed_out(invocation, started_at, Instant::now()) {
            report = report.with_diagnostic(
                "RPC handler completed after its deadline; synchronous execution was not cancelled",
            );
        }

        match handler_result {
            Ok(payload) => report.with_response_payload(payload),
            Err(error) => {
                report.status = RpcDispatchStatus::HandlerFailed;
                report.with_diagnostic(error)
            }
        }
    }

    fn handler_timed_out(
        invocation: &RpcInvocationDescriptor,
        started_at: Instant,
        now: Instant,
    ) -> bool {
        invocation.timeout_ms.is_some_and(|timeout_ms| {
            timeout_ms == 0 || now.duration_since(started_at) > Duration::from_millis(timeout_ms)
        })
    }

    // TODO: [CR-PLUGIN-NET-0018] pending 仅以 request ID 为 key；确认跨 session/并发调用的唯一性和迟到完成归属。
    fn track_pending_request(state: &mut NetRpcRuntimeState, invocation: &RpcInvocationDescriptor) {
        if let Some(request) = invocation.request {
            state.pending_requests.insert(
                request,
                PendingRpcRequest {
                    invocation: invocation.clone(),
                    started_at: Instant::now(),
                    handler_executing: true,
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
}

struct PendingRequestGuard<'a> {
    manager: &'a NetRpcRuntimeManager,
    request: Option<NetRequestId>,
}

impl<'a> PendingRequestGuard<'a> {
    fn new(manager: &'a NetRpcRuntimeManager, request: Option<NetRequestId>) -> Self {
        Self { manager, request }
    }
}

impl Drop for PendingRequestGuard<'_> {
    fn drop(&mut self) {
        if let Some(request) = self.request {
            self.manager
                .state
                .lock()
                .expect("net RPC state mutex poisoned")
                .pending_requests
                .remove(&request);
        }
    }
}

#[cfg(test)]
#[path = "../tests/dispatch_expiration_sweep.rs"]
mod expiration_sweep_tests;

fn take_queued_invocations(
    queued: &mut std::collections::BinaryHeap<QueuedRpcInvocation>,
    max_invocations: usize,
) -> Vec<QueuedRpcInvocation> {
    let take = max_invocations.min(queued.len());
    let mut drained = Vec::with_capacity(take);
    for _ in 0..take {
        if let Some(invocation) = queued.pop() {
            drained.push(invocation);
        }
    }
    drained
}

#[cfg(test)]
#[path = "../tests/dispatch_priority_queue.rs"]
mod priority_queue_tests;

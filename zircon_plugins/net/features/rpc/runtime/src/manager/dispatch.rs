use std::time::{Duration, Instant};

use zircon_runtime::core::framework::net::{
    NetRequestId, NetSessionHandshakeState, RpcDirection, RpcDispatchReport, RpcDispatchStatus,
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
        if Self::invocation_timed_out(&invocation) {
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

        let report = Self::invoke_handler_with_timeout(&invocation, report, handler);
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

        let report = Self::invoke_handler_with_timeout(&invocation, report, handler);
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

    fn invocation_timed_out(invocation: &RpcInvocationDescriptor) -> bool {
        invocation
            .timeout_ms
            .is_some_and(|timeout_ms| timeout_ms == 0)
    }

    fn invoke_handler_with_timeout(
        invocation: &RpcInvocationDescriptor,
        mut report: RpcDispatchReport,
        handler: RpcHandler,
    ) -> RpcDispatchReport {
        let started_at = Instant::now();
        let handler_result = handler(invocation);
        // Synchronous feature handlers cannot be preempted here, so timeout
        // hardening reports and discards late completions after the closure returns.
        if Self::handler_timed_out(invocation, started_at, Instant::now()) {
            report.status = RpcDispatchStatus::TimedOut;
            return report.with_diagnostic("RPC handler exceeded invocation timeout");
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
}

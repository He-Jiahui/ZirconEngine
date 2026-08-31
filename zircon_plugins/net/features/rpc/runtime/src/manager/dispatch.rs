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
        let mut expired_reports = Vec::new();
        state.pending_requests.retain(|_, pending| {
            let expired = pending.invocation.timeout_ms.is_some_and(|timeout_ms| {
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

        if let Some(schema) = descriptor.payload_schema.as_ref() {
            let Some(validator) = state.schema_validators.get(schema.schema_id()) else {
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

#[cfg(test)]
mod expiration_sweep_tests {
    use std::time::{Duration, Instant};

    use zircon_runtime::core::framework::net::{
        NetRequestId, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor,
    };

    use super::{NetRpcRuntimeManager, PendingRpcRequest};

    #[test]
    fn expiration_sweep_removes_only_timed_out_requests() {
        let manager = NetRpcRuntimeManager::new();
        let expired_request = NetRequestId::new(501);
        let live_request = NetRequestId::new(502);
        let now = Instant::now();
        {
            let mut state = manager.state.lock().expect("net RPC state mutex poisoned");
            state.pending_requests.insert(
                expired_request,
                PendingRpcRequest {
                    invocation: RpcInvocationDescriptor::new(
                        "expiration.expired",
                        RpcDirection::ServerToClient,
                        Vec::new(),
                    )
                    .with_request(expired_request)
                    .with_timeout_ms(1),
                    started_at: now - Duration::from_millis(10),
                },
            );
            state.pending_requests.insert(
                live_request,
                PendingRpcRequest {
                    invocation: RpcInvocationDescriptor::new(
                        "expiration.live",
                        RpcDirection::ServerToClient,
                        Vec::new(),
                    )
                    .with_request(live_request)
                    .with_timeout_ms(60_000),
                    started_at: now,
                },
            );
        }

        let reports = manager.expire_pending_requests();

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].status, RpcDispatchStatus::TimedOut);
        assert_eq!(reports[0].request, Some(expired_request));
        assert_eq!(
            reports[0].diagnostic.as_deref(),
            Some("pending RPC request timed out")
        );
        assert!(manager.pending_request(expired_request).is_none());
        assert!(manager.pending_request(live_request).is_some());
    }
}

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
mod priority_queue_tests {
    use std::{collections::BinaryHeap, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        RpcDirection, RpcInvocationDescriptor, RpcPeerRole,
    };

    use super::{take_queued_invocations, QueuedRpcInvocation};

    const RPC_QUEUE_BENCHMARK_DEPTH: usize = 100_000;
    const RPC_QUEUE_BENCHMARK_DRAIN: usize = 64;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    #[ignore = "release-only performance evidence"]
    fn rpc_priority_heap_release_benchmark_evidence() {
        let legacy_base = benchmark_invocations();
        let heap_base = legacy_base.iter().cloned().collect::<BinaryHeap<_>>();

        let mut legacy_equivalence = legacy_base.clone();
        let mut heap_equivalence = heap_base.clone();
        assert_eq!(
            legacy_take_queued_invocations(&mut legacy_equivalence, RPC_QUEUE_BENCHMARK_DRAIN)
                .iter()
                .map(|queued| queued.sequence)
                .collect::<Vec<_>>(),
            take_queued_invocations(&mut heap_equivalence, RPC_QUEUE_BENCHMARK_DRAIN)
                .iter()
                .map(|queued| queued.sequence)
                .collect::<Vec<_>>()
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || {
                let mut queued = legacy_base.clone();
                legacy_take_queued_invocations(&mut queued, RPC_QUEUE_BENCHMARK_DRAIN)
            },
            || {
                let mut queued = heap_base.clone();
                take_queued_invocations(&mut queued, RPC_QUEUE_BENCHMARK_DRAIN)
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_front_shifts = RPC_QUEUE_BENCHMARK_DEPTH - RPC_QUEUE_BENCHMARK_DRAIN;
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins10_rpc_priority_heap depth={} drain={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_full_queue_sorts=1 optimized_full_queue_sorts=0 legacy_front_shifts={} optimized_front_shifts=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            RPC_QUEUE_BENCHMARK_DEPTH,
            RPC_QUEUE_BENCHMARK_DRAIN,
            BENCHMARK_SAMPLE_COUNT,
            legacy_front_shifts,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 20% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn benchmark_invocations() -> Vec<QueuedRpcInvocation> {
        let now = Instant::now();
        (0..RPC_QUEUE_BENCHMARK_DEPTH as u64)
            .map(|sequence| QueuedRpcInvocation {
                invocation: RpcInvocationDescriptor::new(
                    "benchmark.rpc",
                    RpcDirection::ServerToClient,
                    Vec::new(),
                )
                .with_priority(((sequence * 17) % 256) as u8),
                caller: RpcPeerRole::Server,
                enqueued_at: now,
                sequence,
            })
            .collect()
    }

    fn benchmark_paired_samples<L, O>(
        mut legacy: impl FnMut() -> L,
        mut optimized: impl FnMut() -> O,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(&result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }

    fn legacy_take_queued_invocations(
        queued: &mut Vec<QueuedRpcInvocation>,
        max_invocations: usize,
    ) -> Vec<QueuedRpcInvocation> {
        queued.sort_by(|left, right| {
            right
                .invocation
                .priority
                .cmp(&left.invocation.priority)
                .then_with(|| left.sequence.cmp(&right.sequence))
        });
        let take = max_invocations.min(queued.len());
        queued.drain(0..take).collect()
    }
}

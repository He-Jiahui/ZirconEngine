use std::time::{Duration, Instant};

use crate::core::framework::platform::{
    EventLoopBackgroundPolicy, EventLoopHostWakeReason, EventLoopWakeSource,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EventLoopHostWakeEvidence {
    reason: EventLoopHostWakeReason,
    observed_at: Instant,
}

impl EventLoopHostWakeEvidence {
    pub(super) const fn new(reason: EventLoopHostWakeReason, observed_at: Instant) -> Self {
        Self {
            reason,
            observed_at,
        }
    }

    pub(crate) const fn reason(self) -> EventLoopHostWakeReason {
        self.reason
    }

    pub(crate) const fn observed_at(self) -> Instant {
        self.observed_at
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EventLoopWakeDispatchEvidence {
    source: EventLoopWakeSource,
    deadline: Instant,
    dispatched_at: Instant,
    lateness: Duration,
}

impl EventLoopWakeDispatchEvidence {
    pub(super) const fn new(
        source: EventLoopWakeSource,
        deadline: Instant,
        dispatched_at: Instant,
        lateness: Duration,
    ) -> Self {
        Self {
            source,
            deadline,
            dispatched_at,
            lateness,
        }
    }

    pub(crate) const fn source(self) -> EventLoopWakeSource {
        self.source
    }

    pub(crate) const fn deadline(self) -> Instant {
        self.deadline
    }

    pub(crate) const fn dispatched_at(self) -> Instant {
        self.dispatched_at
    }

    pub(crate) const fn lateness(self) -> Duration {
        self.lateness
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EventLoopSchedulerSnapshot {
    pending_sources: usize,
    pending_deadlines: [Option<Instant>; EventLoopWakeSource::COUNT],
    backlog: usize,
    backlogs: [usize; EventLoopWakeSource::COUNT],
    backlog_high_watermark: usize,
    replaced_requests: u64,
    dispatched_wakes: u64,
    overdue_dispatches: u64,
    maximum_lateness: Duration,
    last_dispatches: [Option<EventLoopWakeDispatchEvidence>; EventLoopWakeSource::COUNT],
    starvation_preventions: u64,
    background_policy: Option<EventLoopBackgroundPolicy>,
    background_policy_transitions: u64,
    host_wake_counts: [u64; EventLoopHostWakeReason::COUNT],
    last_host_wake: Option<EventLoopHostWakeEvidence>,
}

impl EventLoopSchedulerSnapshot {
    pub(crate) const fn pending_sources(self) -> usize {
        self.pending_sources
    }
    pub(crate) const fn pending_deadline(self, source: EventLoopWakeSource) -> Option<Instant> {
        self.pending_deadlines[source.index()]
    }
    pub(crate) const fn backlog(self) -> usize {
        self.backlog
    }
    pub(crate) const fn backlog_for(self, source: EventLoopWakeSource) -> usize {
        self.backlogs[source.index()]
    }
    pub(crate) const fn backlog_high_watermark(self) -> usize {
        self.backlog_high_watermark
    }
    pub(crate) const fn replaced_requests(self) -> u64 {
        self.replaced_requests
    }
    pub(crate) const fn dispatched_wakes(self) -> u64 {
        self.dispatched_wakes
    }
    pub(crate) const fn overdue_dispatches(self) -> u64 {
        self.overdue_dispatches
    }
    pub(crate) const fn maximum_lateness(self) -> Duration {
        self.maximum_lateness
    }
    pub(crate) const fn last_dispatch(
        self,
        source: EventLoopWakeSource,
    ) -> Option<EventLoopWakeDispatchEvidence> {
        self.last_dispatches[source.index()]
    }
    pub(crate) const fn starvation_preventions(self) -> u64 {
        self.starvation_preventions
    }
    pub(crate) const fn background_policy(self) -> Option<EventLoopBackgroundPolicy> {
        self.background_policy
    }
    pub(crate) const fn background_policy_transitions(self) -> u64 {
        self.background_policy_transitions
    }
    pub(crate) const fn host_wake_count(self, reason: EventLoopHostWakeReason) -> u64 {
        self.host_wake_counts[reason.index()]
    }
    pub(crate) const fn last_host_wake(self) -> Option<EventLoopHostWakeEvidence> {
        self.last_host_wake
    }

    pub(super) fn set_pending_deadlines(
        &mut self,
        pending_deadlines: [Option<Instant>; EventLoopWakeSource::COUNT],
    ) {
        self.pending_sources = pending_deadlines.iter().flatten().count();
        self.pending_deadlines = pending_deadlines;
    }
    pub(super) fn set_backlog(&mut self, source: EventLoopWakeSource, value: usize) {
        self.backlogs[source.index()] = value;
        self.backlog = self
            .backlogs
            .iter()
            .copied()
            .fold(0usize, usize::saturating_add);
        self.backlog_high_watermark = self.backlog_high_watermark.max(self.backlog);
    }
    pub(super) fn record_replacement(&mut self) {
        self.replaced_requests = self.replaced_requests.saturating_add(1);
    }
    pub(super) fn record_dispatch(&mut self, evidence: EventLoopWakeDispatchEvidence) {
        self.dispatched_wakes = self.dispatched_wakes.saturating_add(1);
        if !evidence.lateness.is_zero() {
            self.overdue_dispatches = self.overdue_dispatches.saturating_add(1);
            self.maximum_lateness = self.maximum_lateness.max(evidence.lateness);
        }
        self.last_dispatches[evidence.source.index()] = Some(evidence);
    }
    pub(super) fn record_starvation_prevention(&mut self, count: usize) {
        self.starvation_preventions = self.starvation_preventions.saturating_add(count as u64);
    }
    pub(super) fn observe_background_policy(&mut self, policy: EventLoopBackgroundPolicy) {
        if self.background_policy != Some(policy) {
            self.background_policy = Some(policy);
            self.background_policy_transitions =
                self.background_policy_transitions.saturating_add(1);
        }
    }
    pub(super) fn record_host_wake(&mut self, evidence: EventLoopHostWakeEvidence) {
        let count = &mut self.host_wake_counts[evidence.reason.index()];
        *count = count.saturating_add(1);
        self.last_host_wake = Some(evidence);
    }
}

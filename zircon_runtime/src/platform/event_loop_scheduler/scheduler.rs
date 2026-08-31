use std::time::Instant;

use crate::core::framework::platform::{
    EventLoopBackgroundPolicy, EventLoopClockDomain, EventLoopControlFlow, EventLoopHostWakeReason,
    EventLoopWakeRequest, EventLoopWakeSource,
};

use super::{EventLoopHostWakeEvidence, EventLoopSchedulerSnapshot, EventLoopWakeDispatchEvidence};

/// Bounded platform-host wake selector. Sources own their payload queues and
/// submit only the current deadline, so this fixed-size state does not need a
/// heap, stale heap entries, or a cross-domain queue scan.
pub(crate) struct EventLoopScheduler {
    deadlines: [Option<Instant>; EventLoopWakeSource::COUNT],
    snapshot: EventLoopSchedulerSnapshot,
}

impl EventLoopScheduler {
    pub(crate) fn schedule(&mut self, request: EventLoopWakeRequest) {
        debug_assert_eq!(request.domain(), EventLoopClockDomain::Monotonic);
        let slot = &mut self.deadlines[request.source().index()];
        if slot.replace(request.deadline()).is_some() {
            self.snapshot.record_replacement();
        }
        self.refresh_pending_deadlines();
    }

    pub(crate) fn observe_backlog(&mut self, source: EventLoopWakeSource, backlog: usize) {
        self.snapshot.set_backlog(source, backlog);
    }

    pub(crate) fn observe_background_policy(&mut self, policy: EventLoopBackgroundPolicy) {
        self.snapshot.observe_background_policy(policy);
    }

    pub(crate) fn observe_host_wake(
        &mut self,
        reason: EventLoopHostWakeReason,
        observed_at: Instant,
    ) {
        self.snapshot
            .record_host_wake(EventLoopHostWakeEvidence::new(reason, observed_at));
    }

    pub(crate) fn control_flow(&self, now: Instant) -> EventLoopControlFlow {
        let Some(deadline) = self.earliest_deadline() else {
            return EventLoopControlFlow::Wait;
        };
        if deadline <= now {
            EventLoopControlFlow::Poll
        } else {
            EventLoopControlFlow::WaitUntil {
                domain: EventLoopClockDomain::Monotonic,
                deadline,
            }
        }
    }

    pub(crate) fn take_due(&mut self, now: Instant) -> EventLoopDueSources {
        let mut due = EventLoopDueSources::default();
        for (index, deadline) in self.deadlines.iter_mut().enumerate() {
            let Some(scheduled) = *deadline else {
                continue;
            };
            if scheduled <= now {
                *deadline = None;
                due.insert_index(index);
                let source = EventLoopWakeSource::ALL[index];
                self.snapshot
                    .record_dispatch(EventLoopWakeDispatchEvidence::new(
                        source,
                        scheduled,
                        now,
                        now.duration_since(scheduled),
                    ));
            }
        }
        // A selector pass emits every due source together. The extra sources
        // prove it did not postpone them behind the first ready source; they
        // do not imply that their owner payloads have finished executing.
        self.snapshot
            .record_starvation_prevention(due.count().saturating_sub(1));
        self.refresh_pending_deadlines();
        due
    }

    pub(crate) const fn snapshot(&self) -> EventLoopSchedulerSnapshot {
        self.snapshot
    }

    fn earliest_deadline(&self) -> Option<Instant> {
        self.deadlines.iter().flatten().copied().min()
    }

    fn refresh_pending_deadlines(&mut self) {
        self.snapshot.set_pending_deadlines(self.deadlines);
    }
}

impl Default for EventLoopScheduler {
    fn default() -> Self {
        Self {
            deadlines: [None; EventLoopWakeSource::COUNT],
            snapshot: EventLoopSchedulerSnapshot::default(),
        }
    }
}

/// Bitset of sources due in one platform-loop pass. All due sources are
/// returned together, preventing scheduler-level starvation by construction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EventLoopDueSources(u8);

impl EventLoopDueSources {
    pub(crate) const fn contains(self, source: EventLoopWakeSource) -> bool {
        self.0 & (1 << source.index()) != 0
    }

    pub(crate) const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    fn insert_index(&mut self, index: usize) {
        debug_assert!(index < EventLoopWakeSource::COUNT);
        self.0 |= 1 << index;
    }
}

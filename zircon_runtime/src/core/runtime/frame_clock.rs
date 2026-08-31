//! Monotonic frame-delta timing.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::clock_source::{ClockSource, FrameClockSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameClockFirstTickPolicy {
    MeasureFromRebase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockLifecycleTransition {
    Foregrounded,
    Backgrounded,
    Suspended,
    Resumed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockDiscontinuity {
    ApplicationLifecycle(ClockLifecycleTransition),
    WindowOcclusionChanged { occluded: bool },
    WindowSurfaceRecreated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameClockRebaseCause {
    Manual,
    SessionActivationCompleted,
    ClockDiscontinuity(ClockDiscontinuity),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameClockRebaseReceipt {
    generation: u64,
    first_tick_policy: FrameClockFirstTickPolicy,
    cause: FrameClockRebaseCause,
}

impl FrameClockRebaseReceipt {
    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn first_tick_policy(self) -> FrameClockFirstTickPolicy {
        self.first_tick_policy
    }

    pub const fn cause(self) -> FrameClockRebaseCause {
        self.cause
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameClockTick {
    delta: Duration,
    rebase: Option<FrameClockRebaseReceipt>,
}

impl FrameClockTick {
    pub(crate) const fn delta(self) -> Duration {
        self.delta
    }

    pub(crate) const fn rebase(self) -> Option<FrameClockRebaseReceipt> {
        self.rebase
    }
}

#[derive(Debug, Clone)]
pub struct FrameClock {
    source: FrameClockSource,
    last_tick: Instant,
    rebase_generation: u64,
    pending_rebase: Option<FrameClockRebaseReceipt>,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::with_source(FrameClockSource::system_monotonic())
    }
}

impl FrameClock {
    /// Creates a frame clock driven by an explicitly owned monotonic source.
    ///
    /// Production construction uses [`Default`], which retains the direct OS
    /// monotonic fast path. Injected sources exist for deterministic tests and
    /// future replay ownership.
    pub fn with_clock_source(source: Arc<dyn ClockSource>) -> Self {
        Self::with_source(FrameClockSource::injected(source))
    }

    fn with_source(source: FrameClockSource) -> Self {
        let last_tick = source.monotonic_now();
        Self {
            source,
            last_tick,
            rebase_generation: 0,
            pending_rebase: None,
        }
    }

    pub fn rebase(&mut self) -> FrameClockRebaseReceipt {
        self.rebase_for(FrameClockRebaseCause::Manual)
    }

    pub(crate) fn rebase_for(&mut self, cause: FrameClockRebaseCause) -> FrameClockRebaseReceipt {
        self.last_tick = self.source.monotonic_now();
        self.rebase_generation = self.rebase_generation.saturating_add(1);
        let receipt = FrameClockRebaseReceipt {
            generation: self.rebase_generation,
            first_tick_policy: FrameClockFirstTickPolicy::MeasureFromRebase,
            cause,
        };
        self.pending_rebase = Some(receipt);
        receipt
    }

    pub(crate) fn tick(&mut self) -> FrameClockTick {
        let now = self.source.monotonic_now();
        let delta = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        FrameClockTick {
            delta,
            rebase: self.pending_rebase.take(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::super::clock_source::ManualClockSource;
    use super::{FrameClock, FrameClockFirstTickPolicy, FrameClockRebaseCause};

    #[test]
    fn rebase_issues_a_monotonic_baseline_receipt() {
        let mut clock = FrameClock::default();

        let first = clock.rebase();
        let second = clock.rebase();

        assert_eq!(first.generation(), 1);
        assert_eq!(second.generation(), 2);
        assert_eq!(
            second.first_tick_policy(),
            FrameClockFirstTickPolicy::MeasureFromRebase
        );
        assert_eq!(second.cause(), FrameClockRebaseCause::Manual);
        assert_eq!(clock.tick().rebase(), Some(second));
        assert_eq!(clock.tick().rebase(), None);
    }

    #[test]
    fn injected_clock_source_drives_tick_and_rebase_without_sleeping() {
        let source = Arc::new(ManualClockSource::with_origin(Instant::now()));
        let mut clock = FrameClock::with_clock_source(source.clone());

        source
            .try_advance_by(Duration::from_millis(16))
            .expect("manual source should advance");
        assert_eq!(clock.tick().delta(), Duration::from_millis(16));

        let receipt = clock.rebase();
        source
            .try_advance_by(Duration::from_millis(8))
            .expect("manual source should advance after rebase");
        let rebased = clock.tick();

        assert_eq!(rebased.delta(), Duration::from_millis(8));
        assert_eq!(rebased.rebase(), Some(receipt));
    }
}

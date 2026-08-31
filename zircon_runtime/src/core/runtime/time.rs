use std::time::Duration;

use crate::core::framework::time::{
    ClockDomainId, ClockDomainStamp, MonotonicReal, Time, TimePolicy, TimePolicyError,
    TimePolicyTransaction,
};

use super::frame_clock::FrameClockRebaseReceipt;

mod product_policy;

pub use product_policy::{ProductTimePolicies, ProductTimePolicyDigest};

/// Diagnostic path for total real-time frames advanced by the runtime.
pub const TIME_FRAME_COUNT_DIAGNOSTIC: &str = "time.frame_count";
/// Diagnostic path for real frame duration measured in milliseconds.
pub const TIME_FRAME_TIME_DIAGNOSTIC: &str = "time.frame_time";
/// Diagnostic path for frames per second derived from real frame duration.
pub const TIME_FPS_DIAGNOSTIC: &str = "time.fps";

/// Core-owned outer-frame clock and defaults for subsequently created Worlds.
///
/// Virtual/fixed simulation time is deliberately absent: each `LevelSystem`
/// owns those derived clocks, their debt, and their commit boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RuntimeTimeAuthority {
    real: Time<MonotonicReal>,
    default_world_time_policy: TimePolicy,
    policy_generation: u64,
}

impl Default for RuntimeTimeAuthority {
    fn default() -> Self {
        Self {
            real: Time::<MonotonicReal>::default(),
            default_world_time_policy: TimePolicy::default(),
            policy_generation: 0,
        }
    }
}

impl RuntimeTimeAuthority {
    pub(crate) const fn real(&self) -> Time<MonotonicReal> {
        self.real
    }

    pub(crate) const fn time_policy(&self) -> TimePolicy {
        self.default_world_time_policy
    }

    pub(crate) const fn time_policy_generation(&self) -> u64 {
        self.policy_generation
    }

    pub(crate) fn apply_time_policy(
        &mut self,
        transaction: TimePolicyTransaction,
    ) -> Result<TimePolicyReceipt, TimePolicyError> {
        let applied = transaction.prepare()?;
        let previous = self.default_world_time_policy;
        let changed = applied != previous;

        if changed {
            self.default_world_time_policy = applied;
            self.policy_generation = self.policy_generation.saturating_add(1);
        }

        Ok(TimePolicyReceipt {
            previous,
            applied,
            generation: self.policy_generation,
            changed,
        })
    }

    pub(crate) fn advance_by_with_discontinuity(
        &mut self,
        raw_real_delta: Duration,
        fixed_step_budget: u32,
        discontinuity: Option<FrameTimeDiscontinuity>,
    ) -> FrameTimeSnapshot {
        if let Some(FrameTimeDiscontinuity::FrameClockRebased(receipt)) = discontinuity {
            self.real
                .set_clock_domain_source_generation(receipt.generation());
        }
        self.real.advance_by(raw_real_delta);

        FrameTimeSnapshot {
            outer_frame_index: self.real.frame_index(),
            raw_real_delta,
            real_elapsed: self.real.elapsed(),
            fixed_step_budget,
            discontinuity,
            real_clock_domain_stamp: self.real.clock_domain_stamp(),
        }
    }
}

/// Immutable evidence for an accepted default World time-policy transaction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimePolicyReceipt {
    previous: TimePolicy,
    applied: TimePolicy,
    generation: u64,
    changed: bool,
}

impl TimePolicyReceipt {
    pub const fn previous(self) -> TimePolicy {
        self.previous
    }

    pub const fn applied(self) -> TimePolicy {
        self.applied
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn changed(self) -> bool {
        self.changed
    }
}

/// Typed discontinuity observed while preparing one outer-frame time snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameTimeDiscontinuity {
    FrameClockRebased(FrameClockRebaseReceipt),
}

/// Immutable real-time input and fixed-step budget for one outer frame.
///
/// World-local virtual/fixed delta, pause state, debt, and clock stamps are
/// materialized later as `WorldTimeSnapshot` by the owning `LevelSystem`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameTimeSnapshot {
    outer_frame_index: u64,
    raw_real_delta: Duration,
    real_elapsed: Duration,
    fixed_step_budget: u32,
    discontinuity: Option<FrameTimeDiscontinuity>,
    real_clock_domain_stamp: ClockDomainStamp,
}

impl FrameTimeSnapshot {
    pub const fn outer_frame_index(self) -> u64 {
        self.outer_frame_index
    }

    pub const fn raw_real_delta(self) -> Duration {
        self.raw_real_delta
    }

    /// Monotonic-real elapsed time captured atomically with this outer frame.
    pub const fn real_elapsed(self) -> Duration {
        self.real_elapsed
    }

    /// Maximum fixed steps the outer-frame owner permits each World to commit.
    pub const fn fixed_step_budget(self) -> u32 {
        self.fixed_step_budget
    }

    pub const fn discontinuity(self) -> Option<FrameTimeDiscontinuity> {
        self.discontinuity
    }

    /// The shared monotonic source stamp for the outer frame.
    pub const fn real_clock_domain_stamp(self) -> ClockDomainStamp {
        self.real_clock_domain_stamp
    }

    /// Only the monotonic-real domain is owned by this outer-frame snapshot.
    pub const fn clock_domain_stamp(self, domain: ClockDomainId) -> Option<ClockDomainStamp> {
        match domain {
            ClockDomainId::MonotonicReal => Some(self.real_clock_domain_stamp),
            ClockDomainId::WorldVirtual
            | ClockDomainId::WorldFixed
            | ClockDomainId::WallUtc
            | ClockDomainId::Input
            | ClockDomainId::Render
            | ClockDomainId::Audio
            | ClockDomainId::Network
            | ClockDomainId::Media
            | ClockDomainId::EditorPreview => None,
        }
    }
}

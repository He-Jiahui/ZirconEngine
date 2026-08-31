use std::time::Duration;

use crate::core::framework::time::{ClockDomainId, ClockDomainStamp, FixedStepPlan};
use crate::core::{FrameTimeDiscontinuity, FrameTimeSnapshot};

/// Immutable virtual/fixed timing work accepted by one World for one outer frame.
///
/// The outer frame's monotonic time remains owned by `CoreRuntime`; this value
/// owns the World-local derived clocks, pause state, and fixed-step debt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTimeSnapshot {
    outer_frame_index: u64,
    raw_real_delta: Duration,
    real_elapsed: Duration,
    virtual_delta: Duration,
    virtual_elapsed: Duration,
    virtual_time_paused: bool,
    virtual_effective_speed: f64,
    time_policy_generation: u64,
    discontinuity: Option<FrameTimeDiscontinuity>,
    fixed_step_plan: FixedStepPlan,
    real_clock_domain_stamp: ClockDomainStamp,
    virtual_clock_domain_stamp: ClockDomainStamp,
    fixed_clock_domain_stamp: ClockDomainStamp,
}

impl WorldTimeSnapshot {
    pub(crate) fn new(
        outer: FrameTimeSnapshot,
        virtual_delta: Duration,
        virtual_elapsed: Duration,
        virtual_time_paused: bool,
        virtual_effective_speed: f64,
        time_policy_generation: u64,
        fixed_step_plan: FixedStepPlan,
        virtual_clock_domain_stamp: ClockDomainStamp,
        fixed_clock_domain_stamp: ClockDomainStamp,
    ) -> Self {
        Self {
            outer_frame_index: outer.outer_frame_index(),
            raw_real_delta: outer.raw_real_delta(),
            real_elapsed: outer.real_elapsed(),
            virtual_delta,
            virtual_elapsed,
            virtual_time_paused,
            virtual_effective_speed,
            time_policy_generation,
            discontinuity: outer.discontinuity(),
            fixed_step_plan,
            real_clock_domain_stamp: outer.real_clock_domain_stamp(),
            virtual_clock_domain_stamp,
            fixed_clock_domain_stamp,
        }
    }

    pub const fn outer_frame_index(self) -> u64 {
        self.outer_frame_index
    }

    pub const fn raw_real_delta(self) -> Duration {
        self.raw_real_delta
    }

    pub const fn real_elapsed(self) -> Duration {
        self.real_elapsed
    }

    pub const fn virtual_delta(self) -> Duration {
        self.virtual_delta
    }

    pub const fn virtual_elapsed(self) -> Duration {
        self.virtual_elapsed
    }

    pub const fn virtual_time_paused(self) -> bool {
        self.virtual_time_paused
    }

    pub const fn virtual_effective_speed(self) -> f64 {
        self.virtual_effective_speed
    }

    pub const fn time_policy_generation(self) -> u64 {
        self.time_policy_generation
    }

    pub const fn discontinuity(self) -> Option<FrameTimeDiscontinuity> {
        self.discontinuity
    }

    pub const fn fixed_step_plan(self) -> FixedStepPlan {
        self.fixed_step_plan
    }

    pub const fn real_clock_domain_stamp(self) -> ClockDomainStamp {
        self.real_clock_domain_stamp
    }

    pub const fn clock_domain_stamp(self, domain: ClockDomainId) -> Option<ClockDomainStamp> {
        match domain {
            ClockDomainId::MonotonicReal => Some(self.real_clock_domain_stamp),
            ClockDomainId::WorldVirtual => Some(self.virtual_clock_domain_stamp),
            ClockDomainId::WorldFixed => Some(self.fixed_clock_domain_stamp),
            ClockDomainId::WallUtc
            | ClockDomainId::Input
            | ClockDomainId::Render
            | ClockDomainId::Audio
            | ClockDomainId::Network
            | ClockDomainId::Media
            | ClockDomainId::EditorPreview => None,
        }
    }
}

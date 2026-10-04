use std::time::Duration;

use crate::core::framework::time::{ClockDomainId, ClockDomainStamp};
use crate::scene::ecs::{SceneSystemClockDomain, SystemStage};
use crate::scene::world_time::SimulationTickId;

/// Immutable clock evidence for one system invocation.
///
/// The World driver constructs this after selecting the current Level-owned
/// clock snapshot. Systems can retain or copy the value for diagnostics, but
/// cannot manufacture a tick that bypasses the frame and fixed-step owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemTickContext {
    stage: SystemStage,
    clock_domain: ClockDomainStamp,
    outer_frame_index: u64,
    simulation_tick: Option<SimulationTickId>,
    delta: Duration,
    elapsed: Duration,
    world_generation: u64,
}

impl SystemTickContext {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        stage: SystemStage,
        clock_domain: ClockDomainStamp,
        outer_frame_index: u64,
        simulation_tick: Option<SimulationTickId>,
        delta: Duration,
        elapsed: Duration,
        world_generation: u64,
    ) -> Self {
        Self {
            stage,
            clock_domain,
            outer_frame_index,
            simulation_tick,
            delta,
            elapsed,
            world_generation,
        }
    }

    pub const fn stage(self) -> SystemStage {
        self.stage
    }

    pub const fn clock_domain(self) -> ClockDomainId {
        self.clock_domain.id()
    }

    pub const fn clock_domain_stamp(self) -> ClockDomainStamp {
        self.clock_domain
    }

    pub const fn outer_frame_index(self) -> u64 {
        self.outer_frame_index
    }

    /// Returns the active fixed-step identity for a fixed-loop system.
    pub const fn simulation_tick(self) -> Option<SimulationTickId> {
        self.simulation_tick
    }

    pub const fn delta(self) -> Duration {
        self.delta
    }

    /// Float projection for system and script callbacks that consume seconds.
    ///
    /// New scheduling code should retain `Duration`; this conversion is kept
    /// at the typed tick boundary so callback consumers cannot choose a clock
    /// source or reconstruct the current frame themselves.
    pub fn delta_seconds(self) -> crate::core::math::Real {
        self.delta.as_secs_f64() as crate::core::math::Real
    }

    pub const fn elapsed(self) -> Duration {
        self.elapsed
    }

    pub const fn world_generation(self) -> u64 {
        self.world_generation
    }
}

/// Per-stage clock contexts selected once by the World driver.
///
/// This remains crate-private because choosing a clock for a system is a
/// schedule-owner responsibility, not a plugin capability.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SceneStageTickContexts {
    virtual_time: SystemTickContext,
    monotonic_real: SystemTickContext,
    fixed: SystemTickContext,
}

impl SceneStageTickContexts {
    pub(crate) const fn new(
        virtual_time: SystemTickContext,
        monotonic_real: SystemTickContext,
        fixed: SystemTickContext,
    ) -> Self {
        Self {
            virtual_time,
            monotonic_real,
            fixed,
        }
    }

    pub(crate) const fn for_domain(self, domain: SceneSystemClockDomain) -> SystemTickContext {
        // 上层只选择预先构造的上下文，不在系统调用期间重新计算时钟或 fixed step。
        match domain {
            SceneSystemClockDomain::Virtual => self.virtual_time,
            SceneSystemClockDomain::MonotonicReal => self.monotonic_real,
            SceneSystemClockDomain::Fixed => self.fixed,
        }
    }
}

#[cfg(test)]
#[path = "tests/tick_context.rs"]
mod tests;

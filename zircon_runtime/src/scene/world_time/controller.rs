use crate::core::framework::time::{
    Fixed, Time, TimePolicy, TimePolicyError, TimePolicyTransaction, Virtual,
};
use crate::core::FrameTimeSnapshot;
use thiserror::Error;

use super::{
    FixedInterpolationContext, FixedInterpolationState, SimulationTickId, WorldFixedStep,
    WorldFixedStepError, WorldTimeSnapshot,
};

/// Mutable timing authority owned by exactly one `LevelSystem`.
///
/// It derives virtual and fixed time from the shared monotonic outer-frame
/// input, without sharing pause state, rate, or fixed debt with another World.
#[derive(Debug, PartialEq)]
pub(crate) struct WorldTimeController {
    virtual_time: Time<Virtual>,
    fixed_time: Time<Fixed>,
    policy_generation: u64,
    fixed_step_budget_remaining: u32,
    active_fixed_step: Option<SimulationTickId>,
    previous_committed_fixed_step: Option<SimulationTickId>,
    current_committed_fixed_step: Option<SimulationTickId>,
    last_outer_frame_index: Option<u64>,
    single_step_requested: bool,
}

impl Default for WorldTimeController {
    fn default() -> Self {
        Self::new(TimePolicy::default()).expect("default world time policy is valid")
    }
}

impl WorldTimeController {
    pub fn new(policy: TimePolicy) -> Result<Self, TimePolicyError> {
        policy.validate()?;
        let mut virtual_time = Time::default();
        virtual_time.set_max_delta(policy.virtual_max_delta());
        virtual_time.set_relative_speed_f64(policy.virtual_relative_speed());
        let mut fixed_time = Time::default();
        fixed_time.set_timestep(policy.fixed_timestep());
        Ok(Self {
            virtual_time,
            fixed_time,
            policy_generation: 0,
            fixed_step_budget_remaining: 0,
            active_fixed_step: None,
            previous_committed_fixed_step: None,
            current_committed_fixed_step: None,
            last_outer_frame_index: None,
            single_step_requested: false,
        })
    }

    pub fn state(&self) -> WorldTimeState {
        WorldTimeState {
            virtual_time: self.virtual_time,
            fixed_time: self.fixed_time,
            policy_generation: self.policy_generation,
            last_outer_frame_index: self.last_outer_frame_index,
        }
    }

    pub fn time_policy(&self) -> TimePolicy {
        TimePolicy::new(
            self.virtual_time.max_delta(),
            self.virtual_time.relative_speed_f64(),
            self.fixed_time.timestep(),
        )
    }

    pub fn apply_time_policy(
        &mut self,
        transaction: TimePolicyTransaction,
    ) -> Result<WorldTimePolicyReceipt, TimePolicyError> {
        let applied = transaction.prepare()?;
        if self.active_fixed_step.is_some() {
            return Err(TimePolicyError::FixedStepActive);
        }
        let previous = self.time_policy();
        if applied.fixed_timestep() != previous.fixed_timestep()
            && !self.fixed_time.overstep().is_zero()
        {
            return Err(TimePolicyError::FixedStepDebtPending {
                remaining: self.fixed_time.overstep(),
            });
        }
        let changed = applied != previous;
        if changed {
            let virtual_clock_changed = applied.virtual_max_delta() != previous.virtual_max_delta()
                || applied.virtual_relative_speed() != previous.virtual_relative_speed();
            let fixed_clock_changed = applied.fixed_timestep() != previous.fixed_timestep();
            if virtual_clock_changed {
                self.virtual_time.set_max_delta(applied.virtual_max_delta());
                self.virtual_time
                    .set_relative_speed_f64(applied.virtual_relative_speed());
                self.virtual_time.bump_clock_domain_epoch();
            }
            if fixed_clock_changed {
                self.fixed_time.set_timestep(applied.fixed_timestep());
                self.fixed_time.bump_clock_domain_epoch();
                self.reset_fixed_interpolation_history();
            }
            self.policy_generation = self.policy_generation.saturating_add(1);
        }
        Ok(WorldTimePolicyReceipt {
            previous,
            applied,
            generation: self.policy_generation,
            changed,
        })
    }

    pub fn pause_virtual_time(&mut self) {
        self.virtual_time.pause();
    }

    pub fn unpause_virtual_time(&mut self) {
        self.single_step_requested = false;
        self.virtual_time.unpause();
    }

    pub fn request_single_step(&mut self) -> Result<(), WorldTimeControlError> {
        if !self.virtual_time.is_paused() {
            return Err(WorldTimeControlError::SingleStepRequiresPause);
        }
        if self.active_fixed_step.is_some() {
            return Err(WorldTimeControlError::FixedStepActive);
        }
        if self.single_step_requested {
            return Err(WorldTimeControlError::SingleStepAlreadyRequested);
        }
        self.single_step_requested = true;
        Ok(())
    }

    pub fn advance(
        &mut self,
        outer: FrameTimeSnapshot,
    ) -> Result<WorldTimeSnapshot, WorldTimeAdvanceError> {
        self.validate_outer_frame(outer)?;
        debug_assert!(
            self.active_fixed_step.is_none(),
            "a World fixed step must commit or abort before the next outer frame"
        );
        let source_generation = outer.real_clock_domain_stamp().source_generation();
        self.virtual_time
            .set_clock_domain_source_generation(source_generation);
        self.fixed_time
            .set_clock_domain_source_generation(source_generation);
        self.virtual_time
            .advance_from_real_delta(outer.raw_real_delta());
        let virtual_time_paused = self.virtual_time.is_paused();
        let single_step_requested = self.single_step_requested;
        if single_step_requested {
            // A failed single-step leaves its debt pending; retry only tops up to one step.
            let required_debt = self
                .fixed_time
                .timestep()
                .saturating_sub(self.fixed_time.overstep());
            self.fixed_time.accumulate_overstep(required_debt);
            self.single_step_requested = false;
        }
        let fixed_step_plan = if virtual_time_paused && !single_step_requested {
            self.fixed_time.plan_steps(0)
        } else if single_step_requested {
            self.fixed_time.plan_steps(1)
        } else {
            self.fixed_time
                .accumulate_overstep(self.virtual_time.delta());
            self.fixed_time.plan_steps(outer.fixed_step_budget())
        };
        self.fixed_step_budget_remaining = fixed_step_plan.step_count;
        let snapshot = WorldTimeSnapshot::new(
            outer,
            self.virtual_time.delta(),
            self.virtual_time.elapsed(),
            virtual_time_paused,
            self.virtual_time.effective_speed_f64(),
            self.policy_generation,
            fixed_step_plan,
            self.virtual_time.clock_domain_stamp(),
            self.fixed_time.clock_domain_stamp(),
        );
        self.last_outer_frame_index = Some(outer.outer_frame_index());
        Ok(snapshot)
    }

    pub(crate) fn begin_fixed_step(
        &mut self,
        world_generation: u64,
    ) -> Result<WorldFixedStep, WorldFixedStepError> {
        if let Some(active) = self.active_fixed_step {
            return Err(WorldFixedStepError::ActiveStep { active });
        }
        if self.fixed_step_budget_remaining == 0 {
            return Err(WorldFixedStepError::BudgetExhausted);
        }
        if self.fixed_time.overstep() < self.fixed_time.timestep() {
            return Err(WorldFixedStepError::InsufficientDebt);
        }
        let id = SimulationTickId::new(
            world_generation,
            self.fixed_time.clock_domain_stamp().epoch(),
            self.fixed_time.frame_index().saturating_add(1),
        );
        let step = WorldFixedStep::new(
            id,
            self.fixed_time.timestep(),
            self.fixed_time
                .elapsed()
                .saturating_add(self.fixed_time.timestep()),
        );
        self.active_fixed_step = Some(id);
        Ok(step)
    }

    pub(crate) fn commit_fixed_step(
        &mut self,
        step: &WorldFixedStep,
    ) -> Result<(), WorldFixedStepError> {
        self.validate_active_step(step)?;
        if !self.fixed_time.try_commit_step() {
            return Err(WorldFixedStepError::InsufficientDebt);
        }
        self.active_fixed_step = None;
        self.fixed_step_budget_remaining = self.fixed_step_budget_remaining.saturating_sub(1);
        self.previous_committed_fixed_step = self.current_committed_fixed_step;
        self.current_committed_fixed_step = Some(step.id());
        Ok(())
    }

    pub(crate) fn abort_fixed_step(
        &mut self,
        step: WorldFixedStep,
    ) -> Result<(), WorldFixedStepError> {
        self.validate_active_step(&step)?;
        self.active_fixed_step = None;
        Ok(())
    }

    pub(crate) fn fixed_interpolation_context(&self) -> FixedInterpolationContext {
        let current_elapsed = self.fixed_time.elapsed();
        let previous_elapsed = if self.current_committed_fixed_step.is_some() {
            current_elapsed.saturating_sub(self.fixed_time.timestep())
        } else {
            current_elapsed
        };
        let remaining_debt = self.fixed_time.overstep();
        FixedInterpolationContext::new(
            FixedInterpolationState::new(self.previous_committed_fixed_step, previous_elapsed),
            FixedInterpolationState::new(self.current_committed_fixed_step, current_elapsed),
            remaining_debt,
            self.fixed_time.timestep(),
            interpolation_fraction(remaining_debt, self.fixed_time.timestep()),
        )
    }

    pub(crate) fn reset_fixed_interpolation_history(&mut self) {
        self.previous_committed_fixed_step = None;
        self.current_committed_fixed_step = None;
    }

    pub(crate) fn reset_after_world_replacement(&mut self) {
        self.reset_fixed_interpolation_history();
        self.single_step_requested = false;
    }

    fn validate_active_step(&self, step: &WorldFixedStep) -> Result<(), WorldFixedStepError> {
        match self.active_fixed_step {
            Some(active) if active == step.id() => Ok(()),
            Some(active) => Err(WorldFixedStepError::ActiveStepMismatch {
                active,
                submitted: step.id(),
            }),
            None => Err(WorldFixedStepError::NoActiveStep {
                submitted: step.id(),
            }),
        }
    }

    fn validate_outer_frame(&self, outer: FrameTimeSnapshot) -> Result<(), WorldTimeAdvanceError> {
        let submitted = outer.outer_frame_index();
        match self.last_outer_frame_index {
            Some(frame_index) if submitted == frame_index => {
                Err(WorldTimeAdvanceError::DuplicateOuterFrame { frame_index })
            }
            Some(last_consumed) if submitted < last_consumed => {
                Err(WorldTimeAdvanceError::OutOfOrderOuterFrame {
                    last_consumed,
                    submitted,
                })
            }
            Some(last_consumed)
                if last_consumed
                    .checked_add(1)
                    .is_some_and(|expected| submitted > expected)
                    && outer.discontinuity().is_none() =>
            {
                Err(WorldTimeAdvanceError::SkippedOuterFrames {
                    last_consumed,
                    submitted,
                })
            }
            _ => Ok(()),
        }
    }
}

/// Rejection returned before a World mutates time for an invalid outer-frame handoff.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum WorldTimeAdvanceError {
    #[error("outer frame {frame_index} was already consumed by this World")]
    DuplicateOuterFrame { frame_index: u64 },
    #[error(
        "outer frame {submitted} is older than this World's last consumed frame {last_consumed}"
    )]
    OutOfOrderOuterFrame { last_consumed: u64, submitted: u64 },
    #[error(
        "outer frame {submitted} skipped one or more frames after {last_consumed} without a discontinuity"
    )]
    SkippedOuterFrames { last_consumed: u64, submitted: u64 },
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum WorldTimeControlError {
    #[error("single-step requires paused virtual time")]
    SingleStepRequiresPause,
    #[error("cannot request single-step while a fixed step is active")]
    FixedStepActive,
    #[error("a single-step request is already pending")]
    SingleStepAlreadyRequested,
}

fn interpolation_fraction(
    remaining_debt: std::time::Duration,
    timestep: std::time::Duration,
) -> f32 {
    if timestep.is_zero() {
        return 0.0;
    }
    let timestep_nanos = timestep.as_nanos();
    let remainder_nanos = remaining_debt.as_nanos() % timestep_nanos;
    (remainder_nanos as f64 / timestep_nanos as f64) as f32
}

/// Copyable observation of a World's mutable timing state outside its lock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTimeState {
    virtual_time: Time<Virtual>,
    fixed_time: Time<Fixed>,
    policy_generation: u64,
    last_outer_frame_index: Option<u64>,
}

impl WorldTimeState {
    pub const fn virtual_time(self) -> Time<Virtual> {
        self.virtual_time
    }

    pub const fn fixed_time(self) -> Time<Fixed> {
        self.fixed_time
    }

    pub const fn policy_generation(self) -> u64 {
        self.policy_generation
    }

    pub const fn last_outer_frame_index(self) -> Option<u64> {
        self.last_outer_frame_index
    }
}

/// Immutable receipt for a World-local time-policy transaction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTimePolicyReceipt {
    previous: TimePolicy,
    applied: TimePolicy,
    generation: u64,
    changed: bool,
}

impl WorldTimePolicyReceipt {
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

#[cfg(test)]
#[path = "tests/controller.rs"]
mod tests;

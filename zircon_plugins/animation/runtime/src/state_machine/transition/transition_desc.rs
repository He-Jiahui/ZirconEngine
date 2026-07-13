use zircon_runtime::core::math::Real;

use super::InterruptionPolicy;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionDesc {
    duration_seconds: Real,
    exit_time: Option<Real>,
    interruption: InterruptionPolicy,
}

impl TransitionDesc {
    pub fn new(duration_seconds: Real) -> Self {
        Self {
            duration_seconds: finite_non_negative(duration_seconds),
            exit_time: None,
            interruption: InterruptionPolicy::None,
        }
    }

    pub fn with_exit_time(mut self, normalized_exit_time: Real) -> Self {
        self.exit_time = normalized_exit_time
            .is_finite()
            .then(|| normalized_exit_time.clamp(0.0, 1.0));
        self
    }

    pub fn with_optional_exit_time(mut self, normalized_exit_time: Option<Real>) -> Self {
        self.exit_time = normalized_exit_time
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 1.0));
        self
    }

    pub fn with_interruption(mut self, interruption: InterruptionPolicy) -> Self {
        self.interruption = interruption;
        self
    }

    pub fn exit_ready(self, normalized_state_time: Real) -> bool {
        normalized_state_time.is_finite()
            && self
                .exit_time
                .is_none_or(|exit_time| normalized_state_time >= exit_time)
    }

    pub const fn duration_seconds(self) -> Real {
        self.duration_seconds
    }

    pub const fn exit_time(self) -> Option<Real> {
        self.exit_time
    }

    pub const fn interruption(self) -> InterruptionPolicy {
        self.interruption
    }
}

fn finite_non_negative(value: Real) -> Real {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

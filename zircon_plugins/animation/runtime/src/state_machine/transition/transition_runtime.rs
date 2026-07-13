use zircon_runtime::core::math::Real;

use super::{InterruptionPolicy, TransitionRequest, TransitionState, TransitionWeights};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionRuntime {
    request: TransitionRequest,
    elapsed_seconds: Real,
}

impl TransitionRuntime {
    pub fn begin(request: TransitionRequest, elapsed_seconds: Real) -> Self {
        Self {
            request,
            elapsed_seconds: finite_non_negative(elapsed_seconds)
                .min(request.desc.duration_seconds()),
        }
    }

    pub fn advance(&mut self, delta_seconds: Real) {
        self.elapsed_seconds = (self.elapsed_seconds + finite_non_negative(delta_seconds))
            .min(self.duration_seconds());
    }

    pub fn can_interrupt_from(self, requested_from: TransitionState) -> bool {
        match self.request.desc.interruption() {
            InterruptionPolicy::None => false,
            InterruptionPolicy::CurrentToNext => requested_from == self.request.from,
            InterruptionPolicy::NextToNext => requested_from == self.request.to,
            InterruptionPolicy::Both => {
                requested_from == self.request.from || requested_from == self.request.to
            }
        }
    }

    pub fn crossfade_weights(self) -> TransitionWeights {
        if self.duration_seconds() <= Real::EPSILON {
            return TransitionWeights::from_progress(1.0);
        }
        TransitionWeights::from_progress(self.elapsed_seconds / self.duration_seconds())
    }

    pub fn is_complete(self) -> bool {
        self.duration_seconds() <= Real::EPSILON || self.elapsed_seconds >= self.duration_seconds()
    }

    pub const fn duration_seconds(self) -> Real {
        self.request.desc.duration_seconds()
    }

    pub const fn elapsed_seconds(self) -> Real {
        self.elapsed_seconds
    }
}

fn finite_non_negative(value: Real) -> Real {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

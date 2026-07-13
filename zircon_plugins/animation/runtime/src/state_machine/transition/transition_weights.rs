use zircon_runtime::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionWeights {
    pub source: Real,
    pub target: Real,
}

impl TransitionWeights {
    pub(super) fn from_progress(progress: Real) -> Self {
        let target = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            1.0
        };
        Self {
            source: 1.0 - target,
            target,
        }
    }
}

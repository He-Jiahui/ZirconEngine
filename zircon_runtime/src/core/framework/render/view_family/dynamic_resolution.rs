use crate::core::math::Real;

use super::{resolution::normalize_fraction, RenderUpscalerKind};

/// A bounded dynamic-resolution controller driven by GPU frame time.
///
/// Raster cost is approximately proportional to pixel count, so the controller uses square-root
/// feedback to convert a time ratio into a linear resolution fraction. Hysteresis and a bounded
/// per-frame step prevent frame-time noise from causing visible scale oscillation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDynamicResolutionController {
    min_primary_fraction: Real,
    max_primary_fraction: Real,
    target_frame_time_ms: Real,
    max_fraction_step: Real,
    hysteresis_ms: Real,
}

impl RenderDynamicResolutionController {
    pub fn new(
        min_primary_fraction: Real,
        max_primary_fraction: Real,
        target_frame_time_ms: Real,
        max_fraction_step: Real,
        hysteresis_ms: Real,
    ) -> Self {
        let first_fraction = normalize_fraction(min_primary_fraction);
        let second_fraction = normalize_fraction(max_primary_fraction);
        Self {
            min_primary_fraction: first_fraction.min(second_fraction),
            max_primary_fraction: first_fraction.max(second_fraction),
            target_frame_time_ms: normalize_positive(target_frame_time_ms, 16.6),
            max_fraction_step: normalize_nonnegative(max_fraction_step).min(1.0),
            hysteresis_ms: normalize_nonnegative(hysteresis_ms),
        }
    }

    pub fn next_primary_fraction(
        self,
        current_primary_fraction: Real,
        gpu_frame_time_ms: Real,
    ) -> Real {
        let current =
            current_primary_fraction.clamp(self.min_primary_fraction, self.max_primary_fraction);
        if !gpu_frame_time_ms.is_finite() || gpu_frame_time_ms <= 0.0 {
            return current;
        }
        if (gpu_frame_time_ms - self.target_frame_time_ms).abs() <= self.hysteresis_ms {
            return current;
        }
        let desired = (current * (self.target_frame_time_ms / gpu_frame_time_ms).sqrt())
            .clamp(self.min_primary_fraction, self.max_primary_fraction);
        let bounded_delta =
            (desired - current).clamp(-self.max_fraction_step, self.max_fraction_step);
        (current + bounded_delta).clamp(self.min_primary_fraction, self.max_primary_fraction)
    }

    pub const fn primary_upper_bound(self) -> Real {
        self.max_primary_fraction
    }
}

/// Identifies the view-family state allowed to consume a GPU timing result.
///
/// A viewport generation changes on resize or device recreation. The reconstruction category is
/// part of the scope because changing it also changes temporal-history compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderDynamicResolutionScope {
    view_family_id: u64,
    viewport_generation: u64,
    upscaler: RenderUpscalerKind,
}

impl RenderDynamicResolutionScope {
    pub const fn new(
        view_family_id: u64,
        viewport_generation: u64,
        upscaler: RenderUpscalerKind,
    ) -> Self {
        Self {
            view_family_id,
            viewport_generation,
            upscaler,
        }
    }

    pub const fn view_family_id(self) -> u64 {
        self.view_family_id
    }

    pub const fn viewport_generation(self) -> u64 {
        self.viewport_generation
    }

    pub const fn upscaler(self) -> RenderUpscalerKind {
        self.upscaler
    }
}

/// A delayed timing result published by the render-device owner after submission completes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderDynamicResolutionGpuSample {
    Completed {
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
        gpu_frame_time_ms: Real,
    },
    Unavailable {
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    },
    TimedOut {
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    },
}

impl RenderDynamicResolutionGpuSample {
    pub const fn completed(
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
        gpu_frame_time_ms: Real,
    ) -> Self {
        Self::Completed {
            scope,
            source_frame_generation,
            gpu_frame_time_ms,
        }
    }

    pub const fn unavailable(
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    ) -> Self {
        Self::Unavailable {
            scope,
            source_frame_generation,
        }
    }

    pub const fn timed_out(
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    ) -> Self {
        Self::TimedOut {
            scope,
            source_frame_generation,
        }
    }

    pub const fn scope(self) -> RenderDynamicResolutionScope {
        match self {
            Self::Completed { scope, .. }
            | Self::Unavailable { scope, .. }
            | Self::TimedOut { scope, .. } => scope,
        }
    }

    pub const fn source_frame_generation(self) -> u64 {
        match self {
            Self::Completed {
                source_frame_generation,
                ..
            }
            | Self::Unavailable {
                source_frame_generation,
                ..
            }
            | Self::TimedOut {
                source_frame_generation,
                ..
            } => source_frame_generation,
        }
    }
}

/// Explains why a frame received its immutable dynamic-resolution decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderDynamicResolutionDecisionReason {
    Disabled,
    Unsupported,
    AwaitingGpuSample,
    CompletedGpuSample,
    UnavailableGpuSample,
    TimedOutGpuSample,
}

/// Immutable dynamic-resolution input to one ViewFamily plan resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDynamicResolutionDecision {
    scope: RenderDynamicResolutionScope,
    decision_generation: u64,
    source_frame_generation: Option<u64>,
    primary_fraction: Real,
    primary_upper_bound: Real,
    reason: RenderDynamicResolutionDecisionReason,
    requires_temporal_history_reset: bool,
}

impl RenderDynamicResolutionDecision {
    pub fn new(
        scope: RenderDynamicResolutionScope,
        decision_generation: u64,
        source_frame_generation: Option<u64>,
        primary_fraction: Real,
        primary_upper_bound: Real,
        reason: RenderDynamicResolutionDecisionReason,
        requires_temporal_history_reset: bool,
    ) -> Self {
        let primary_upper_bound = normalize_fraction(primary_upper_bound);
        Self {
            scope,
            decision_generation,
            source_frame_generation,
            primary_fraction: normalize_fraction(primary_fraction).min(primary_upper_bound),
            primary_upper_bound,
            reason,
            requires_temporal_history_reset,
        }
    }

    pub const fn scope(self) -> RenderDynamicResolutionScope {
        self.scope
    }

    pub const fn decision_generation(self) -> u64 {
        self.decision_generation
    }

    pub const fn source_frame_generation(self) -> Option<u64> {
        self.source_frame_generation
    }

    pub const fn primary_fraction(self) -> Real {
        self.primary_fraction
    }

    pub const fn primary_upper_bound(self) -> Real {
        self.primary_upper_bound
    }

    pub const fn reason(self) -> RenderDynamicResolutionDecisionReason {
        self.reason
    }

    pub const fn requires_temporal_history_reset(self) -> bool {
        self.requires_temporal_history_reset
    }
}

fn normalize_positive(value: Real, fallback: Real) -> Real {
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .unwrap_or(fallback)
}

fn normalize_nonnegative(value: Real) -> Real {
    (value.is_finite() && value >= 0.0)
        .then_some(value)
        .unwrap_or_default()
}

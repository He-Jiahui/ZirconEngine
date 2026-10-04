use crate::scene::ecs::SystemStage;

/// Runtime clock domain selected by a scene-system tick policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneSystemClockDomain {
    #[default]
    Virtual,
    MonotonicReal,
    Fixed,
}

/// Whether a system is eligible to run while virtual world time is paused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneSystemPauseBehavior {
    #[default]
    SkipWhenVirtualPaused,
    RunWhenVirtualPaused,
}

/// Declared timing and pause contract for a scheduled scene system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneSystemTickPolicy {
    clock_domain: SceneSystemClockDomain,
    pause_behavior: SceneSystemPauseBehavior,
}

impl SceneSystemTickPolicy {
    // stage 默认值是调度器的单一时钟契约；执行前用 is_valid_for_stage 拒绝跨域组合。
    pub const fn for_stage(stage: SystemStage) -> Self {
        if stage.is_fixed_loop() {
            Self::fixed()
        } else {
            Self::virtual_time()
        }
    }

    pub const fn virtual_time() -> Self {
        Self::new(
            SceneSystemClockDomain::Virtual,
            SceneSystemPauseBehavior::SkipWhenVirtualPaused,
        )
    }

    pub const fn monotonic_real() -> Self {
        Self::new(
            SceneSystemClockDomain::MonotonicReal,
            SceneSystemPauseBehavior::RunWhenVirtualPaused,
        )
    }

    pub const fn fixed() -> Self {
        Self::new(
            SceneSystemClockDomain::Fixed,
            SceneSystemPauseBehavior::SkipWhenVirtualPaused,
        )
    }

    pub const fn new(
        clock_domain: SceneSystemClockDomain,
        pause_behavior: SceneSystemPauseBehavior,
    ) -> Self {
        Self {
            clock_domain,
            pause_behavior,
        }
    }

    pub const fn clock_domain(self) -> SceneSystemClockDomain {
        self.clock_domain
    }

    pub const fn pause_behavior(self) -> SceneSystemPauseBehavior {
        self.pause_behavior
    }

    pub const fn runs_when_virtual_paused(self) -> bool {
        matches!(
            self.pause_behavior,
            SceneSystemPauseBehavior::RunWhenVirtualPaused
        )
    }

    /// Fixed stages must consume the committed fixed clock. Other stages cannot consume it.
    pub const fn is_valid_for_stage(self, stage: SystemStage) -> bool {
        if stage.is_fixed_loop() {
            matches!(self.clock_domain, SceneSystemClockDomain::Fixed)
                && matches!(
                    self.pause_behavior,
                    SceneSystemPauseBehavior::SkipWhenVirtualPaused
                )
        } else {
            !matches!(self.clock_domain, SceneSystemClockDomain::Fixed)
        }
    }
}

#[cfg(test)]
#[path = "tests/tick_policy.rs"]
mod tests;

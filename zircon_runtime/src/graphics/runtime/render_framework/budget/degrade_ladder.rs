//! 内存超预算时依既定顺序调整本帧规模和特性，避免子系统独立降级导致图配置不一致。
use crate::core::framework::render::RenderFrameProfile;

use super::{is_memory_over_budget, GpuMemoryBudget};

const DEFAULT_HYSTERESIS_FRAMES: u32 = 120;
const DEFAULT_RENDER_SCALE: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::runtime::render_framework) enum DegradeStep {
    RenderScale(f32),
    GlobalMipBias(i32),
    DisableFeature(&'static str),
}

const FIXED_DEGRADE_STEPS: [DegradeStep; 7] = [
    DegradeStep::RenderScale(0.85),
    DegradeStep::RenderScale(0.7),
    DegradeStep::GlobalMipBias(1),
    DegradeStep::DisableFeature("ssr"),
    DegradeStep::DisableFeature("ssao"),
    DegradeStep::DisableFeature("contact_shadow"),
    DegradeStep::DisableFeature("bloom_high"),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::runtime::render_framework) struct BudgetDegradeSettings {
    pub(in crate::graphics::runtime::render_framework) render_scale: f32,
    pub(in crate::graphics::runtime::render_framework) global_mip_bias: i32,
    pub(in crate::graphics::runtime::render_framework) disable_ssr: bool,
    pub(in crate::graphics::runtime::render_framework) disable_ssao: bool,
    pub(in crate::graphics::runtime::render_framework) disable_contact_shadow: bool,
    pub(in crate::graphics::runtime::render_framework) disable_bloom_high: bool,
}

impl Default for BudgetDegradeSettings {
    fn default() -> Self {
        Self {
            render_scale: DEFAULT_RENDER_SCALE,
            global_mip_bias: 0,
            disable_ssr: false,
            disable_ssao: false,
            disable_contact_shadow: false,
            disable_bloom_high: false,
        }
    }
}

pub(in crate::graphics::runtime::render_framework) struct BudgetDegradeLadder {
    active: usize,
    hysteresis_frames: u32,
    frames_under_budget: u32,
}

impl Default for BudgetDegradeLadder {
    fn default() -> Self {
        Self::with_hysteresis_frames(DEFAULT_HYSTERESIS_FRAMES)
    }
}

impl BudgetDegradeLadder {
    pub(in crate::graphics::runtime::render_framework) const fn with_hysteresis_frames(
        hysteresis_frames: u32,
    ) -> Self {
        Self {
            active: 0,
            hysteresis_frames: if hysteresis_frames == 0 {
                1
            } else {
                hysteresis_frames
            },
            frames_under_budget: 0,
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn evaluate(
        &mut self,
        profile: &RenderFrameProfile,
        budget: &GpuMemoryBudget,
    ) -> Option<&DegradeStep> {
        if is_memory_over_budget(profile, *budget) {
            self.frames_under_budget = 0;
            self.active = self.active.saturating_add(1).min(FIXED_DEGRADE_STEPS.len());
        } else if self.active > 0 {
            self.frames_under_budget = self.frames_under_budget.saturating_add(1);
            if self.frames_under_budget >= self.hysteresis_frames {
                self.active -= 1;
                self.frames_under_budget = 0;
            }
        } else {
            self.frames_under_budget = 0;
        }

        self.active_step()
    }

    pub(in crate::graphics::runtime::render_framework) const fn active_level(&self) -> usize {
        self.active
    }

    pub(in crate::graphics::runtime::render_framework) fn settings(&self) -> BudgetDegradeSettings {
        let mut settings = BudgetDegradeSettings::default();
        for step in FIXED_DEGRADE_STEPS.iter().take(self.active) {
            match *step {
                DegradeStep::RenderScale(scale) => settings.render_scale = scale,
                DegradeStep::GlobalMipBias(bias) => settings.global_mip_bias = bias,
                DegradeStep::DisableFeature("ssr") => settings.disable_ssr = true,
                DegradeStep::DisableFeature("ssao") => settings.disable_ssao = true,
                DegradeStep::DisableFeature("contact_shadow") => {
                    settings.disable_contact_shadow = true;
                }
                DegradeStep::DisableFeature("bloom_high") => {
                    settings.disable_bloom_high = true;
                }
                DegradeStep::DisableFeature(_) => {}
            }
        }
        settings
    }

    fn active_step(&self) -> Option<&DegradeStep> {
        self.active
            .checked_sub(1)
            .and_then(|index| FIXED_DEGRADE_STEPS.get(index))
    }
}

#[cfg(test)]
#[path = "tests/degrade_ladder.rs"]
mod tests;

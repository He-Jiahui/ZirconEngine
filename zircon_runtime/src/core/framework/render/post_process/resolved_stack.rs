use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings,
    RenderPostProcessEffectStackSettings,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderResolvedPostProcessSettings {
    pub bloom: RenderBloomSettings,
    pub exposure: RenderExposureSettings,
    pub color_grading: RenderColorGradingSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings,
}

impl RenderResolvedPostProcessSettings {
    pub const fn new(
        bloom: RenderBloomSettings,
        exposure: RenderExposureSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        Self {
            bloom,
            exposure,
            color_grading,
            effect_stack,
        }
    }
}

use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderPostProcessEffectStackSettings,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderPostProcessVolumeProfile {
    pub bloom: Option<RenderBloomSettings>,
    pub color_grading: Option<RenderColorGradingSettings>,
    pub effect_stack: Option<RenderPostProcessEffectStackSettings>,
}

impl RenderPostProcessVolumeProfile {
    pub const fn with_bloom(mut self, bloom: RenderBloomSettings) -> Self {
        self.bloom = Some(bloom);
        self
    }

    pub const fn with_color_grading(mut self, color_grading: RenderColorGradingSettings) -> Self {
        self.color_grading = Some(color_grading);
        self
    }

    pub const fn with_effect_stack(
        mut self,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        self.effect_stack = Some(effect_stack);
        self
    }
}

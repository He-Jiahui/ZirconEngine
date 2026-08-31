use crate::core::framework::render::{
    AoSourceSettings, RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings,
    RenderPostProcessEffectStackSettings, VolumetricFogSettings,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderResolvedPostProcessSettings {
    pub ambient_occlusion: AoSourceSettings,
    pub bloom: RenderBloomSettings,
    pub exposure: RenderExposureSettings,
    pub color_grading: RenderColorGradingSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings,
    pub volumetric_fog: VolumetricFogSettings,
}

impl RenderResolvedPostProcessSettings {
    pub const fn new(
        bloom: RenderBloomSettings,
        exposure: RenderExposureSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        Self {
            ambient_occlusion: AoSourceSettings::DEFAULT,
            bloom,
            exposure,
            color_grading,
            effect_stack,
            volumetric_fog: VolumetricFogSettings::DEFAULT,
        }
    }

    pub const fn with_ambient_occlusion(mut self, ambient_occlusion: AoSourceSettings) -> Self {
        self.ambient_occlusion = ambient_occlusion;
        self
    }
}

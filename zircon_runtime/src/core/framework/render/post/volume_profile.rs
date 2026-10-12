use crate::core::framework::render::{
    AoSourceSettings, RenderBloomSettings, RenderColorGradingSettings,
    RenderPostProcessEffectStackSettings, VolumetricFogSettings,
};

/// 场景序列化及创作端使用的稀疏体积配置；`None` 表示该效果族不覆盖相机基础值。
/// 提取时需转换为按组件 schema 排列的覆盖向量，再由评估器逐字段插值。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderPostProcessVolumeProfile {
    pub ambient_occlusion: Option<AoSourceSettings>,
    pub volumetric_fog: Option<VolumetricFogSettings>,
    pub bloom: Option<RenderBloomSettings>,
    pub color_grading: Option<RenderColorGradingSettings>,
    pub effect_stack: Option<RenderPostProcessEffectStackSettings>,
}

impl RenderPostProcessVolumeProfile {
    pub const fn with_ambient_occlusion(mut self, ambient_occlusion: AoSourceSettings) -> Self {
        self.ambient_occlusion = Some(ambient_occlusion);
        self
    }

    pub const fn with_volumetric_fog(mut self, volumetric_fog: VolumetricFogSettings) -> Self {
        self.volumetric_fog = Some(volumetric_fog);
        self
    }

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

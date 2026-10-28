use crate::core::framework::render::{
    AoSourceSettings, RenderBloomSettings, RenderColorGradingSettings,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile,
};
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::PostProcessSettingsComponent",
    field(
        name = "ao_intensity",
        value_type_path = "Scalar",
        read = "super::reflection::post_process::read_ao_intensity",
        readonly
    ),
    field(
        name = "bloom_threshold",
        value_type_path = "Scalar",
        read = "super::reflection::post_process::read_bloom_threshold",
        readonly
    ),
    field(
        name = "bloom_intensity",
        value_type_path = "Scalar",
        read = "super::reflection::post_process::read_bloom_intensity",
        readonly
    ),
    field(
        name = "bloom_radius",
        value_type_path = "Scalar",
        read = "super::reflection::post_process::read_bloom_radius",
        readonly
    ),
    field(
        name = "color_grading_exposure",
        value_type_path = "Scalar",
        read = "super::reflection::post_process::read_color_grading_exposure",
        readonly
    )
)]
pub struct PostProcessSettingsComponent {
    #[zr_reflect(skip)]
    pub ambient_occlusion: AoSourceSettings,
    #[zr_reflect(skip)]
    pub bloom: RenderBloomSettings,
    #[zr_reflect(skip)]
    pub color_grading: RenderColorGradingSettings,
    #[zr_reflect(skip)]
    pub effect_stack: RenderPostProcessEffectStackSettings,
}

impl Default for PostProcessSettingsComponent {
    fn default() -> Self {
        Self {
            ambient_occlusion: AoSourceSettings::default(),
            bloom: RenderBloomSettings::default(),
            color_grading: RenderColorGradingSettings::default(),
            effect_stack: RenderPostProcessEffectStackSettings::default(),
        }
    }
}

impl PostProcessSettingsComponent {
    pub fn from_parts(
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        Self {
            ambient_occlusion: AoSourceSettings::default(),
            bloom,
            color_grading,
            effect_stack,
        }
    }

    pub const fn with_ambient_occlusion(mut self, ambient_occlusion: AoSourceSettings) -> Self {
        self.ambient_occlusion = ambient_occlusion;
        self
    }
}

#[derive(Clone, Debug, PartialEq, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::PostProcessVolumeComponent"
)]
/// 摄像机周围可混合的后处理体积配置；提取阶段按空间关系与优先级组合，而非直接修改全局渲染设置。
pub struct PostProcessVolumeComponent {
    #[zr_reflect(readonly)]
    pub active: bool,
    #[zr_reflect(readonly)]
    pub is_global: bool,
    #[zr_reflect(readonly)]
    pub priority: Real,
    #[zr_reflect(readonly)]
    pub weight: Real,
    /// Scene-space fade distance outside a local volume collider.
    /// Global volumes ignore this field.
    #[zr_reflect(readonly)]
    pub blend_distance: Real,
    #[zr_reflect(skip)]
    pub profile: RenderPostProcessVolumeProfile,
}

impl Default for PostProcessVolumeComponent {
    fn default() -> Self {
        Self {
            active: true,
            is_global: true,
            priority: 0.0,
            weight: 1.0,
            blend_distance: 0.0,
            profile: RenderPostProcessVolumeProfile::default(),
        }
    }
}

impl PostProcessVolumeComponent {
    pub fn global(priority: Real, profile: RenderPostProcessVolumeProfile) -> Self {
        Self {
            priority,
            profile,
            ..Self::default()
        }
    }

    /// 创建局部体积配置；实体还须挂载盒形或球形碰撞体，帧提取才会形成对应体积。
    pub fn local(
        priority: Real,
        weight: Real,
        blend_distance: Real,
        profile: RenderPostProcessVolumeProfile,
    ) -> Self {
        Self {
            is_global: false,
            priority,
            weight,
            blend_distance,
            profile,
            ..Self::default()
        }
    }

    pub const fn with_weight(mut self, weight: Real) -> Self {
        self.weight = weight;
        self
    }
}

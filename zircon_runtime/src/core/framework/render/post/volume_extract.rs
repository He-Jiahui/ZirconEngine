use crate::core::framework::render::{
    RenderColorLookupTextureLayout, RenderLayerSet, RenderPostProcessEffectStackSettings,
    RenderPostProcessVolumeProfile, RenderTonemapOperator, VOLUMETRIC_FOG_COMPONENT_ID,
};
use crate::core::math::{Quat, Real, Vec3};

use super::VolumeParamValue;

const EFFECT_STACK_PROFILE_OVERRIDE_COUNT: usize = 11;

/// 场景提取后的体积形状快照，供相机位置计算局部影响权重。
/// 局部体积的边界距离与 `blend_distance` 一起决定混合，Global 不依赖相机位置。
#[derive(Clone, Debug, PartialEq)]
pub enum VolumeShapeExtract {
    Global,
    Box {
        center: Vec3,
        half_extents: Vec3,
        rotation: Quat,
        blend_distance: Real,
    },
    Sphere {
        center: Vec3,
        radius: Real,
        blend_distance: Real,
    },
}

impl VolumeShapeExtract {
    pub const fn global() -> Self {
        Self::Global
    }

    pub fn box_shape(
        center: Vec3,
        half_extents: Vec3,
        rotation: Quat,
        blend_distance: Real,
    ) -> Self {
        Self::Box {
            center,
            half_extents: half_extents.abs(),
            rotation,
            blend_distance: sanitize_blend_distance(blend_distance),
        }
    }

    pub fn sphere(center: Vec3, radius: Real, blend_distance: Real) -> Self {
        Self::Sphere {
            center,
            radius: radius.max(0.0),
            blend_distance: sanitize_blend_distance(blend_distance),
        }
    }

    pub const fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VolumeComponentOverride {
    pub component_id: String,
    pub values: Vec<Option<VolumeParamValue>>,
}

impl VolumeComponentOverride {
    pub fn new(
        component_id: impl Into<String>,
        values: impl IntoIterator<Item = Option<VolumeParamValue>>,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            values: values.into_iter().collect(),
        }
    }

    pub fn from_values(
        component_id: impl Into<String>,
        values: impl IntoIterator<Item = VolumeParamValue>,
    ) -> Self {
        Self::new(component_id, values.into_iter().map(Some))
    }

    pub fn from_profile(profile: &RenderPostProcessVolumeProfile) -> Vec<Self> {
        let override_count = profile_override_count(profile);
        let mut overrides = Vec::with_capacity(override_count);
        if let Some(volumetric_fog) = profile.volumetric_fog {
            overrides.push(Self::from_values(
                VOLUMETRIC_FOG_COMPONENT_ID,
                [
                    VolumeParamValue::Float(volumetric_fog.density),
                    VolumeParamValue::Vec3(volumetric_fog.albedo),
                    VolumeParamValue::Float(volumetric_fog.phase_g),
                    VolumeParamValue::Float(volumetric_fog.height_falloff),
                    VolumeParamValue::Float(volumetric_fog.scattering_intensity),
                    VolumeParamValue::Float(volumetric_fog.depth_distribution_exp),
                    VolumeParamValue::Bool(volumetric_fog.temporal),
                ],
            ));
        }
        if let Some(ambient_occlusion) = profile.ambient_occlusion {
            overrides.push(Self::from_values(
                "post.ambient-occlusion",
                [
                    VolumeParamValue::Float(ambient_occlusion.intensity),
                    VolumeParamValue::Float(ambient_occlusion.radius_meters),
                    VolumeParamValue::Float(ambient_occlusion.thickness_meters),
                    VolumeParamValue::Float(ambient_occlusion.depth_bias_meters),
                    VolumeParamValue::Float(ambient_occlusion.falloff_start_meters),
                    VolumeParamValue::Enum(ambient_occlusion.quality.stable_id()),
                    VolumeParamValue::Bool(ambient_occlusion.half_resolution),
                    VolumeParamValue::Bool(ambient_occlusion.temporal),
                ],
            ));
        }
        if let Some(bloom) = profile.bloom {
            overrides.push(Self::from_values(
                "post.bloom",
                [
                    VolumeParamValue::Float(bloom.threshold),
                    VolumeParamValue::Float(bloom.intensity),
                    VolumeParamValue::Float(bloom.radius),
                ],
            ));
        }
        if let Some(color_grading) = profile.color_grading {
            overrides.push(Self::from_values(
                "post.color-grading",
                [
                    VolumeParamValue::Float(color_grading.exposure),
                    VolumeParamValue::Float(color_grading.contrast),
                    VolumeParamValue::Float(color_grading.saturation),
                    VolumeParamValue::Float(color_grading.gamma),
                    VolumeParamValue::Vec3(color_grading.tint),
                ],
            ));
        }
        if let Some(effect_stack) = profile.effect_stack {
            push_effect_stack_overrides(&mut overrides, effect_stack);
        }
        overrides
    }
}

/// 场景层交给每相机求值器的体积快照；层掩码、优先级和权重先于参数插值生效。
/// 同优先级体积保持场景提取顺序，以便连续帧得到稳定的覆盖结果。
#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessVolumeExtract {
    pub active: bool,
    pub shape: VolumeShapeExtract,
    // TODO: [CR-RENDER-POST-0001] 明确非有限优先级的排序规则；当前场景提取与求值均把 NaN 比较当作相等，缺少输入约束和回归测试。
    pub priority: Real,
    pub weight: Real,
    pub volume_mask: RenderLayerSet,
    pub overrides: Vec<VolumeComponentOverride>,
}

impl PostProcessVolumeExtract {
    pub fn new(
        active: bool,
        shape: VolumeShapeExtract,
        priority: Real,
        weight: Real,
        volume_mask: RenderLayerSet,
        overrides: Vec<VolumeComponentOverride>,
    ) -> Self {
        Self {
            active,
            shape,
            priority,
            weight,
            volume_mask,
            overrides,
        }
    }

    pub fn global(
        priority: Real,
        weight: Real,
        volume_mask: RenderLayerSet,
        overrides: Vec<VolumeComponentOverride>,
    ) -> Self {
        Self::new(
            true,
            VolumeShapeExtract::Global,
            priority,
            weight,
            volume_mask,
            overrides,
        )
    }

    pub fn clamped_weight(&self) -> Real {
        saturate(self.weight)
    }
}

fn profile_override_count(profile: &RenderPostProcessVolumeProfile) -> usize {
    profile.volumetric_fog.is_some() as usize
        + profile.ambient_occlusion.is_some() as usize
        + profile.bloom.is_some() as usize
        + profile.color_grading.is_some() as usize
        + profile.effect_stack.is_some() as usize * EFFECT_STACK_PROFILE_OVERRIDE_COUNT
}

fn push_effect_stack_overrides(
    overrides: &mut Vec<VolumeComponentOverride>,
    effect_stack: RenderPostProcessEffectStackSettings,
) {
    overrides.push(EffectStackOverride::depth_of_field(effect_stack));
    overrides.push(EffectStackOverride::motion_blur(effect_stack));
    overrides.push(EffectStackOverride::screen_space_reflection(effect_stack));
    overrides.push(EffectStackOverride::screen_space_fog(effect_stack));
    overrides.push(EffectStackOverride::tonemap(effect_stack));
    overrides.push(EffectStackOverride::vignette(effect_stack));
    overrides.push(EffectStackOverride::grain(effect_stack));
    overrides.push(EffectStackOverride::dither(effect_stack));
    overrides.push(EffectStackOverride::chromatic_aberration(effect_stack));
    overrides.push(EffectStackOverride::color_lookup(effect_stack));
    overrides.push(EffectStackOverride::blur(effect_stack));
}

struct EffectStackOverride;

impl EffectStackOverride {
    fn depth_of_field(
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> VolumeComponentOverride {
        let settings = effect_stack.depth_of_field;
        VolumeComponentOverride::from_values(
            "post.depth-of-field",
            [
                VolumeParamValue::Float(settings.focus_distance),
                VolumeParamValue::Float(settings.focus_range),
                VolumeParamValue::Float(settings.aperture),
                VolumeParamValue::Float(settings.focal_length_mm),
                VolumeParamValue::Float(settings.max_blur_radius),
                VolumeParamValue::Uint(settings.bokeh_blade_count),
                VolumeParamValue::Float(settings.bokeh_rotation_radians),
            ],
        )
    }

    fn motion_blur(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.motion_blur;
        VolumeComponentOverride::from_values(
            "post.motion-blur",
            [
                VolumeParamValue::Float(settings.shutter_angle),
                VolumeParamValue::Uint(settings.samples),
            ],
        )
    }

    fn screen_space_reflection(
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> VolumeComponentOverride {
        let settings = effect_stack.screen_space_reflection;
        VolumeComponentOverride::from_values(
            "post.screen-space-reflection",
            [
                VolumeParamValue::Float(settings.intensity),
                VolumeParamValue::Float(settings.thickness),
                VolumeParamValue::Float(settings.max_ray_distance),
                VolumeParamValue::Uint(settings.max_steps),
                VolumeParamValue::Float(settings.temporal_blend_factor),
                VolumeParamValue::Float(settings.roughness_mip_bias),
            ],
        )
    }

    fn screen_space_fog(
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> VolumeComponentOverride {
        let settings = effect_stack.fog;
        VolumeComponentOverride::from_values(
            "post.screen-space-fog",
            [
                VolumeParamValue::Float(settings.density),
                VolumeParamValue::Float(settings.height_falloff),
                VolumeParamValue::Vec3(settings.color),
            ],
        )
    }

    fn tonemap(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.tonemap;
        VolumeComponentOverride::from_values(
            "post.tonemap",
            [
                VolumeParamValue::Enum(tonemap_operator_id(settings.operator)),
                VolumeParamValue::Float(settings.exposure_bias),
                VolumeParamValue::Float(settings.white_point),
            ],
        )
    }

    fn vignette(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.vignette;
        VolumeComponentOverride::from_values(
            "post.vignette",
            [
                VolumeParamValue::Float(settings.intensity),
                VolumeParamValue::Float(settings.smoothness),
                VolumeParamValue::Float(settings.roundness),
            ],
        )
    }

    fn grain(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.grain;
        VolumeComponentOverride::from_values(
            "post.grain",
            [
                VolumeParamValue::Float(settings.intensity),
                VolumeParamValue::Float(settings.response),
            ],
        )
    }

    fn dither(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.dither;
        VolumeComponentOverride::from_values(
            "post.dither",
            [
                VolumeParamValue::Float(settings.intensity),
                VolumeParamValue::Float(settings.scale),
            ],
        )
    }

    fn chromatic_aberration(
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> VolumeComponentOverride {
        let settings = effect_stack.chromatic_aberration;
        VolumeComponentOverride::from_values(
            "post.chromatic-aberration",
            [
                VolumeParamValue::Float(settings.intensity),
                VolumeParamValue::Float(settings.sample_spread),
            ],
        )
    }

    fn color_lookup(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.color_lookup;
        let (layout_id, size) = color_lookup_layout_ids(settings.texture_layout);
        VolumeComponentOverride::from_values(
            "post.color-lookup",
            [
                VolumeParamValue::Enum(layout_id),
                VolumeParamValue::Uint(size),
                VolumeParamValue::Float(settings.intensity),
            ],
        )
    }

    fn blur(effect_stack: RenderPostProcessEffectStackSettings) -> VolumeComponentOverride {
        let settings = effect_stack.blur;
        VolumeComponentOverride::from_values(
            "post.blur",
            [VolumeParamValue::Float(settings.radius)],
        )
    }
}

fn tonemap_operator_id(operator: RenderTonemapOperator) -> u32 {
    match operator {
        RenderTonemapOperator::None => 0,
        RenderTonemapOperator::Reinhard => 1,
        RenderTonemapOperator::Aces => 2,
        RenderTonemapOperator::Filmic => 3,
    }
}

fn color_lookup_layout_ids(layout: RenderColorLookupTextureLayout) -> (u32, u32) {
    match layout {
        RenderColorLookupTextureLayout::Auto => (0, 0),
        RenderColorLookupTextureLayout::Texture2dStrip { size } => (1, size),
        RenderColorLookupTextureLayout::Texture3d { size } => (2, size),
    }
}

fn sanitize_blend_distance(blend_distance: Real) -> Real {
    if blend_distance.is_finite() {
        blend_distance.max(0.0)
    } else {
        0.0
    }
}

fn saturate(value: Real) -> Real {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/volume_extract.rs"]
mod tests;

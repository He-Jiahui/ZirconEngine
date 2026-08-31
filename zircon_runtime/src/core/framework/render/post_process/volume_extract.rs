use crate::core::framework::render::{
    RenderColorLookupTextureLayout, RenderLayerSet, RenderPostProcessEffectStackSettings,
    RenderPostProcessVolumeProfile, RenderTonemapOperator, VOLUMETRIC_FOG_COMPONENT_ID,
};
use crate::core::math::{Quat, Real, Vec3};

use super::VolumeParamValue;

const EFFECT_STACK_PROFILE_OVERRIDE_COUNT: usize = 11;

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

#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessVolumeExtract {
    pub active: bool,
    pub shape: VolumeShapeExtract,
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
mod tests {
    use crate::core::framework::render::{
        AoQualityTier, AoSourceSettings, RenderBloomSettings, RenderColorGradingSettings,
        RenderLayerSet, RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile,
        RenderTonemapOperator, RenderTonemapSettings,
    };
    use crate::core::math::{Quat, Vec3};

    use super::{
        PostProcessVolumeExtract, VolumeComponentOverride, VolumeParamValue, VolumeShapeExtract,
    };

    #[test]
    fn render_volume_extract_maps_profile_to_component_overrides() {
        let overrides = VolumeComponentOverride::from_profile(
            &RenderPostProcessVolumeProfile::default()
                .with_ambient_occlusion(AoSourceSettings {
                    intensity: 0.6,
                    quality: AoQualityTier::Ultra,
                    ..AoSourceSettings::default()
                })
                .with_bloom(RenderBloomSettings {
                    intensity: 0.75,
                    ..RenderBloomSettings::default()
                })
                .with_color_grading(RenderColorGradingSettings {
                    saturation: 0.5,
                    ..RenderColorGradingSettings::default()
                })
                .with_effect_stack(RenderPostProcessEffectStackSettings {
                    tonemap: RenderTonemapSettings {
                        operator: RenderTonemapOperator::Aces,
                        exposure_bias: 0.25,
                        ..RenderTonemapSettings::default()
                    },
                    ..RenderPostProcessEffectStackSettings::default()
                }),
        );

        assert!(overrides
            .iter()
            .any(|override_entry| override_entry.component_id == "post.ambient-occlusion"));
        assert!(overrides
            .iter()
            .any(|override_entry| override_entry.component_id == "post.bloom"));
        assert!(overrides
            .iter()
            .any(|override_entry| override_entry.component_id == "post.color-grading"));
        let tonemap = overrides
            .iter()
            .find(|override_entry| override_entry.component_id == "post.tonemap")
            .expect("effect stack profile should emit tonemap override");
        assert_eq!(tonemap.values[0], Some(VolumeParamValue::Enum(2)));
        assert_eq!(tonemap.values[1], Some(VolumeParamValue::Float(0.25)));
    }

    #[test]
    fn render_volume_extract_profile_override_order_is_stable() {
        let overrides = VolumeComponentOverride::from_profile(
            &RenderPostProcessVolumeProfile::default()
                .with_ambient_occlusion(AoSourceSettings::default())
                .with_bloom(RenderBloomSettings::default())
                .with_color_grading(RenderColorGradingSettings::default())
                .with_effect_stack(RenderPostProcessEffectStackSettings::default()),
        );

        let component_ids = overrides
            .iter()
            .map(|override_entry| override_entry.component_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            component_ids,
            [
                "post.ambient-occlusion",
                "post.bloom",
                "post.color-grading",
                "post.depth-of-field",
                "post.motion-blur",
                "post.screen-space-reflection",
                "post.screen-space-fog",
                "post.tonemap",
                "post.vignette",
                "post.grain",
                "post.dither",
                "post.chromatic-aberration",
                "post.color-lookup",
                "post.blur",
            ]
        );
    }

    #[test]
    fn render_volume_extract_preserves_unset_component_params() {
        let override_entry = VolumeComponentOverride::new(
            "post.bloom",
            [
                Some(VolumeParamValue::Float(1.0)),
                None,
                Some(VolumeParamValue::Float(0.25)),
            ],
        );

        assert_eq!(override_entry.values[1], None);
    }

    #[test]
    fn render_volume_extract_stores_global_box_and_sphere_shapes() {
        let global =
            PostProcessVolumeExtract::global(2.0, 1.25, RenderLayerSet::layer(3), Vec::new());
        assert!(global.shape.is_global());
        assert_eq!(global.clamped_weight(), 1.0);

        let box_shape = VolumeShapeExtract::box_shape(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(-2.0, 3.0, 4.0),
            Quat::IDENTITY,
            -5.0,
        );
        assert_eq!(
            box_shape,
            VolumeShapeExtract::Box {
                center: Vec3::new(1.0, 2.0, 3.0),
                half_extents: Vec3::new(2.0, 3.0, 4.0),
                rotation: Quat::IDENTITY,
                blend_distance: 0.0,
            }
        );

        assert_eq!(
            VolumeShapeExtract::sphere(Vec3::ONE, -4.0, f32::NAN),
            VolumeShapeExtract::Sphere {
                center: Vec3::ONE,
                radius: 0.0,
                blend_distance: 0.0,
            }
        );
    }
}

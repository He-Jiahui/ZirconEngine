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
    let global = PostProcessVolumeExtract::global(2.0, 1.25, RenderLayerSet::layer(3), Vec::new());
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

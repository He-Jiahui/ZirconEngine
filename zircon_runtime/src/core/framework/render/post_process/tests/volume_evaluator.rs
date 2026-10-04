use crate::core::framework::render::{
    AoSourceSettings, RenderBloomSettings, RenderColorGradingSettings, RenderExposureMode,
    RenderExposureSettings, RenderLayerSet, RenderPostProcessEffectStackSettings,
    RenderPostProcessVolumeProfile, RenderTonemapOperator, RenderTonemapSettings,
};
use crate::core::math::{Quat, Vec3};

use super::{
    PostProcessVolumeExtract, VolumeComponentOverride, VolumeEvaluationError,
    VolumeEvaluationRequest, VolumeEvaluator, VolumeParamValue, VolumeShapeExtract,
};

#[test]
fn render_volume_evaluator_blends_global_volumes_by_priority_order() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [
        PostProcessVolumeExtract::global(
            10.0,
            0.5,
            RenderLayerSet::default(),
            VolumeComponentOverride::from_profile(
                &RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                    threshold: 1.0,
                    intensity: 1.0,
                    radius: 0.4,
                }),
            ),
        ),
        PostProcessVolumeExtract::global(
            0.0,
            1.0,
            RenderLayerSet::default(),
            VolumeComponentOverride::from_profile(
                &RenderPostProcessVolumeProfile::default()
                    .with_bloom(RenderBloomSettings {
                        threshold: 1.0,
                        intensity: 0.5,
                        radius: 0.2,
                    })
                    .with_color_grading(RenderColorGradingSettings {
                        exposure: 1.2,
                        tint: Vec3::new(0.8, 0.9, 1.0),
                        ..Default::default()
                    }),
            ),
        ),
    ];
    let sorted_volumes = [volumes[1].clone(), volumes[0].clone()];

    let camera_mask = RenderLayerSet::default();
    let resolved = evaluator
        .evaluate(request(Vec3::ZERO, &camera_mask, &volumes))
        .unwrap();
    let sorted_resolved = evaluator
        .evaluate(request(Vec3::ZERO, &camera_mask, &sorted_volumes))
        .unwrap();

    assert_eq!(resolved, sorted_resolved);
    assert_near(resolved.bloom.intensity, 0.75);
    assert_near(resolved.bloom.radius, 0.3);
    assert_near(resolved.color_grading.exposure, 1.2);
    assert_eq!(resolved.color_grading.tint, Vec3::new(0.8, 0.9, 1.0));
}

#[test]
fn render_volume_evaluator_box_blend_distance_weight() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [PostProcessVolumeExtract::new(
        true,
        VolumeShapeExtract::box_shape(Vec3::ZERO, Vec3::splat(1.0), Quat::IDENTITY, 2.0),
        0.0,
        1.0,
        RenderLayerSet::default(),
        VolumeComponentOverride::from_profile(
            &RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                intensity: 1.0,
                ..RenderBloomSettings::default()
            }),
        ),
    )];

    let camera_mask = RenderLayerSet::default();
    let resolved = evaluator
        .evaluate(request(Vec3::new(2.0, 0.5, 0.25), &camera_mask, &volumes))
        .unwrap();

    assert_near(resolved.bloom.intensity, 0.75);
}

#[test]
fn render_volume_evaluator_sphere_boundary_zero_influence() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [PostProcessVolumeExtract::new(
        true,
        VolumeShapeExtract::sphere(Vec3::ZERO, 1.0, 1.0),
        0.0,
        1.0,
        RenderLayerSet::default(),
        VolumeComponentOverride::from_profile(
            &RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                intensity: 1.0,
                ..RenderBloomSettings::default()
            }),
        ),
    )];

    let camera_mask = RenderLayerSet::default();
    let resolved = evaluator
        .evaluate(request(Vec3::new(3.5, 0.0, 0.0), &camera_mask, &volumes))
        .unwrap();

    assert_eq!(resolved.bloom, RenderBloomSettings::default());
}

#[test]
fn render_volume_evaluator_respects_camera_volume_mask() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [PostProcessVolumeExtract::global(
        0.0,
        1.0,
        RenderLayerSet::layer(9),
        VolumeComponentOverride::from_profile(
            &RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                intensity: 1.0,
                ..RenderBloomSettings::default()
            }),
        ),
    )];

    let camera_mask = RenderLayerSet::default();
    let resolved = evaluator
        .evaluate(request(Vec3::ZERO, &camera_mask, &volumes))
        .unwrap();

    assert_eq!(resolved.bloom, RenderBloomSettings::default());
}

#[test]
fn render_volume_evaluator_blends_exposure_component() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [PostProcessVolumeExtract::global(
        0.0,
        0.5,
        RenderLayerSet::default(),
        vec![VolumeComponentOverride::from_values(
            "post.exposure",
            [
                VolumeParamValue::Enum(1),
                VolumeParamValue::Float(7.7),
                VolumeParamValue::Float(2.0),
                VolumeParamValue::Float(-6.0),
                VolumeParamValue::Float(10.0),
                VolumeParamValue::Float(0.2),
                VolumeParamValue::Float(0.8),
                VolumeParamValue::Float(2.0),
                VolumeParamValue::Float(0.5),
            ],
        )],
    )];
    let camera_mask = RenderLayerSet::default();
    let mut request = request(Vec3::ZERO, &camera_mask, &volumes);
    request.base_exposure = RenderExposureSettings::manual_ev100(9.7);

    let resolved = evaluator.evaluate(request).unwrap();

    assert_eq!(resolved.exposure.mode, RenderExposureMode::Histogram);
    assert_near(resolved.exposure.manual_ev100, 8.7);
    assert_near(resolved.exposure.compensation_ev, 1.0);
    assert_near(resolved.exposure.min_ev100, -7.0);
    assert_near(resolved.exposure.max_ev100, 9.0);
    assert_near(resolved.exposure.low_percent, 0.15);
    assert_near(resolved.exposure.high_percent, 0.85);
    assert_near(resolved.exposure.speed_brighten, 2.5);
    assert_near(resolved.exposure.speed_darken, 0.75);
}

#[test]
fn render_volume_evaluator_keeps_unset_component_params_from_current_stack() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [PostProcessVolumeExtract::global(
        0.0,
        0.5,
        RenderLayerSet::default(),
        vec![VolumeComponentOverride::new(
            "post.tonemap",
            [
                None,
                Some(VolumeParamValue::Float(2.0)),
                Some(VolumeParamValue::Float(3.0)),
            ],
        )],
    )];
    let camera_mask = RenderLayerSet::default();
    let mut request = request(Vec3::ZERO, &camera_mask, &volumes);
    request.base_effect_stack = RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: RenderTonemapOperator::Aces,
            exposure_bias: 0.0,
            white_point: 1.0,
        },
        ..RenderPostProcessEffectStackSettings::default()
    };

    let resolved = evaluator.evaluate(request).unwrap();

    assert_eq!(
        resolved.effect_stack.tonemap.operator,
        RenderTonemapOperator::Aces
    );
    assert_near(resolved.effect_stack.tonemap.exposure_bias, 1.0);
    assert_near(resolved.effect_stack.tonemap.white_point, 2.0);
}

#[test]
fn render_volume_evaluator_reports_unknown_component() {
    let evaluator = VolumeEvaluator::default();
    let volumes = [PostProcessVolumeExtract::global(
        0.0,
        1.0,
        RenderLayerSet::default(),
        vec![VolumeComponentOverride::from_values(
            "post.unknown",
            [VolumeParamValue::Float(1.0)],
        )],
    )];

    assert_eq!(
        {
            let camera_mask = RenderLayerSet::default();
            evaluator.evaluate(request(Vec3::ZERO, &camera_mask, &volumes))
        },
        Err(VolumeEvaluationError::UnknownComponentId {
            component_id: "post.unknown".to_string(),
        })
    );
}

fn request<'a>(
    camera_position: Vec3,
    camera_volume_mask: &'a RenderLayerSet,
    volumes: &'a [PostProcessVolumeExtract],
) -> VolumeEvaluationRequest<'a> {
    VolumeEvaluationRequest {
        camera_position,
        camera_volume_mask,
        base_ambient_occlusion: AoSourceSettings::default(),
        base_bloom: RenderBloomSettings::default(),
        base_exposure: RenderExposureSettings::default(),
        base_color_grading: RenderColorGradingSettings::default(),
        base_effect_stack: RenderPostProcessEffectStackSettings::default(),
        volumes,
    }
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "expected {actual} to be near {expected}"
    );
}

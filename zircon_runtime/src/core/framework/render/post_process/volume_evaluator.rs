use thiserror::Error;

use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings, RenderLayerSet,
    RenderPostProcessEffectStackSettings,
};
use crate::core::math::{Real, Vec3};

use super::resolved_stack::RenderResolvedPostProcessSettings;
use super::{
    PostProcessVolumeExtract, VolumeComponentApplyError, VolumeComponentDescriptor,
    VolumeComponentOverride, VolumeComponentRegistry, VolumeParamValue, VolumeShapeExtract,
};

pub type ResolvedPostProcessStack = RenderResolvedPostProcessSettings;

#[derive(Clone, Debug)]
pub struct VolumeEvaluationRequest<'a> {
    pub camera_position: Vec3,
    pub camera_volume_mask: &'a RenderLayerSet,
    pub base_bloom: RenderBloomSettings,
    pub base_exposure: RenderExposureSettings,
    pub base_color_grading: RenderColorGradingSettings,
    pub base_effect_stack: RenderPostProcessEffectStackSettings,
    pub volumes: &'a [PostProcessVolumeExtract],
}

impl<'a> VolumeEvaluationRequest<'a> {
    pub fn base_settings(&self) -> RenderResolvedPostProcessSettings {
        RenderResolvedPostProcessSettings::new(
            self.base_bloom,
            self.base_exposure,
            self.base_color_grading,
            self.base_effect_stack,
        )
    }
}

#[derive(Clone, Debug)]
pub struct VolumeEvaluator {
    registry: VolumeComponentRegistry,
}

impl Default for VolumeEvaluator {
    fn default() -> Self {
        Self::with_builtin_post_process_components()
    }
}

impl VolumeEvaluator {
    pub fn new(registry: VolumeComponentRegistry) -> Self {
        Self { registry }
    }

    pub fn with_builtin_post_process_components() -> Self {
        Self::new(VolumeComponentRegistry::with_builtin_post_process_components())
    }

    pub fn registry(&self) -> &VolumeComponentRegistry {
        &self.registry
    }

    pub fn evaluate(
        &self,
        request: VolumeEvaluationRequest<'_>,
    ) -> Result<ResolvedPostProcessStack, VolumeEvaluationError> {
        let mut settings = request.base_settings();
        let mut applicable = request
            .volumes
            .iter()
            .enumerate()
            .filter(|(_, volume)| volume.active)
            .filter(|(_, volume)| volume.volume_mask.intersects(request.camera_volume_mask))
            .filter_map(|(index, volume)| {
                let influence = volume_influence(volume, request.camera_position);
                (influence > 0.0).then_some((index, volume, influence))
            })
            .collect::<Vec<_>>();
        applicable.sort_by(|left, right| compare_volume_priority(left, right));

        for (_, volume, influence) in applicable {
            self.apply_volume(&mut settings, volume, influence)?;
        }

        Ok(settings)
    }

    fn apply_volume(
        &self,
        settings: &mut RenderResolvedPostProcessSettings,
        volume: &PostProcessVolumeExtract,
        influence: Real,
    ) -> Result<(), VolumeEvaluationError> {
        for override_entry in &volume.overrides {
            let descriptor = self.descriptor_for(override_entry)?;
            let blended_values =
                blended_override_values(descriptor, settings, override_entry, influence)?;
            descriptor
                .apply_values(settings, &blended_values)
                .map_err(VolumeEvaluationError::Apply)?;
        }
        Ok(())
    }

    fn descriptor_for(
        &self,
        override_entry: &VolumeComponentOverride,
    ) -> Result<VolumeComponentDescriptor, VolumeEvaluationError> {
        self.registry
            .get(&override_entry.component_id)
            .copied()
            .ok_or_else(|| VolumeEvaluationError::UnknownComponentId {
                component_id: override_entry.component_id.clone(),
            })
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum VolumeEvaluationError {
    #[error("post-process volume component `{component_id}` is not registered")]
    UnknownComponentId { component_id: String },
    #[error("post-process volume component application failed: {0:?}")]
    Apply(VolumeComponentApplyError),
}

fn blended_override_values(
    descriptor: VolumeComponentDescriptor,
    settings: &RenderResolvedPostProcessSettings,
    override_entry: &VolumeComponentOverride,
    influence: Real,
) -> Result<Vec<VolumeParamValue>, VolumeEvaluationError> {
    if override_entry.values.len() != descriptor.params.len() {
        return Err(VolumeEvaluationError::Apply(
            VolumeComponentApplyError::ParamCountMismatch {
                component_id: descriptor.component_id,
                expected: descriptor.params.len(),
                actual: override_entry.values.len(),
            },
        ));
    }

    let mut values = descriptor.read_values(settings);
    if values.len() != descriptor.params.len() {
        return Err(VolumeEvaluationError::Apply(
            VolumeComponentApplyError::ParamCountMismatch {
                component_id: descriptor.component_id,
                expected: descriptor.params.len(),
                actual: values.len(),
            },
        ));
    }

    let influence = saturate(influence);
    for ((value, override_value), param) in values
        .iter_mut()
        .zip(override_entry.values.iter())
        .zip(descriptor.params.iter())
    {
        if let Some(to) = override_value {
            *value = (param.interp)(*value, *to, influence);
        }
    }

    Ok(values)
}

fn compare_volume_priority(
    left: &(usize, &PostProcessVolumeExtract, Real),
    right: &(usize, &PostProcessVolumeExtract, Real),
) -> std::cmp::Ordering {
    left.1
        .priority
        .partial_cmp(&right.1.priority)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| left.0.cmp(&right.0))
}

fn volume_influence(volume: &PostProcessVolumeExtract, camera_position: Vec3) -> Real {
    volume.clamped_weight() * shape_influence(&volume.shape, camera_position)
}

fn shape_influence(shape: &VolumeShapeExtract, camera_position: Vec3) -> Real {
    match shape {
        VolumeShapeExtract::Global => 1.0,
        VolumeShapeExtract::Box {
            center,
            half_extents,
            rotation,
            blend_distance,
        } => {
            let local_position = rotation.inverse() * (camera_position - *center);
            let distance_sq = (local_position.abs() - half_extents.abs())
                .max(Vec3::ZERO)
                .length_squared();
            influence_from_distance_squared(distance_sq, *blend_distance)
        }
        VolumeShapeExtract::Sphere {
            center,
            radius,
            blend_distance,
        } => {
            let distance_sq = (camera_position.distance(*center) - radius.max(0.0))
                .max(0.0)
                .powi(2);
            influence_from_distance_squared(distance_sq, *blend_distance)
        }
    }
}

fn influence_from_distance_squared(distance_sq: Real, blend_distance: Real) -> Real {
    if distance_sq <= 0.0 {
        return 1.0;
    }
    let blend_distance = if blend_distance.is_finite() {
        blend_distance.max(0.0)
    } else {
        0.0
    };
    if blend_distance <= 0.0 {
        return 0.0;
    }
    let blend_distance_sq = blend_distance * blend_distance;
    if distance_sq > blend_distance_sq {
        return 0.0;
    }
    1.0 - (distance_sq / blend_distance_sq)
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
        RenderBloomSettings, RenderColorGradingSettings, RenderExposureMode,
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

        let camera_mask = RenderLayerSet::default();
        let resolved = evaluator
            .evaluate(request(Vec3::ZERO, &camera_mask, &volumes))
            .unwrap();

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
}

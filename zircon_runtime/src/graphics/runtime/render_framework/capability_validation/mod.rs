use crate::core::framework::render::{
    RenderCapabilityMismatchDetail, RenderCapabilitySummary, RenderFrameworkError,
    RenderPipelineHandle, RenderQualityProfile,
};
use crate::{CompiledRenderPipeline, RenderFeatureCapabilityRequirement};

pub(in crate::graphics::runtime::render_framework) fn validate_quality_profile_capabilities(
    pipeline: Option<RenderPipelineHandle>,
    profile: &RenderQualityProfile,
    capabilities: &RenderCapabilitySummary,
) -> Result<(), RenderFrameworkError> {
    let missing = missing_capability_details(profile.capability_requirements(), capabilities);

    if missing.is_empty() {
        return Ok(());
    }

    let missing_labels = missing_labels(&missing);

    Err(RenderFrameworkError::CapabilityMismatch {
        pipeline: pipeline.map(RenderPipelineHandle::raw).unwrap_or(0),
        reason: format!(
            "quality profile `{}` requires {}",
            profile.name,
            missing_labels.join(", ")
        ),
        missing,
    })
}

pub(in crate::graphics::runtime::render_framework) fn validate_compiled_pipeline_capabilities(
    pipeline: &CompiledRenderPipeline,
    capabilities: &RenderCapabilitySummary,
) -> Result<(), RenderFrameworkError> {
    let missing = missing_capability_details(
        pipeline.capability_requirements.iter().copied(),
        capabilities,
    );

    if missing.is_empty() {
        return Ok(());
    }

    let missing_labels = missing_labels(&missing);

    Err(RenderFrameworkError::CapabilityMismatch {
        pipeline: pipeline.handle.raw(),
        reason: format!(
            "pipeline `{}` requires {}",
            pipeline.name,
            missing_labels.join(", ")
        ),
        missing,
    })
}

fn missing_capability_details(
    requirements: impl IntoIterator<Item = RenderFeatureCapabilityRequirement>,
    capabilities: &RenderCapabilitySummary,
) -> Vec<RenderCapabilityMismatchDetail> {
    requirements
        .into_iter()
        .filter(|requirement| !requirement.is_satisfied_by(capabilities))
        .map(|requirement| RenderCapabilityMismatchDetail::new(requirement.capability_kind()))
        .collect()
}

fn missing_labels(missing: &[RenderCapabilityMismatchDetail]) -> Vec<&'static str> {
    missing.iter().map(|detail| (*detail).label()).collect()
}

trait RenderQualityProfileCapabilityRequirements {
    fn capability_requirements(&self) -> Vec<RenderFeatureCapabilityRequirement>;
}

impl RenderQualityProfileCapabilityRequirements for RenderQualityProfile {
    fn capability_requirements(&self) -> Vec<RenderFeatureCapabilityRequirement> {
        let mut requirements = Vec::new();
        if self.features.virtual_geometry {
            requirements.push(RenderFeatureCapabilityRequirement::VirtualGeometry);
        }
        if self.features.hybrid_global_illumination {
            requirements.push(RenderFeatureCapabilityRequirement::HybridGlobalIllumination);
        }
        requirements
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderCapabilityKind, RenderCapabilityMismatchDetail, RenderCapabilitySummary,
        RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile,
        RenderWorldSnapshotHandle,
    };
    use crate::render_graph::QueueLane;
    use crate::scene::world::World;
    use crate::{
        RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
    };

    use super::{validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities};

    #[test]
    fn quality_profile_capability_validation_reports_all_missing_flagship_features() {
        let profile = RenderQualityProfile::new("flagship")
            .with_virtual_geometry(true)
            .with_hybrid_global_illumination(true);
        let capabilities = RenderCapabilitySummary {
            backend_name: "capability-test".to_string(),
            supports_offscreen: true,
            ..Default::default()
        };

        let error = validate_quality_profile_capabilities(
            Some(RenderPipelineHandle::new(7)),
            &profile,
            &capabilities,
        )
        .unwrap_err();

        assert_eq!(
            error,
            RenderFrameworkError::CapabilityMismatch {
                pipeline: 7,
                reason:
                    "quality profile `flagship` requires virtual_geometry, hybrid_global_illumination"
                        .to_string(),
                missing: vec![
                    RenderCapabilityMismatchDetail::new(RenderCapabilityKind::VirtualGeometry),
                    RenderCapabilityMismatchDetail::new(
                        RenderCapabilityKind::HybridGlobalIllumination,
                    ),
                ],
            }
        );
    }

    #[test]
    fn compiled_pipeline_capability_validation_reports_descriptor_requirements() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "plugin.virtual_geometry.capability_validation",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::DepthPrepass,
                        "plugin-virtual-geometry-capability-validation",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.virtual-geometry.capability-validation")
                    .with_side_effects()],
                )
                .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
            ));
        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry),
            )
            .unwrap();
        assert_eq!(
            compiled.capability_requirements,
            vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
        );
        let capabilities = RenderCapabilitySummary {
            backend_name: "capability-test".to_string(),
            supports_offscreen: true,
            ..Default::default()
        };

        let error = validate_compiled_pipeline_capabilities(&compiled, &capabilities).unwrap_err();

        assert_eq!(
            error,
            RenderFrameworkError::CapabilityMismatch {
                pipeline: compiled.handle.raw(),
                reason: format!("pipeline `{}` requires virtual_geometry", compiled.name),
                missing: vec![RenderCapabilityMismatchDetail::new(
                    RenderCapabilityKind::VirtualGeometry,
                )],
            }
        );
    }

    #[test]
    fn compiled_pipeline_capability_validation_splits_rt_backend_requirements() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "plugin.rt.capability_validation",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "plugin-rt-capability-validation",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.rt.capability-validation")
                    .with_side_effects()],
                )
                .with_capability_requirement(
                    RenderFeatureCapabilityRequirement::AccelerationStructures,
                )
                .with_capability_requirement(
                    RenderFeatureCapabilityRequirement::RayTracingPipeline,
                ),
            ));
        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_capability_enabled(
                        RenderFeatureCapabilityRequirement::AccelerationStructures,
                    )
                    .with_capability_enabled(
                        RenderFeatureCapabilityRequirement::RayTracingPipeline,
                    ),
            )
            .unwrap();
        let capabilities = RenderCapabilitySummary {
            backend_name: "capability-test".to_string(),
            supports_offscreen: true,
            ..Default::default()
        };

        let error = validate_compiled_pipeline_capabilities(&compiled, &capabilities).unwrap_err();

        assert_eq!(
            error,
            RenderFrameworkError::CapabilityMismatch {
                pipeline: compiled.handle.raw(),
                reason: format!(
                    "pipeline `{}` requires acceleration_structures, ray_tracing_pipeline",
                    compiled.name
                ),
                missing: vec![
                    RenderCapabilityMismatchDetail::new(
                        RenderCapabilityKind::AccelerationStructures,
                    ),
                    RenderCapabilityMismatchDetail::new(RenderCapabilityKind::RayTracingPipeline),
                ],
            }
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }
}

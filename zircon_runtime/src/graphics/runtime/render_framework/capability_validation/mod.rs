use crate::core::framework::render::{
    RenderCapabilitySummary, RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile,
};
use crate::{CompiledRenderPipeline, RenderFeatureCapabilityRequirement};

pub(in crate::graphics::runtime::render_framework) fn validate_quality_profile_capabilities(
    pipeline: Option<RenderPipelineHandle>,
    profile: &RenderQualityProfile,
    capabilities: &RenderCapabilitySummary,
) -> Result<(), RenderFrameworkError> {
    let missing = profile
        .capability_requirements()
        .into_iter()
        .filter(|requirement| !requirement.is_satisfied_by(capabilities))
        .map(RenderFeatureCapabilityRequirement::label)
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(());
    }

    Err(RenderFrameworkError::CapabilityMismatch {
        pipeline: pipeline.map(RenderPipelineHandle::raw).unwrap_or(0),
        reason: format!(
            "quality profile `{}` requires {}",
            profile.name,
            missing.join(", ")
        ),
    })
}

pub(in crate::graphics::runtime::render_framework) fn validate_compiled_pipeline_capabilities(
    pipeline: &CompiledRenderPipeline,
    capabilities: &RenderCapabilitySummary,
) -> Result<(), RenderFrameworkError> {
    let missing = pipeline
        .capability_requirements
        .iter()
        .copied()
        .filter(|requirement| !requirement.is_satisfied_by(capabilities))
        .map(RenderFeatureCapabilityRequirement::label)
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(());
    }

    Err(RenderFrameworkError::CapabilityMismatch {
        pipeline: pipeline.handle.raw(),
        reason: format!(
            "pipeline `{}` requires {}",
            pipeline.name,
            missing.join(", ")
        ),
    })
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
        RenderCapabilitySummary, RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle,
        RenderQualityProfile, RenderWorldSnapshotHandle,
    };
    use crate::scene::world::World;
    use crate::{
        BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderPipelineAsset,
        RenderPipelineCompileOptions,
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
            }
        );
    }

    #[test]
    fn compiled_pipeline_capability_validation_reports_descriptor_requirements() {
        let pipeline = RenderPipelineAsset::default_forward_plus();
        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry),
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

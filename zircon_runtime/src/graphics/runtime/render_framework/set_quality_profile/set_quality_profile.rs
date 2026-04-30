use crate::core::framework::render::{
    RenderFrameworkError, RenderQualityProfile, RenderViewportHandle,
};

use super::super::capability_validation::{
    validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities,
};
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_quality_profile(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    profile: RenderQualityProfile,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let capabilities = state.stats.capabilities.clone();
    let active_pipeline = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .pipeline();
    let effective_pipeline = active_pipeline.or(profile.pipeline_override);
    if let Some(pipeline) = profile.pipeline_override {
        if !state.pipelines.contains_key(&pipeline) {
            return Err(RenderFrameworkError::UnknownPipeline {
                pipeline: pipeline.raw(),
            });
        }
    }
    if let Some(pipeline) = effective_pipeline {
        let pipeline_asset = state.pipelines.get(&pipeline).cloned().ok_or(
            RenderFrameworkError::UnknownPipeline {
                pipeline: pipeline.raw(),
            },
        )?;
        let compiled = compile_pipeline_for_validation(&pipeline_asset)?;
        state
            .renderer
            .validate_compiled_pipeline_executors(&compiled)
            .map_err(|message| RenderFrameworkError::GraphCompileFailure {
                pipeline: pipeline.raw(),
                message,
            })?;
        validate_compiled_pipeline_capabilities(&compiled, &capabilities)?;
    }
    validate_quality_profile_capabilities(effective_pipeline, &profile, &capabilities)?;
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
    record.set_quality_profile(profile.clone());
    state.stats.last_quality_profile = Some(profile.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::{
        RenderFramework, RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile,
        RenderViewportDescriptor,
    };
    use crate::core::math::UVec2;
    use crate::render_graph::QueueLane;
    use crate::{
        BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
    };

    use super::set_quality_profile;

    #[test]
    fn set_quality_profile_revalidates_override_graph_executor_contract() {
        let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
            .unwrap();
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline.handle = RenderPipelineHandle::new(91);
        pipeline.name = "profile-override-invalid-executor-pipeline".to_string();
        let bloom = pipeline
            .renderer
            .features
            .iter_mut()
            .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
            .expect("default pipeline should include bloom");
        *bloom = bloom
            .clone()
            .with_descriptor_override(RenderFeatureDescriptor::new(
                "profile-override-invalid-executor-feature",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "profile-override-invalid-executor-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("custom.profile-override-missing-executor")
                .with_side_effects()],
            ));
        framework
            .state
            .lock()
            .unwrap()
            .pipelines
            .insert(pipeline.handle, pipeline);

        let error = set_quality_profile(
            &framework,
            viewport,
            RenderQualityProfile::new("profile-override-stale")
                .with_pipeline_asset(RenderPipelineHandle::new(91)),
        )
        .expect_err("quality profile override should re-run graph executor validation");

        assert_eq!(
            error,
            RenderFrameworkError::GraphCompileFailure {
                pipeline: 91,
                message:
                    "render pass `profile-override-invalid-executor-pass` references unregistered executor `custom.profile-override-missing-executor`"
                        .to_string(),
            }
        );
    }
}

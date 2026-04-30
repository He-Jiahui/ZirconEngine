use crate::core::framework::render::{
    RenderFrameworkError, RenderPipelineHandle, RenderViewportHandle,
};

use super::super::capability_validation::{
    validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities,
};
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_pipeline_asset(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let pipeline_asset =
        state
            .pipelines
            .get(&pipeline)
            .cloned()
            .ok_or(RenderFrameworkError::UnknownPipeline {
                pipeline: pipeline.raw(),
            })?;
    let capabilities = state.stats.capabilities.clone();
    let compiled = compile_pipeline_for_validation(&pipeline_asset)?;
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.raw(),
            message,
        })?;
    validate_compiled_pipeline_capabilities(&compiled, &capabilities)?;
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    if let Some(profile) = record.quality_profile() {
        validate_quality_profile_capabilities(Some(pipeline), profile, &capabilities)?;
    }
    record.set_pipeline(pipeline);
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::{
        RenderFramework, RenderFrameworkError, RenderPipelineHandle, RenderViewportDescriptor,
    };
    use crate::core::math::UVec2;
    use crate::render_graph::QueueLane;
    use crate::{
        BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
    };

    use super::set_pipeline_asset;

    #[test]
    fn set_pipeline_asset_revalidates_stale_graph_executor_contract() {
        let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
            .unwrap();
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline.handle = RenderPipelineHandle::new(90);
        pipeline.name = "stale-invalid-executor-pipeline".to_string();
        let bloom = pipeline
            .renderer
            .features
            .iter_mut()
            .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
            .expect("default pipeline should include bloom");
        *bloom = bloom
            .clone()
            .with_descriptor_override(RenderFeatureDescriptor::new(
                "stale-invalid-executor-feature",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "stale-invalid-executor-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("custom.stale-missing-executor")
                .with_side_effects()],
            ));
        framework
            .state
            .lock()
            .unwrap()
            .pipelines
            .insert(pipeline.handle, pipeline);

        let error = set_pipeline_asset(&framework, viewport, RenderPipelineHandle::new(90))
            .expect_err("setting a stale invalid pipeline should re-run graph executor validation");

        assert_eq!(
            error,
            RenderFrameworkError::GraphCompileFailure {
                pipeline: 90,
                message:
                    "render pass `stale-invalid-executor-pass` references unregistered executor `custom.stale-missing-executor`"
                        .to_string(),
            }
        );
    }
}

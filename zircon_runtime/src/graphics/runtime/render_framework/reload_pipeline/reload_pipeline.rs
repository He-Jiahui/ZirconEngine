use crate::core::framework::render::{RenderFrameworkError, RenderPipelineHandle};
use crate::graphics::RenderPipelineAsset;

use super::super::capability_validation::validate_compiled_pipeline_capabilities;
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn reload_pipeline(
    framework: &WgpuRenderFramework,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    let pipeline_asset =
        state
            .pipelines
            .get(&pipeline)
            .cloned()
            .ok_or(RenderFrameworkError::UnknownPipeline {
                pipeline: pipeline.raw(),
            })?;
    let compiled = compile_pipeline_for_validation(&pipeline_asset)?;
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.raw(),
            message,
        })?;
    let default_pipeline = RenderPipelineAsset::default_forward_plus().handle;
    let active_for_viewport = state
        .viewports
        .values()
        .any(|record| record.effective_pipeline(default_pipeline) == pipeline);
    if active_for_viewport {
        validate_compiled_pipeline_capabilities(&compiled, &state.stats.capabilities)?;
    }
    if let Some(pipeline_asset) = state.pipelines.get_mut(&pipeline) {
        pipeline_asset.bump_revision();
    }
    state.compiled_graph_cache.invalidate_pipeline(pipeline);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::RenderFrameworkError;
    use crate::graphics::{
        RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
    };
    use crate::graphics::{RenderPassExecutionContext, RenderPassExecutorRegistration};
    use crate::render_graph::QueueLane;

    use super::reload_pipeline;

    #[test]
    fn reload_pipeline_rejects_plugin_executor_without_linked_descriptor() {
        let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
        let pipeline = plugin_virtual_geometry_pipeline();
        let handle = pipeline.handle;
        framework
            .state
            .lock()
            .unwrap()
            .pipelines
            .insert(handle, pipeline);

        let error = reload_pipeline(&framework, handle)
            .expect_err("unlinked plugin executor ids should not reload");

        assert!(
            matches!(
                error,
                RenderFrameworkError::GraphCompileFailure { ref message, .. }
                    if message.contains("virtual-geometry.prepare")
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn reload_pipeline_rejects_plugin_executor_from_descriptor_only() {
        let descriptor = plugin_virtual_geometry_descriptor();
        let framework = WgpuRenderFramework::new_with_plugin_render_features(
            Arc::new(ProjectAssetManager::default()),
            [descriptor],
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let pipeline = plugin_virtual_geometry_pipeline();
        let handle = pipeline.handle;
        framework
            .state
            .lock()
            .unwrap()
            .pipelines
            .insert(handle, pipeline);

        let error = reload_pipeline(&framework, handle)
            .expect_err("plugin descriptors should not auto-register runtime no-op executors");

        assert!(
            matches!(
                error,
                RenderFrameworkError::GraphCompileFailure { ref message, .. }
                    if message.contains("virtual-geometry.prepare")
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn reload_pipeline_accepts_plugin_executor_from_explicit_registration() {
        let descriptor = plugin_virtual_geometry_descriptor();
        let framework = WgpuRenderFramework::new_with_plugin_render_features(
            Arc::new(ProjectAssetManager::default()),
            [descriptor],
            [RenderPassExecutorRegistration::new(
                "virtual-geometry.prepare",
                plugin_virtual_geometry_executor,
            )],
            Vec::new(),
        )
        .unwrap();
        let pipeline = plugin_virtual_geometry_pipeline();
        let handle = pipeline.handle;
        framework
            .state
            .lock()
            .unwrap()
            .pipelines
            .insert(handle, pipeline);

        reload_pipeline(&framework, handle)
            .expect("explicit plugin executor registration should satisfy the graph");
    }

    fn plugin_virtual_geometry_pipeline() -> RenderPipelineAsset {
        RenderPipelineAsset::default_forward_plus()
            .with_plugin_render_features([plugin_virtual_geometry_descriptor()])
    }

    fn plugin_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
        RenderFeatureDescriptor::new(
            "plugin.virtual_geometry.reload_asset",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plugin-virtual-geometry-reload-asset",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .with_side_effects()],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
    }

    fn plugin_virtual_geometry_executor(
        _context: &mut RenderPassExecutionContext<'_>,
    ) -> Result<(), String> {
        Ok(())
    }
}

use crate::core::framework::render::{RenderFrameworkError, RenderPipelineHandle};

use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn reload_pipeline(
    server: &WgpuRenderFramework,
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
    let compiled = compile_pipeline_for_validation(&pipeline_asset)?;
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.raw(),
            message,
        })?;
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::RenderFrameworkError;
    use crate::render_graph::QueueLane;
    use crate::{
        RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
    };

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
    fn reload_pipeline_accepts_plugin_executor_from_linked_descriptor() {
        let descriptor = plugin_virtual_geometry_descriptor();
        let framework = WgpuRenderFramework::new_with_plugin_render_features(
            Arc::new(ProjectAssetManager::default()),
            [descriptor],
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
            .expect("linked plugin descriptor should register its executor id");
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
}

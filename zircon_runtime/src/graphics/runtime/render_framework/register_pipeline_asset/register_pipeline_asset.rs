use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle, RenderWorldSnapshotHandle,
};
use crate::scene::world::World;
use crate::{CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn register_pipeline_asset(
    server: &WgpuRenderFramework,
    pipeline: RenderPipelineAsset,
) -> Result<RenderPipelineHandle, RenderFrameworkError> {
    let handle = pipeline.handle;
    let compiled = compile_pipeline_for_validation(&pipeline)?;

    let mut state = server.state.lock().unwrap();
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: handle.raw(),
            message,
        })?;
    state.pipelines.insert(handle, pipeline);
    Ok(handle)
}

pub(in crate::graphics::runtime::render_framework) fn compile_pipeline_for_validation(
    pipeline: &RenderPipelineAsset,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    pipeline
        .compile_with_options(&validation_extract(), &validation_compile_options(pipeline))
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.handle.raw(),
            message,
        })
}

fn validation_compile_options(pipeline: &RenderPipelineAsset) -> RenderPipelineCompileOptions {
    pipeline
        .renderer
        .features
        .iter()
        .filter(|feature| feature.enabled)
        .fold(
            RenderPipelineCompileOptions::default(),
            |mut options, feature| {
                if let Some(builtin) = feature.builtin_feature() {
                    options = options.with_feature_enabled(builtin);
                }
                if let Some(gate) = feature.quality_gate {
                    options = options.with_feature_enabled(gate);
                }
                for requirement in feature
                    .capability_requirements
                    .iter()
                    .chain(feature.descriptor().capability_requirements.iter())
                {
                    options = options.with_capability_enabled(*requirement);
                }
                options
            },
        )
}

fn validation_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(0),
        World::new().to_render_snapshot(),
    )
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

    use super::register_pipeline_asset;

    #[test]
    fn register_pipeline_asset_rejects_plugin_executor_without_linked_descriptor() {
        let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();

        let error = register_pipeline_asset(&framework, plugin_virtual_geometry_pipeline())
            .expect_err("unlinked plugin executor ids should not be accepted");

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
    fn register_pipeline_asset_accepts_plugin_executor_from_linked_descriptor() {
        let descriptor = plugin_virtual_geometry_descriptor();
        let framework = WgpuRenderFramework::new_with_plugin_render_features(
            Arc::new(ProjectAssetManager::default()),
            [descriptor],
            Vec::new(),
            Vec::new(),
        )
        .unwrap();

        let handle = register_pipeline_asset(&framework, plugin_virtual_geometry_pipeline())
            .expect("linked plugin descriptor should register its executor id");

        assert_eq!(handle, plugin_virtual_geometry_pipeline().handle);
    }

    fn plugin_virtual_geometry_pipeline() -> RenderPipelineAsset {
        RenderPipelineAsset::default_forward_plus()
            .with_plugin_render_features([plugin_virtual_geometry_descriptor()])
    }

    fn plugin_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
        RenderFeatureDescriptor::new(
            "plugin.virtual_geometry.registered_asset",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plugin-virtual-geometry-registered-asset",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .with_side_effects()],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
    }
}

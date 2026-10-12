use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::RenderFrameworkError;
use crate::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
};
use crate::graphics::{RenderPassExecutionContext, RenderPassExecutorRegistration};
use crate::render_graph::QueueLane;

use super::register_pipeline_asset;

#[test]
fn register_pipeline_asset_rejects_plugin_executor_without_linked_descriptor() {
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();

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
fn register_pipeline_asset_rejects_plugin_executor_from_descriptor_only() {
    let descriptor = plugin_virtual_geometry_descriptor();
    let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [descriptor],
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let error = register_pipeline_asset(&framework, plugin_virtual_geometry_pipeline())
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
fn register_pipeline_asset_accepts_plugin_executor_from_explicit_registration() {
    let descriptor = plugin_virtual_geometry_descriptor();
    let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [descriptor],
        [RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            plugin_virtual_geometry_executor,
        )],
        Vec::new(),
    )
    .unwrap();

    let handle = register_pipeline_asset(&framework, plugin_virtual_geometry_pipeline())
        .expect("explicit plugin executor registration should satisfy the graph");

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

fn plugin_virtual_geometry_executor(
    _context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    Ok(())
}

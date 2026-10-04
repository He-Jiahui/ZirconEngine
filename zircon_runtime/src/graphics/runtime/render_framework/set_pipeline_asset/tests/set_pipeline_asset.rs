use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFramework, RenderFrameworkError, RenderPipelineHandle, RenderViewportDescriptor,
};
use crate::core::math::UVec2;
use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
    RenderPipelineAsset, WgpuRenderFramework,
};
use crate::render_graph::QueueLane;

use super::set_pipeline_asset;

#[test]
fn set_pipeline_asset_compiles_outside_framework_state_lock() {
    let source = include_str!("../set_pipeline_asset.rs");
    let compile = source
        .find(concat!(
            "let compiled = compile_",
            "pipeline_for_validation"
        ))
        .expect("pipeline selection should compile the validation graph");
    let snapshot = source[..compile]
        .rfind(concat!("let (pipeline_asset, capabilities) = ", "{"))
        .expect("pipeline asset should be snapshotted in a short lock scope");
    let relock = compile
        + source[compile..]
            .find(concat!("let mut state = framework.", "lock_state();"))
            .expect("framework state should be reacquired after compilation");

    assert!(snapshot < compile && compile < relock);
}

#[test]
fn set_pipeline_asset_revalidates_stale_graph_executor_contract() {
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
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
        .lock_state()
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

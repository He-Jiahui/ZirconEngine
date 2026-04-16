use zircon_render_graph::QueueLane;
use zircon_scene::{RenderFrameExtract, RenderWorldSnapshotHandle, World};

use crate::{
    BuiltinRenderFeature, FrameHistoryBinding, FrameHistorySlot, RenderPipelineAsset,
    RenderPipelineCompileOptions,
};

#[test]
fn legacy_snapshot_adapter_initializes_m5_flagship_extract_slots_as_opt_in_empty() {
    let extract = test_extract();

    assert!(extract.geometry.virtual_geometry.is_none());
    assert!(extract.lighting.hybrid_global_illumination.is_none());
}

#[test]
fn default_forward_plus_pipeline_keeps_m5_flagship_features_opted_out() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"virtual-geometry-prepare"));
    assert!(!pass_names.contains(&"hybrid-gi-resolve"));
    assert!(!compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )));
}

#[test]
fn compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_async_compute(false),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"virtual-geometry-prepare"));
    assert!(pass_names.contains(&"hybrid-gi-resolve"));
    assert!(compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )));
    assert!(compiled.graph.passes().iter().any(|pass| {
        pass.name == "virtual-geometry-prepare" && pass.queue == QueueLane::Graphics
    }));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "hybrid-gi-resolve" && pass.queue == QueueLane::Graphics));
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::{debug_markers, runtime::WgpuRenderFramework};
use crate::scene::world::World;
use crate::{RenderPipelineAsset, RenderPipelineCompileOptions};

#[test]
fn render_framework_stats_report_transient_allocation_bytes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);
    let expected_pipeline = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_async_compute(false),
        )
        .unwrap();
    let expected_allocation_plan = expected_pipeline.graph.transient_allocation_plan();

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_graph_transient_texture_slot_count,
        expected_allocation_plan.texture_slot_count
    );
    assert_eq!(
        stats.last_graph_sparse_texture_slot_count,
        expected_allocation_plan.sparse_texture_slot_count
    );
    assert_eq!(
        stats.last_graph_transient_buffer_slot_count,
        expected_allocation_plan.buffer_slot_count
    );
    assert_eq!(
        stats.last_graph_transient_texture_bytes_reserved,
        expected_allocation_plan.dense_texture_bytes_reserved
    );
    assert_eq!(
        stats.last_graph_transient_buffer_bytes_reserved,
        expected_allocation_plan.dense_buffer_bytes_reserved
    );
    assert_eq!(
        stats.last_graph_transient_dense_bytes_reserved,
        expected_allocation_plan.total_dense_bytes_reserved()
    );
    assert_eq!(
        stats.last_graph_sparse_texture_virtual_bytes,
        expected_allocation_plan.sparse_texture_virtual_bytes
    );
}

#[test]
fn render_framework_stats_report_shadow_map_graph_execution() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert!(
        stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == "shadow-map"),
        "shadow-map should execute through the RenderFramework graph path; executed={:?}",
        stats.last_graph_executed_passes
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executor_id| executor_id == "shadow.map"),
        "shadow.map executor should be recorded by RenderFramework stats; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert_eq!(stats.last_shadow_graph_executed_pass_count, 1);
    assert!(
        stats
            .last_graph_executed_debug_markers
            .iter()
            .any(|marker| marker == &debug_markers::marker_for_render_graph_pass("shadow-map")),
        "shadow-map should emit a graph debug marker; markers={:?}",
        stats.last_graph_executed_debug_markers
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

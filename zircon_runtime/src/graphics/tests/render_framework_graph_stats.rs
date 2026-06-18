use std::{collections::BTreeSet, sync::Arc};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderGraphStageExecutionReport, RenderViewportDescriptor,
    RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::{
    debug_markers, runtime::WgpuRenderFramework, RenderPipelineAsset, RenderPipelineCompileOptions,
};
use crate::scene::world::World;

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
fn render_framework_stats_report_graph_execution_coverage() {
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
    let planned_live_pass_count = expected_pipeline
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .count();

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    let report = stats.last_graph_execution_coverage_report;

    assert_eq!(report.planned_live_pass_count, planned_live_pass_count);
    assert_eq!(
        report.executed_pass_count,
        stats.last_graph_executed_pass_count
    );
    assert_eq!(report.executed_pass_count, planned_live_pass_count);
    assert_eq!(report.matched_planned_pass_count, planned_live_pass_count);
    assert_eq!(report.missing_planned_pass_count, 0);
    assert_eq!(report.unexpected_executed_pass_count, 0);
    assert_eq!(report.duplicate_executed_pass_count, 0);
}

#[test]
fn render_framework_stats_report_graph_stage_execution() {
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
    let expected_report = live_stage_execution_report(&expected_pipeline);

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(stats.last_graph_stage_execution_report, expected_report);
    assert_eq!(
        stats
            .last_graph_stage_execution_report
            .stage_order_violation_count,
        0
    );
}

#[test]
fn render_framework_stats_report_shadow_atlas_graph_execution() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let pipeline = server.register_pipeline_asset(pipeline).unwrap();
    server.set_pipeline_asset(viewport, pipeline).unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert!(
        stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == "shadow-atlas"),
        "shadow-atlas should execute through the RenderFramework graph path; executed={:?}",
        stats.last_graph_executed_passes
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executor_id| executor_id == "shadow.atlas"),
        "shadow.atlas executor should be recorded by RenderFramework stats; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert_eq!(stats.last_shadow_graph_executed_pass_count, 1);
    assert!(
        stats
            .last_graph_executed_debug_markers
            .iter()
            .any(|marker| marker == &debug_markers::marker_for_render_graph_pass("shadow-atlas")),
        "shadow-atlas should emit a graph debug marker; markers={:?}",
        stats.last_graph_executed_debug_markers
    );
}

fn live_stage_execution_report(
    compiled_pipeline: &crate::graphics::pipeline::CompiledRenderPipeline,
) -> RenderGraphStageExecutionReport {
    let mut unique_stages = BTreeSet::new();
    let mut staged_pass_count = 0;
    let mut unstaged_pass_count = 0;
    let mut stage_transition_count = 0;
    let mut stage_order_violation_count = 0;
    let mut previous_stage = None;

    for pass in compiled_pipeline
        .graph
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
    {
        if let Some(stage) = compiled_pipeline.pass_stage(&pass.name) {
            staged_pass_count += 1;
            unique_stages.insert(stage);
            if let Some(previous) = previous_stage {
                if previous != stage {
                    stage_transition_count += 1;
                }
                if previous > stage {
                    stage_order_violation_count += 1;
                }
            }
            previous_stage = Some(stage);
        } else {
            unstaged_pass_count += 1;
            previous_stage = None;
        }
    }

    RenderGraphStageExecutionReport::new(
        staged_pass_count,
        unstaged_pass_count,
        unique_stages.len(),
        stage_transition_count,
        stage_order_violation_count,
    )
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

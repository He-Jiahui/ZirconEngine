use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFrameProfile, RenderFramework, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::{RenderPipelineAsset, debug_markers, runtime::WgpuRenderFramework};
use crate::scene::world::World;

#[test]
fn render_framework_stats_publish_transient_allocation_bytes_to_captured_profile() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);
    let (stats, capture) = submit_frame_with_capture(&server, viewport, extract);
    let profile = captured_frame_profile(&capture);

    assert_eq!(
        profile.transient_texture_peak_bytes,
        stats.last_graph_transient_texture_bytes_reserved
    );
    assert_eq!(
        profile.transient_buffer_peak_bytes,
        stats.last_graph_transient_buffer_bytes_reserved
    );
    assert_eq!(
        profile
            .transient_texture_peak_bytes
            .saturating_add(profile.transient_buffer_peak_bytes),
        stats.last_graph_transient_dense_bytes_reserved
    );
}

#[test]
fn render_framework_stats_report_graph_execution_coverage() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);
    let (stats, capture) = submit_frame_with_capture(&server, viewport, extract);
    let planned_live_pass_count =
        non_culled_pass_names_from_graph_dump(captured_graph_dump(&capture)).len();
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
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);
    let (stats, capture) = submit_frame_with_capture(&server, viewport, extract);
    let planned_live_pass_count =
        non_culled_pass_names_from_graph_dump(captured_graph_dump(&capture)).len();
    let report = stats.last_graph_stage_execution_report;

    assert_eq!(
        report.staged_pass_count + report.unstaged_pass_count,
        planned_live_pass_count
    );
    assert_eq!(
        report.staged_pass_count,
        stats.last_graph_executed_pass_count
    );
    assert!(report.unique_stage_count <= report.staged_pass_count);
    assert!(report.stage_transition_count <= report.staged_pass_count.saturating_sub(1));
    assert_eq!(report.stage_order_violation_count, 0);
}

#[test]
fn render_framework_stats_report_compiled_execution_batches() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);
    let (stats, capture) = submit_frame_with_capture(&server, viewport, extract);
    let planned_live_pass_count =
        non_culled_pass_names_from_graph_dump(captured_graph_dump(&capture)).len();
    let report = stats.last_graph_execution_batch_report;

    assert_eq!(report.planned_live_pass_count, planned_live_pass_count);
    assert_eq!(
        report.graphics_batch_count
            + report.async_compute_batch_count
            + report.async_copy_batch_count,
        report.planned_batch_count
    );
    assert!(report.planned_batch_count > 0);
    assert!(report.max_passes_per_batch > 0);
    assert!(report.queue_transition_count <= report.planned_batch_count.saturating_sub(1));
}

#[test]
fn render_graph_steady_state_second_frame_skips_recompile() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);

    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let first = server.query_stats().unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let second = server.query_stats().unwrap();

    assert!(first.last_graph_compiled_cache_miss_count > 0);
    assert_eq!(
        second.last_graph_compiled_cache_miss_count, first.last_graph_compiled_cache_miss_count,
        "an unchanged second frame must not recompile the graph"
    );
    assert_eq!(
        second.last_graph_compiled_cache_hit_count,
        first.last_graph_compiled_cache_hit_count + 1,
        "the unchanged second frame must reuse the compiled graph"
    );
    assert_eq!(
        second.last_graph_compiled_cache_entry_count,
        first.last_graph_compiled_cache_entry_count
    );
}

#[test]
fn render_perf_pass_names_match_graph_dump_and_markers() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let mut extract = test_extract();
    extract.apply_viewport_size(viewport_size);
    let (stats, capture) = submit_frame_with_capture(&server, viewport, extract);
    let expected_pass_names = non_culled_pass_names_from_graph_dump(captured_graph_dump(&capture));
    let capture_profile = captured_frame_profile(&capture);
    let expected_markers = expected_pass_names
        .iter()
        .map(|pass_name| debug_markers::marker_for_render_graph_pass(pass_name))
        .collect::<Vec<_>>();

    assert_eq!(stats.last_graph_executed_debug_markers, expected_markers);
    assert_eq!(
        capture_profile
            .passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        expected_pass_names
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        capture_profile
            .passes
            .iter()
            .map(|pass| debug_markers::marker_for_render_graph_pass(&pass.pass_name))
            .collect::<Vec<_>>(),
        stats.last_graph_executed_debug_markers,
    );
}

fn non_culled_pass_names_from_graph_dump(graph_dump: &str) -> Vec<String> {
    graph_dump
        .lines()
        .filter_map(|line| {
            let pass_row = line.strip_prefix("  pass[")?;
            let (_, pass_row) = pass_row.split_once(" name=")?;
            let (name, pass_row) = pass_row.split_once(" layer=")?;
            let (_, pass_row) = pass_row.split_once(" queue=")?;
            pass_row.contains(" culled=false ").then(|| name.to_owned())
        })
        .collect()
}

#[test]
fn render_framework_graph_dump_parser_preserves_names_after_topology_layers() {
    let graph_dump = concat!(
        "render_graph name=unit passes=2 executable=1 culled=1 resources=0 topology_layers=1 topology_peak_width=1\n",
        "passes:\n",
        "  pass[0] id=3 name=draw-ui layer=0 queue=Graphics declared_queue=Graphics fallback=false culled=false executor=- deps=- resources=0\n",
        "  pass[1] id=4 name=culled-bloom layer=- queue=Graphics declared_queue=Graphics fallback=false culled=true executor=- deps=- resources=0\n",
    );

    assert_eq!(
        non_culled_pass_names_from_graph_dump(graph_dump),
        vec!["draw-ui".to_owned()]
    );
}

fn submit_frame_with_capture(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (RenderStats, CapturedFrame) {
    server.request_graphics_debugger_capture(viewport).unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    let capture = server
        .capture_frame(viewport)
        .unwrap()
        .expect("requested graph/profile capture should be available");
    (stats, capture)
}

fn captured_graph_dump(capture: &CapturedFrame) -> &str {
    capture
        .graph_dump
        .as_deref()
        .expect("capture should retain the runtime graph dump")
}

fn captured_frame_profile(capture: &CapturedFrame) -> RenderFrameProfile {
    serde_json::from_str(
        capture
            .frame_profile_json
            .as_deref()
            .expect("capture should retain the runtime frame profile"),
    )
    .expect("captured frame profile should remain decodable")
}

#[test]
fn render_framework_stats_report_shadow_atlas_graph_execution() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
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

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

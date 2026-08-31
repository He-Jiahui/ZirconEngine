#![cfg(feature = "profiling")]

use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
#[cfg(feature = "profiling-chrome")]
use crate::core::diagnostics::profiling::{
    PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE, export_report, stop_capture,
};
use crate::core::diagnostics::profiling::{
    ProfileCaptureConfig, reset_capture, snapshot, start_capture, test_capture_lock,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    UiRenderSubmission,
};
#[cfg(feature = "ui")]
use crate::core::framework::render::{RenderPipelineHandle, RenderQualityProfile};
use crate::core::math::UVec2;
use crate::graphics::{ViewportRenderFrame, runtime::WgpuRenderFramework};
use crate::scene::world::World;
use zircon_runtime_interface::ProfileSnapshot;
#[cfg(feature = "ui")]
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
#[cfg(feature = "ui")]
use zircon_runtime_interface::ui::layout::UiFrame;
#[cfg(feature = "ui")]
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};

#[cfg(feature = "ui")]
const UI_TEXT_PROFILE_SETTLE_MAX_FRAMES: usize = 120;
#[cfg(feature = "ui")]
const UI_TEXT_PROFILE_SETTLE_FRAME_DELAY_MILLIS: u64 = 2;
#[cfg(feature = "profiling-chrome")]
const RUNTIME_TEXT_PROFILE_WORK_DIRECTORY: &str = ".runtime_text_profile_work";

#[cfg(all(feature = "ui", feature = "profiling-chrome", target_os = "windows"))]
#[path = "render_profiling/text_baseline.rs"]
mod text_baseline;

#[test]
fn render_submit_records_render_graph_pass_and_wait_spans() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "render-submit-m4-spans".to_string();
    config.max_spans = 256;
    start_capture(config);

    framework
        .submit_frame_extract(viewport, test_extract())
        .unwrap();

    let snapshot = snapshot();
    reset_capture();
    assert_span(&snapshot, "render_framework.wait", "operation_lock");
    assert_span(&snapshot, "render_framework.wait", "state");
    assert_span(&snapshot, "render_graph.stage", "DepthPrepass");
    assert_span(&snapshot, "render_graph.pass", "depth-prepass");
    assert!(
        snapshot.spans.iter().any(|span| {
            span.category == "render_graph.pass"
                && span.name == "depth-prepass"
                && span.path
                    == "runtime/render_framework:submit_frame_extract/render_framework:render_frame_with_pipeline/render_graph.stage:DepthPrepass/render_graph.pass:depth-prepass"
        }),
        "render graph pass span should be nested under its runtime submit and stage path"
    );
}

#[test]
fn direct_runtime_frame_submit_nests_render_graph_spans_under_pipeline_scope() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "runtime-frame-m4-spans".to_string();
    config.max_spans = 256;
    start_capture(config);

    framework
        .submit_runtime_frame(
            viewport,
            ViewportRenderFrame::from_extract(test_extract(), UVec2::new(320, 240)),
        )
        .unwrap();

    let snapshot = snapshot();
    reset_capture();
    assert_span(&snapshot, "render_framework", "build_submission_context");
    assert_span(&snapshot, "render_framework", "prepare_runtime_submission");
    assert_span(&snapshot, "render_framework", "render_frame_with_pipeline");
    assert_span(&snapshot, "render_framework", "collect_runtime_feedback");
    assert!(
        snapshot.spans.iter().any(|span| {
            span.category == "render_graph.pass"
                && span.name == "depth-prepass"
                && span.path
                    == "runtime/render_framework:submit_runtime_frame/render_framework:render_frame_with_pipeline/render_graph.stage:DepthPrepass/render_graph.pass:depth-prepass"
        }),
        "direct runtime frame submit should nest graph pass spans under submit_runtime_frame/render_frame_with_pipeline"
    );
}

#[cfg(feature = "ui")]
#[test]
fn ui_text_prepare_profiles_mixed_native_and_sdf_batches() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(320, 160);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("ui-text-profile-contract")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    let ui = Arc::new(mixed_text_ui_extract());
    let mut warm_stats = None;
    for frame_index in 0..UI_TEXT_PROFILE_SETTLE_MAX_FRAMES {
        framework
            .submit_frame_extract_with_ui(
                viewport,
                test_extract(),
                Some(UiRenderSubmission::single(ui.clone())),
            )
            .unwrap();
        let stats = framework.query_stats().unwrap();
        if native_text_raster_is_settled(&stats) {
            warm_stats = Some(stats);
            break;
        }
        if frame_index + 1 < UI_TEXT_PROFILE_SETTLE_MAX_FRAMES {
            std::thread::sleep(std::time::Duration::from_millis(
                UI_TEXT_PROFILE_SETTLE_FRAME_DELAY_MILLIS,
            ));
        }
    }
    let warm_stats = warm_stats.expect(
        "mixed UI text profile capture requires a settled native raster frame before profiling",
    );
    assert!(warm_stats.last_ui_text_visible_raster_glyph_count > 0);
    assert!(warm_stats.last_ui_text_raster_source_image_count > 0);

    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-mixed-prepare-profile".to_string();
    config.max_spans = 512;
    config.max_counters = 128;
    start_capture(config);

    framework
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(UiRenderSubmission::single(ui)),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();
    let profile = snapshot();
    reset_capture();

    assert_eq!(stats.last_ui_text_payload_count, 2);
    assert_span(&profile, "ui_text.prepare", "screen_space_ui_text");
    assert_span(
        &profile,
        "ui_text.native_raster_plan",
        "native_text_prepare",
    );
    assert_category_span(&profile, "ui_text.atlas_upload");
    assert_span(&profile, "ui_text.sdf_prepare", "sdf_atlas_plan");
    assert_span(&profile, "ui_text.sdf_prepare", "sdf_renderer_prepare");
    assert_counter_value(&profile, "ui_text.prepare.input_batches", 2.0);
    assert_counter_value(&profile, "ui_text.prepare.resolved_native_batches", 1.0);
    assert_counter_value(&profile, "ui_text.prepare.resolved_sdf_batches", 1.0);
    assert_counter_greater_than(
        &profile,
        "ui_text.native_raster_plan.worker_font_resident_bytes",
        0.0,
    );
    assert_counter_greater_than(
        &profile,
        "ui_text.native_raster_plan.worker_font_resident_entries",
        0.0,
    );
    assert_counter_greater_than(&profile, "ui_text.atlas_upload.native_instances", 0.0);
    assert_counter_greater_than(&profile, "ui_text.atlas_upload.native_draws", 0.0);
    assert_counter_value(&profile, "ui_text.sdf_prepare.text_batches", 1.0);
}

#[cfg(feature = "profiling-chrome")]
#[test]
fn direct_runtime_frame_submit_exports_perfetto_trace_artifacts() {
    let _guard = test_capture_lock();
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let output_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest must have a workspace parent")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join(RUNTIME_TEXT_PROFILE_WORK_DIRECTORY)
        .join(format!(
            "zircon-runtime-f3-trace-export-{}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&output_root);
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "runtime-frame-f3-trace-export".to_string();
    config.output_root = output_root.to_string_lossy().into_owned();
    config.max_spans = 256;
    config.include_perfetto = true;
    start_capture(config);

    framework
        .submit_runtime_frame(
            viewport,
            ViewportRenderFrame::from_extract(test_extract(), UVec2::new(320, 240)),
        )
        .unwrap();

    stop_capture();
    let report = export_report().expect("export profiling trace report");
    reset_capture();

    assert_profile_file(&report.files, PROFILE_TIMELINE_NATIVE_FILE);
    assert_profile_file(&report.files, PROFILE_TIMELINE_PERFETTO_FILE);
    assert_profile_file(&report.files, PROFILE_HOTSPOTS_FILE);
    assert_profile_file(&report.files, PROFILE_SUMMARY_FILE);

    let export_dir = std::path::Path::new(&report.export_dir);
    assert!(
        export_dir.starts_with(&output_root),
        "profile trace exports must stay under the workspace text artifact root"
    );
    let native_trace = std::fs::read_to_string(export_dir.join(PROFILE_TIMELINE_NATIVE_FILE))
        .expect("read native profile trace");
    let perfetto_trace = std::fs::read_to_string(export_dir.join(PROFILE_TIMELINE_PERFETTO_FILE))
        .expect("read Perfetto profile trace");
    let summary =
        std::fs::read_to_string(export_dir.join(PROFILE_SUMMARY_FILE)).expect("read summary");

    for required_trace_anchor in [
        "submit_runtime_frame",
        "render_frame_with_pipeline",
        "DepthPrepass",
        "depth-prepass",
    ] {
        assert!(
            native_trace.contains(required_trace_anchor),
            "native trace should retain Runtime 07 F3 anchor `{required_trace_anchor}`"
        );
        assert!(
            perfetto_trace.contains(required_trace_anchor),
            "Perfetto trace should retain Runtime 07 F3 anchor `{required_trace_anchor}`"
        );
    }
    assert!(
        summary.contains("runtime-frame-f3-trace-export"),
        "profile summary should identify the exported Runtime 07 F3 trace session"
    );

    let _ = std::fs::remove_dir_all(output_root);
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(501),
        World::new().to_render_snapshot(),
    )
}

fn assert_span(snapshot: &ProfileSnapshot, category: &str, name: &str) {
    assert!(
        snapshot
            .spans
            .iter()
            .any(|span| span.category == category && span.name == name),
        "expected span {category}:{name}, captured spans: {:?}",
        snapshot
            .spans
            .iter()
            .map(|span| format!("{}:{}", span.category, span.name))
            .collect::<Vec<_>>()
    );
}

#[cfg(feature = "ui")]
fn assert_category_span(snapshot: &ProfileSnapshot, category: &str) {
    assert!(
        snapshot.spans.iter().any(|span| span.category == category),
        "expected a span in {category}, captured spans: {:?}",
        snapshot
            .spans
            .iter()
            .map(|span| format!("{}:{}", span.category, span.name))
            .collect::<Vec<_>>(),
    );
}

#[cfg(feature = "ui")]
fn assert_counter_value(snapshot: &ProfileSnapshot, name: &str, expected: f64) {
    assert_eq!(
        snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value),
        Some(expected),
        "expected fixed counter {name}",
    );
}

#[cfg(feature = "ui")]
fn assert_counter_greater_than(snapshot: &ProfileSnapshot, name: &str, lower_bound: f64) {
    assert!(
        snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .is_some_and(|counter| counter.value > lower_bound),
        "expected profile counter {name} > {lower_bound}, captured counters: {:?}",
        snapshot
            .counters
            .iter()
            .filter(|counter| counter.name == name)
            .map(|counter| counter.value)
            .collect::<Vec<_>>(),
    );
}

#[cfg(feature = "ui")]
fn native_text_raster_is_settled(stats: &crate::core::framework::render::RenderStats) -> bool {
    stats.last_ui_text_visible_raster_glyph_count > 0
        && stats.last_ui_text_raster_source_image_count > 0
        && stats.last_ui_text_raster_worker_pending_count == 0
        && stats.last_ui_text_raster_worker_failed_count == 0
        && stats.last_ui_text_visible_missing_raster_image_count == 0
        && stats.last_ui_text_visible_raster_placeholder_count == 0
        && stats.last_ui_text_raster_renderer_upload_requeued_count == 0
        && stats.last_ui_text_raster_renderer_upload_failure_count == 0
        && stats.last_ui_text_sdf_generation_pending_batch_count == 0
        && stats.last_ui_text_sdf_generation_completion_backlog_count == 0
        && stats.last_ui_text_sdf_generation_failure_count == 0
}

#[cfg(feature = "ui")]
fn mixed_text_ui_extract() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.text.mixed-profile"),
        list: UiRenderList {
            commands: vec![
                profile_text_command(
                    1,
                    UiFrame::new(16.0, 18.0, 260.0, 32.0),
                    "Native profiler text",
                    UiTextRenderMode::Native,
                ),
                profile_text_command(
                    2,
                    UiFrame::new(16.0, 76.0, 260.0, 44.0),
                    "SDF profiler text",
                    UiTextRenderMode::Sdf,
                ),
            ],
        },
        raster_scale: 1.0,
    }
}

#[cfg(feature = "ui")]
fn profile_text_command(
    node_id: u64,
    frame: UiFrame,
    text: &str,
    text_render_mode: UiTextRenderMode,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            foreground_color: Some("#f5f7fb".to_string()),
            font_size: 18.0,
            line_height: 24.0,
            text_align: UiTextAlign::Start,
            wrap: UiTextWrap::None,
            text_render_mode,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}

#[cfg(feature = "profiling-chrome")]
fn assert_profile_file(files: &[String], expected_file: &str) {
    assert!(
        files.iter().any(|file| file == expected_file),
        "profile export should include `{expected_file}`, files: {files:?}"
    );
}

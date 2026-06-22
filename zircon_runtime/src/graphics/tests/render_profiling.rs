#![cfg(feature = "profiling")]

use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
#[cfg(feature = "profiling-chrome")]
use crate::core::diagnostics::profiling::{
    export_report, stop_capture, PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE,
    PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE,
};
use crate::core::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::{runtime::WgpuRenderFramework, ViewportRenderFrame};
use crate::scene::world::World;
use zircon_runtime_interface::ProfileSnapshot;

#[test]
fn render_submit_records_render_graph_pass_and_wait_spans() {
    let _guard = test_capture_lock();
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
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
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
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

#[cfg(feature = "profiling-chrome")]
#[test]
fn direct_runtime_frame_submit_exports_perfetto_trace_artifacts() {
    let _guard = test_capture_lock();
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let output_root = std::env::temp_dir().join(format!(
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

#[cfg(feature = "profiling-chrome")]
fn assert_profile_file(files: &[String], expected_file: &str) {
    assert!(
        files.iter().any(|file| file == expected_file),
        "profile export should include `{expected_file}`, files: {files:?}"
    );
}

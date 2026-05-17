#![cfg(feature = "profiling")]

use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
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

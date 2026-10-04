use crate::core::math::UVec2;
use crate::graphics::backend::{OffscreenTarget, RenderBackend};
use crate::graphics::scene::scene_renderer::core::DEPTH_FORMAT;
use crate::graphics::scene::scene_renderer::core::SCENE_COLOR_HDR_FORMAT;

use super::*;

#[test]
fn scene_region_clear_resources_build_for_offscreen_backend() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
    let resources =
        SceneRegionClearResources::new(&backend.device, SCENE_COLOR_HDR_FORMAT, DEPTH_FORMAT);
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-region-clear-test-encoder"),
        });

    let color_uploads = resources.record(
        &mut encoder,
        &target.scene_color_view,
        &target.depth_view,
        ViewportRenderRegion::full_target(target.size),
        Some(Vec4::ONE),
        true,
    );
    assert!(!color_uploads.is_empty());

    let depth_only_uploads = resources.record(
        &mut encoder,
        &target.scene_color_view,
        &target.depth_view,
        ViewportRenderRegion::full_target(target.size),
        None,
        true,
    );
    assert!(depth_only_uploads.is_empty());

    let _upload_submission = backend
        .enqueue_copy_buffer_upload_batch(color_uploads)
        .unwrap();
    backend.queue.submit([encoder.finish()]);
}

#[test]
fn scene_region_clear_defers_color_upload_to_frame_transaction() {
    let source = include_str!("../scene_region_clear_resources.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let stage_source = include_str!(
        "../../core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs"
    );
    let frame_source =
        include_str!("../../core/scene_renderer_core_render_compiled_scene/render/render.rs");

    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(!production.contains("queue.write_buffer("));
    assert!(!production.contains("queue: &wgpu::Queue"));

    let clear_record = stage_source
        .find("let mut scene_clear_uploads = scene_clear.record_frame_clear(")
        .expect("scene clear must prepare its color upload while recording the clear draw");
    let graph_append = stage_source
        .find("graph_execution.append_buffer_uploads(&mut scene_clear_uploads)")
        .expect("scene clear upload must join graph-owned pending uploads");
    assert!(clear_record < graph_append);

    let graph_success = frame_source
        .find("let mut graph_buffer_uploads = graph_execution.take_buffer_uploads()")
        .expect("frame owner must retain graph uploads only after graph success");
    let upload_accept = frame_source
        .find(".enqueue_copy_resource_upload_batch(")
        .expect("frame owner must accept one merged buffer upload batch");
    assert!(graph_success < upload_accept);
}

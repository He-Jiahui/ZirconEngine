use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_draws_reuse_segment_local_vertex_buffers() {
    let renderer = read_runtime_src("graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs");
    let construct = read_runtime_src("graphics/scene/scene_renderer/ui/construct.rs");
    let render = read_runtime_src("graphics/scene/scene_renderer/ui/render.rs");
    let record = read_runtime_src("graphics/scene/scene_renderer/ui/render/record.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/render/tests.rs");
    let plan_cache_tests =
        read_runtime_src("graphics/scene/scene_renderer/ui/render/tests/plan_cache.rs");

    assert_contains_all(
        "screen-space UI renderer owns reusable segment-local vertex buffer state",
        &renderer,
        &[
            "vertex_segments: Vec<ScreenSpaceUiVertexSegmentBuffer>",
            "buffer: Option<wgpu::Buffer>",
            "capacity_bytes: u64",
            "payload_hash: Option<[u8; 32]>",
            "plan: Option<Weak<PlannedScreenSpaceUi>>",
        ],
    );
    assert_contains_all(
        "screen-space UI construction initializes reusable segment buffers",
        &construct,
        &["vertex_segments: Vec::new()"],
    );
    assert_contains_all(
        "screen-space UI planning retains vertices for record-time upload",
        &render,
        &[
            "vertices: Vec<ScreenSpaceUiVertex>",
            "render_segments: Arc<[Arc<PlannedScreenSpaceUi>]>",
            "append_non_render_payload_cloned",
        ],
    );
    assert_eq!(
        render.matches("device.create_buffer_init(").count(),
        0,
        "screen-space UI planning must not allocate a vertex buffer per frame"
    );
    assert_contains_all(
        "screen-space UI record owns segment-local dynamic buffer uploads",
        &record,
        &[
            "fn write_screen_space_ui_vertex_buffers(",
            "prepared.render_segments.iter().zip(&self.vertex_segments)",
            "segment.vertices.as_slice()",
            "vertex_segment.buffer.is_none()",
            "screen_space_ui_vertex_segment_plan_reused(",
            "screen_space_ui_vertex_buffer_capacity(required_byte_len)",
            "wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST",
            "screen_space_ui_vertex_buffer_write_required(",
            "uploads.push(WgpuBufferUpload::from_bytes(",
            "force_full_upload: bool",
            "vertex_segment.payload_hash = Some(payload_hash);",
            "vertex_segment.buffer.as_ref()",
        ],
    );
    assert!(!record.contains("queue.write_buffer("));
    assert!(
        !record.contains("prepared.vertices.as_slice()"),
        "screen-space UI record must not hash or upload one flattened prepared vertex payload"
    );
    assert_eq!(
        record
            .matches("device.create_buffer(&wgpu::BufferDescriptor {")
            .count(),
        1,
        "screen-space UI record must have one controlled segment-buffer allocation path"
    );
    assert!(
        !record.contains("&& let"),
        "screen-space UI record must remain compatible with the Rust 2021 workspace edition"
    );
    assert_contains_all(
        "screen-space UI dynamic buffer behavior test",
        &tests,
        &[
            "screen_space_ui_vertex_buffer_writes_only_for_new_payloads_or_reallocation",
            "record::screen_space_ui_vertex_buffer_write_required(",
        ],
    );
    assert_contains_all(
        "screen-space UI segment vertex plan identity test",
        &plan_cache_tests,
        &["screen_space_ui_vertex_segment_plan_reuse_requires_exact_segment_identity"],
    );

    for (path, source) in [
        ("scene_renderer/ui/render.rs", render.as_str()),
        ("scene_renderer/ui/render/record.rs", record.as_str()),
    ] {
        assert!(
            source.lines().count() < 800,
            "{path} should stay under the R1.4 owner budget"
        );
    }
}

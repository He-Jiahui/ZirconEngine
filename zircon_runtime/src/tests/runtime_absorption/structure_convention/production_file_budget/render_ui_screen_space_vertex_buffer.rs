use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_draws_reuse_a_dynamic_vertex_buffer() {
    let renderer = read_runtime_src("graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs");
    let construct = read_runtime_src("graphics/scene/scene_renderer/ui/construct.rs");
    let render = read_runtime_src("graphics/scene/scene_renderer/ui/render.rs");
    let record = read_runtime_src("graphics/scene/scene_renderer/ui/render/record.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/render/tests.rs");

    assert_contains_all(
        "screen-space UI renderer owns its reusable vertex buffer state",
        &renderer,
        &[
            "vertex_buffer: Option<wgpu::Buffer>",
            "vertex_buffer_capacity_bytes: u64",
            "vertex_buffer_payload_hash: Option<[u8; 32]>",
        ],
    );
    assert_contains_all(
        "screen-space UI construction initializes reusable vertex buffer state",
        &construct,
        &[
            "vertex_buffer: None",
            "vertex_buffer_capacity_bytes: 0",
            "vertex_buffer_payload_hash: None",
        ],
    );
    assert_contains_all(
        "screen-space UI planning retains vertices for record-time upload",
        &render,
        &[
            "vertices: Vec<ScreenSpaceUiVertex>",
            "vertices: plan.vertices",
        ],
    );
    assert_eq!(
        render.matches("device.create_buffer_init(").count(),
        0,
        "screen-space UI planning must not allocate a vertex buffer per frame"
    );
    assert_contains_all(
        "screen-space UI record owns the dynamic buffer upload",
        &record,
        &[
            "fn write_screen_space_ui_vertex_buffer(",
            "prepared.vertices.as_slice()",
            "self.vertex_buffer.is_none()",
            "screen_space_ui_vertex_buffer_capacity(required_byte_len)",
            "wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST",
            "screen_space_ui_vertex_buffer_write_required(",
            "queue.write_buffer(vertex_buffer, 0, vertex_bytes);",
            "self.vertex_buffer_payload_hash = Some(payload_hash);",
            "self.vertex_buffer.as_ref()",
        ],
    );
    assert_eq!(
        record
            .matches("device.create_buffer(&wgpu::BufferDescriptor {")
            .count(),
        1,
        "screen-space UI record must have one controlled reusable-buffer allocation path"
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

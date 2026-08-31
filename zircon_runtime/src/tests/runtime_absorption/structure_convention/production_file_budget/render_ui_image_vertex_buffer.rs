use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_images_reuse_a_single_vertex_buffer() {
    let image = read_runtime_src("graphics/scene/scene_renderer/ui/image.rs");
    let image_tests = read_runtime_src("graphics/scene/scene_renderer/ui/image/tests.rs");
    let record = read_runtime_src("graphics/scene/scene_renderer/ui/render/record.rs");

    assert_contains_all(
        "screen-space UI image vertex buffer reuse",
        &image,
        &[
            "image_vertices: ScreenSpaceUiImageVertexBuffer",
            "struct ScreenSpaceUiImageVertexBuffer",
            "buffer: Option<wgpu::Buffer>",
            "capacity_bytes: u64",
            "payload_hash: Option<[u8; 32]>",
            "vertices: Vec<ScreenSpaceUiImageVertex>",
            "fn write_screen_space_ui_image_vertex_buffer(",
            "fn image_vertex_buffer_requires_reallocation(",
            "fn image_vertex_buffer_write_required(",
            "if image_vertices.buffer.is_none()",
            "image_vertex_buffer_requires_reallocation(",
            "device.create_buffer(&wgpu::BufferDescriptor {",
            "wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST",
            "uploads.push(WgpuBufferUpload::from_bytes(",
            "force_full_upload: bool",
            "image_vertices.payload_hash = Some(payload_hash);",
            "vertex_range: Range<u32>",
            "pass.set_vertex_buffer(0, vertex_buffer.slice(..));",
            "pass.draw(image.vertex_range.clone(), 0..1);",
            "#[cfg(test)]\nmod tests;",
        ],
    );
    assert_eq!(
        image.matches("device.create_buffer_init(").count(),
        0,
        "screen-space UI images must not create one vertex buffer per batch"
    );
    assert_eq!(
        image
            .matches("device.create_buffer(&wgpu::BufferDescriptor {")
            .count(),
        1,
        "screen-space UI image vertex buffers must have one controlled allocation path"
    );
    assert!(
        image.matches("image_vertices.vertices.clear();").count() >= 2,
        "both empty-streamer and normal preparation paths must clear retained image vertices"
    );
    assert_contains_all(
        "screen-space UI record attaches image writes to its prepared batch",
        &record,
        &[
            "self.image_system.prepare(",
            "prepared_upload.uploads_mut(),",
            "force_full_upload,",
        ],
    );
    assert!(!image.contains("queue.write_buffer("));
    assert_contains_all(
        "screen-space UI image vertex buffer capacity tests",
        &image_tests,
        &[
            "screen_space_ui_image_vertex_buffer_uses_a_small_minimum_growth_capacity",
            "screen_space_ui_image_vertex_buffer_reallocates_only_when_required",
            "screen_space_ui_image_vertex_buffer_writes_only_for_new_payloads_or_reallocation",
        ],
    );

    assert!(
        image.lines().count() < 800,
        "scene_renderer/ui/image.rs should stay under the R1.4 owner budget"
    );
}

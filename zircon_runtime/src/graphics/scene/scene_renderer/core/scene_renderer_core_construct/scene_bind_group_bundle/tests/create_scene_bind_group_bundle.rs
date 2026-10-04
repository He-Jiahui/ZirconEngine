#[test]
fn scene_sh9_default_uses_cold_mapped_initialization() {
    let production = include_str!("../create_scene_bind_group_bundle.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene bind group construction must retain a test boundary");

    assert!(production.contains("device.create_buffer_init("));
    assert!(production.contains("contents: bytemuck::bytes_of(&environment_sh9)"));
    assert!(production.contains("wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST"));
    assert!(!production.contains("queue.write_buffer("));
}

#[test]
fn scene_bundle_consumes_system_textures_without_queue_authority() {
    let production = include_str!("../create_scene_bind_group_bundle.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene bind group construction must retain a test boundary");

    assert!(production.contains("SceneEnvironmentCubemap::fallback(system_textures)"));
    assert!(production.contains("SceneEnvironmentBrdfLut::from_system_textures(system_textures)"));
    assert!(!production.contains("queue: &wgpu::Queue"));
    assert!(!production.contains("queue.write_texture"));
}

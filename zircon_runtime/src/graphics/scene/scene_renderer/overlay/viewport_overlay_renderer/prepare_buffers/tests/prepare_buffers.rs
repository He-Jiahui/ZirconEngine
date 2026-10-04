const SOURCE: &str = include_str!("../prepare_buffers.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("overlay preparation should retain a test-module boundary")
}

#[test]
fn disabled_interaction_overlays_skip_cpu_vertex_generation() {
    let source = production_source();
    let interaction_guard = source
        .find("let Some(interaction_overlays) = self.interaction_overlays.as_mut() else {")
        .expect("overlay preparation should exit when interaction resources are absent");
    let selection_generation = source
        .find("build_selection_vertices")
        .expect("interactive overlays should still prepare selection geometry");

    assert!(
        interaction_guard < selection_generation,
        "the EnvironmentOnly path must exit before generating interaction vertices"
    );
    assert!(source.contains("selection_buffer: None"));
    assert!(source.contains("wireframe_buffer: None"));
    assert!(source.contains("handle_buffer: None"));
}

#[test]
fn viewport_icon_uploads_join_the_existing_frame_texture_batch_without_a_queue() {
    let source = production_source();

    assert!(source.contains("frame_texture_uploads: &mut WgpuTextureUploadBatch"));
    assert!(source.contains("frame_texture_uploads,"));
    assert!(!source.contains("queue: &wgpu::Queue"));
}

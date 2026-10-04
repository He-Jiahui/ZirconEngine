#[test]
fn ssao_binding_prepares_upload_without_touching_the_queue() {
    let source = include_str!("../bind_ssao_compute_graph_resources.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("compiled graph SSAO resource binding source");

    assert!(!production.contains("queue: &wgpu::Queue"));
    assert!(production.contains("prepare_ssao_compute_params_upload("));
    assert!(production.contains("pipeline.ambient_occlusion_profile()"));
    assert!(production.contains("target.render_size"));
    assert!(production.contains("import_borrowed_buffer_with_physical_desc("));
    assert!(production.contains("ssao_params_buffer_desc("));
    assert!(production.contains("frame_buffer_uploads"));
    assert!(!production.contains("HISTORY_PREVIOUS_AMBIENT_OCCLUSION"));
    assert!(!production.contains("white_texture_view"));
}

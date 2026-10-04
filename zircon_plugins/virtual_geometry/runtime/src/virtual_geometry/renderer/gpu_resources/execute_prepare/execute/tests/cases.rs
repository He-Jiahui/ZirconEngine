#[test]
fn virtual_geometry_prepare_params_use_a_queue_free_upload_sink() {
    let execute = include_str!("../execute.rs");
    let params = include_str!("../queue_params.rs");

    assert!(execute.contains("buffer_uploads: &mut dyn RenderPassBufferUploadSink"));
    assert!(params.contains("buffer_uploads.write_buffer("));
    assert!(!execute.contains("queue: &wgpu::Queue"));
    assert!(!params.contains("queue.write_buffer"));
}

#[test]
fn transparent_params_follow_the_render_pass_upload_recorder_chain() {
    let transparent = include_str!("../transparent.rs")
        .split_once("#[cfg(test)]")
        .map(|(product, _)| product)
        .expect("transparent renderer should retain a test boundary");
    let backend = include_str!("../backend.rs");
    let runtime_owner = include_str!("../runtime_owner.rs");
    let executors = include_str!("../../executors.rs");

    assert!(transparent.contains("RenderPassBufferUploadSink"));
    assert!(!transparent.contains("queue.write_buffer"));
    assert!(transparent.contains("write_buffer(&self.render_params_buffer, 0, &params.encode())"));

    for source in [backend, runtime_owner] {
        let transparent_record = source
            .split_once("pub fn record_transparent_render")
            .map(|(_, tail)| tail)
            .expect("particle transparent upload chain");
        let signature = transparent_record
            .split_once('{')
            .map(|(signature, _)| signature)
            .expect("particle transparent record signature");
        assert!(!signature.contains("wgpu::Queue"));
        assert!(signature.contains("RenderPassBufferUploadSink"));
    }
    let executor = executors
        .split_once("fn record_particle_gpu_transparent")
        .map(|(_, tail)| tail)
        .expect("particle transparent executor");
    let executor_signature = executor
        .split_once('{')
        .map(|(signature, _)| signature)
        .expect("particle transparent executor signature");
    assert!(!executor_signature.contains("wgpu::Queue"));
    assert!(executor.contains("&mut draw.buffer_uploads"));
    assert!(executors.contains("gpu.plugin_outputs_mut().particles"));
    assert!(!executors.contains("gpu.plugin_outputs.particles"));
}

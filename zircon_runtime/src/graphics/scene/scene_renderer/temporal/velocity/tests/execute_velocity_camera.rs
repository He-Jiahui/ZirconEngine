#[test]
fn velocity_camera_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../execute_velocity_camera.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("velocity camera production source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}

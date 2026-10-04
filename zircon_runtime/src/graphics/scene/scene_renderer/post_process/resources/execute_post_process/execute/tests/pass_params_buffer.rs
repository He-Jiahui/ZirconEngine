#[test]
fn post_process_pass_params_use_persistent_slots_and_pre_submit_uploads() {
    let source = include_str!("../pass_params_buffer.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("post-process pass params production source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("device.create_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}

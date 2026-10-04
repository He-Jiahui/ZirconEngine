#[test]
fn bloom_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../execute_bloom.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("bloom production source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}

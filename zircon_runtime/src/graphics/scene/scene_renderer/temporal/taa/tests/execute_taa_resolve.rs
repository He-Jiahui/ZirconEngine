#[test]
fn taa_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../execute_taa_resolve.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("TAA resolve production source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}

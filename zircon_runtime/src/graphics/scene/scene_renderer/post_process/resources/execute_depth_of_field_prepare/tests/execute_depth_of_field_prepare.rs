#[test]
fn depth_of_field_prepare_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../execute_depth_of_field_prepare.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("depth-of-field prepare production source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("return WgpuBufferUploadBatch::new()"));
}

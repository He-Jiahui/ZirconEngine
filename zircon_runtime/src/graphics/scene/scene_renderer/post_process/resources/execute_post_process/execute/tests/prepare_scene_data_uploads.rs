#[test]
fn scene_data_uploads_share_one_exact_payload_and_skip_empty_targets() {
    let source = include_str!("../prepare_scene_data_uploads.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("post-process scene-data upload source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("Vec::with_capacity(payload_byte_len)"));
    assert_eq!(production.matches("let payload: Arc<[u8]>").count(), 1);
    assert_eq!(production.matches("push_non_empty_upload(").count(), 4);
    assert!(production.contains("if payload_byte_len == 0"));
    assert!(production.contains("if source_range.is_empty()"));
}

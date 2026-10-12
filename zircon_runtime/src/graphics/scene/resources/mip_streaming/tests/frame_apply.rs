#[test]
fn mip_rebuild_upload_joins_the_frame_submission_transaction() {
    let source = include_str!("../frame_apply.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mip frame apply test boundary");

    assert!(source.contains("enqueue_gpu_texture_upload_work_for_frame("));
    assert!(source.contains("submission_transaction"));
    assert!(!source.contains(".enqueue_gpu_texture_upload_work(backend"));
}

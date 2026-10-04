#[test]
fn lut_publication_uses_one_snapshot_and_the_frame_texture_transaction() {
    let production = include_str!("../resource_streamer_ensure_post_process_lut_texture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("LUT streamer test boundary");
    let prepare = production
        .find("prepare_from_rgba8_asset(")
        .expect("LUT preparation");
    let enqueue = production
        .find("enqueue_copy_texture_upload_batch(upload_batch)")
        .expect("backend upload admission");
    let record = production
        .find("record_pre_scene_resource_submission(")
        .expect("frame transaction record");
    let publish = production
        .find("PreparedPostProcessLutTexture {")
        .expect("same-frame LUT publication");

    assert!(production.contains("load_texture_asset_snapshot(id)"));
    assert!(production.contains("let revision = texture.revision();"));
    assert!(!production.contains("(*texture).clone()"));
    assert!(production.contains("ensure_post_process_lut_texture_snapshot("));
    assert!(prepare < enqueue && enqueue < record && record < publish);
    assert!(!production.contains("wgpu::Queue"));
    assert!(!production.contains("queue.write_texture"));
}

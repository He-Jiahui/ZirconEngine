#[test]
fn post_process_scene_data_and_params_share_the_pass_upload_transaction() {
    let source = include_str!("../execute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("post-process execute source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("queue: &wgpu::Queue"));
    let scene_data = production
        .find("prepare_scene_data_uploads(")
        .expect("scene-data preparation");
    let params = production
        .find("post_process_params_upload(")
        .expect("parameter preparation");
    let append = production.find("uploads.append(").expect("batch append");
    assert!(scene_data < params && params < append);
}

use super::SsaoParams;

#[test]
fn ssao_params_are_prepared_into_the_frame_upload_transaction() {
    let source = include_str!("../profiled_scene_post_process_resources.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("profiled post-process resources source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("queue: &wgpu::Queue"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("frame_uploads.push("));
    assert!(production.contains("SsaoParams::from_compiled_profile("));
    assert!(!production.contains("tuning: ["));
}

#[test]
fn ssao_params_share_the_feature_owned_abi_layout() {
    assert_eq!(
        std::mem::size_of::<SsaoParams>(),
        std::mem::size_of::<([u32; 4], [u32; 4], [f32; 4], [f32; 4])>()
    );
}

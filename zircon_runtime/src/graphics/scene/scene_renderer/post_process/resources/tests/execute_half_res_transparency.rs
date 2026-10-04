#[test]
fn half_resolution_transparency_passes_keep_cached_pipelines() {
    let source = include_str!("../execute_half_res_transparency.rs");

    assert!(source.contains("half_res_transparency_depth_downsample_pipeline"));
    assert!(source.contains("half_res_transparency_composite_pipeline"));
    // BUG: [CR-SCENE-POST-0005] source 包含本测试的负向检查字面量，因此这里和后面的管线创建负向检查会命中自身并必败。
    assert!(!source.contains("queue.write_buffer("));
    assert!(source.contains("WgpuBufferUpload::from_bytes("));
    assert!(source.contains("WgpuBufferUploadBatch"));
    assert!(source.contains("binding: 4"));
    assert!(!source.contains("create_render_pipeline"));
}

use crate::graphics::scene::resources::default_pipeline_key;

#[test]
fn mirrored_pipeline_key_reverses_the_raster_front_face() {
    let mut key = default_pipeline_key();
    assert_eq!(super::mesh_front_face(&key), wgpu::FrontFace::Ccw);

    key.reverse_raster_winding = true;
    assert_eq!(super::mesh_front_face(&key), wgpu::FrontFace::Cw);
}

#[test]
fn every_mesh_raster_pass_consumes_the_shared_front_face_policy() {
    // TODO: [CR-R02-runtime_wave12_graphics_mesh_pipeline-0007] 确认子文件守卫是否应先排除各自测试文本；HitProxy 的 front_face needle 也出现在其测试断言中，尚无删除生产策略后的失败证据，需验证此循环能拒绝该变化。
    let pipeline_sources = [
        include_str!("../create_depth_prepass_mesh_pipeline.rs"),
        include_str!("../create_gbuffer_mesh_pipeline.rs"),
        include_str!("../create_hit_proxy_mesh_pipeline.rs"),
        include_str!("../create_mesh_pipeline.rs"),
        include_str!("../create_oit_mesh_pipeline.rs"),
        include_str!("../create_shadow_mesh_pipeline.rs"),
        include_str!("../create_taa_reactive_mask_mesh_pipeline.rs"),
        include_str!("../create_velocity_mesh_pipeline.rs"),
    ];

    for source in pipeline_sources {
        assert!(source.contains("front_face: super::mesh_front_face(key)"));
    }
}

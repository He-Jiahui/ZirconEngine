#[test]
fn depth_prepass_binds_forward_shadow_receiver_layout_slot() {
    let source = include_str!("mesh_recording.rs");

    assert!(source.contains("record_depth_prepass_to_resources"));
    assert!(source.contains("create_forward_shadow_receiver_bind_group"));
    assert!(source.contains("bind_forward_shadow_receiver_if_needed"));
}

#[test]
fn disabled_forward_volumetric_params_buffer_is_cache_owned() {
    let cache_source = include_str!("../../../mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs");
    let construct_source = include_str!("../../../mesh/mesh_pipeline_cache/construct.rs");
    let binding_source =
        include_str!("../../../mesh/mesh_pipeline_cache/forward_shadow_receiver.rs");
    let per_pass_buffer_creation = ["create_disabled_params_", "buffer(device"].concat();

    assert!(cache_source.contains("forward_volumetric_disabled_params_buffer: wgpu::Buffer"));
    assert!(construct_source.contains("let forward_volumetric_disabled_params_buffer ="));
    assert!(construct_source.contains("forward_volumetric_disabled_params_buffer,"));
    assert!(binding_source.contains("&self.forward_volumetric_disabled_params_buffer"));
    assert!(!binding_source.contains(&per_pass_buffer_creation));
}

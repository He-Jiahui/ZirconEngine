#[test]
fn depth_prepass_binds_forward_shadow_receiver_layout_slot() {
    let source = include_str!("mesh_recording.rs");

    assert!(source.contains("record_depth_prepass_to_resources"));
    assert!(source.contains("create_forward_shadow_receiver_bind_group"));
    assert!(source.contains("bind_forward_shadow_receiver_if_needed"));
}

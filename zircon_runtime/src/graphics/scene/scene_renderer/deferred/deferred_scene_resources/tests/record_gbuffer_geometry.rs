#[test]
fn deferred_gbuffer_binds_forward_shadow_receiver_layout_slot() {
    let source = include_str!("../record_gbuffer_geometry.rs");

    assert!(source.contains("create_forward_shadow_receiver_bind_group"));
    assert!(source.contains("bind_forward_shadow_receiver_if_needed"));
}

#[test]
fn forward_receiver_creation_profiles_standard_and_full_shapes_per_frame() {
    let source = include_str!("../forward_shadow_receiver.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("forward receiver production source");

    assert!(source.contains("fn begin_forward_receiver_binding_profile_frame(&mut self)"));
    assert!(source.contains("standard_forward_receiver_bind_group_create_count = 0;"));
    assert!(source.contains("full_forward_receiver_bind_group_create_count = 0;"));
    assert!(source.contains("\"standard_bind_group_create\""));
    assert!(source.contains("\"full_binding_prepare\""));
    assert!(source.contains("forward_receiver_standard_bind_group_create_count"));
    assert!(source.contains("forward_receiver_full_bind_group_create_count"));
    assert_eq!(source.matches(".saturating_add(1);").count(), 2);
}

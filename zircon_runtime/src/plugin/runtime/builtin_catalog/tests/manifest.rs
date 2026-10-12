use super::join_string_parts;

#[test]
fn exact_particles_identifier_join_preserves_feature_id() {
    assert_eq!(
        join_string_parts(&["particles.", "animation_control"]),
        "particles.animation_control"
    );
}

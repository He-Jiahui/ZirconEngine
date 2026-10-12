#[test]
fn registration_order_uses_constant_time_seen_membership() {
    let source = include_str!("../order.rs");
    let linear_membership = ["ordered_registration_indices", ".contains("].concat();
    assert!(!source.contains(&linear_membership));
}

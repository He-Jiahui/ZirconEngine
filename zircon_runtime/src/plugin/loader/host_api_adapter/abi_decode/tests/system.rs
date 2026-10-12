use super::system_access_id;

#[test]
fn exact_capacity_system_access_id_preserves_output() {
    assert_eq!(
        system_access_id("read", "component", "weather.velocity"),
        "read:component:weather.velocity"
    );
}

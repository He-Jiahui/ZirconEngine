use super::has_module_owner_prefix;

#[test]
fn borrowed_owner_prefix_preserves_boundary_semantics() {
    assert!(has_module_owner_prefix("weather.runtime", "weather"));
    assert!(!has_module_owner_prefix("weather2.runtime", "weather"));
    assert!(!has_module_owner_prefix("weather", "weather"));
    assert!(has_module_owner_prefix(".runtime", ""));
}

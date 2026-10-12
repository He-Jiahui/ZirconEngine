#[test]
fn module_system_owner_check_does_not_format_a_prefix() {
    let source = include_str!("../systems.rs");
    let formatted_prefix = ["format!(\"", "{package_id}.", "\")"].concat();
    assert!(!source.contains(&formatted_prefix));
}

#[test]
fn module_system_owner_check_preserves_the_dot_boundary() {
    assert!(super::runtime_plugin_package_system_name_has_owner(
        "physics",
        "physics.simulation"
    ));
    assert!(!super::runtime_plugin_package_system_name_has_owner(
        "phys",
        "physics.simulation"
    ));
}

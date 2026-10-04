#[test]
fn transient_projection_borrows_paths_and_reuses_properties() {
    let source = include_str!("../transient_ui_state.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(!implementation.contains("node.node_path.0.clone()"));
    assert!(implementation.contains("node.properties.get_mut(name)"));
}

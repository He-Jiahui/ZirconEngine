#[test]
fn window_registry_indexes_instances_without_cloning_rows() {
    let source = include_str!("../editor_window_registry.rs");
    let index = source
        .split("fn instances_by_id")
        .nth(1)
        .expect("instances_by_id body")
        .split("#[cfg(test)]")
        .next()
        .expect("instances_by_id implementation");
    assert!(!index.contains(".cloned()"));
    assert!(index.contains("&ViewInstance"));
}

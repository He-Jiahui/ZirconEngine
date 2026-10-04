#[test]
fn virtual_row_reconcile_uses_runtime_materialization_authority() {
    let source = include_str!("../virtual_rows.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("reconcile_virtual_list_materialization_with_keys"));
    assert!(implementation.contains("ensure_virtual_list_prototype_slots"));
    assert!(!implementation.contains("surface.tree.nodes.iter()"));
    assert!(!implementation.contains("surface.tree.nodes.values()"));
    assert!(!implementation.contains("fn inventory("));
    assert!(!implementation.contains("fn next_node_id("));
}

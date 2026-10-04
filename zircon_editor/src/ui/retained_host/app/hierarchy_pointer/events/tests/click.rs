#[test]
fn rename_reads_the_exact_runtime_row_after_sparse_name_patches() {
    let source = include_str!("../click.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(production.contains("self.runtime.scene_inspection_hierarchy_row(entity)"));
    assert!(!production.contains("get(*item_index).cloned()"));
}

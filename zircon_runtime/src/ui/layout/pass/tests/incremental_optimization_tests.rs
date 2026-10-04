#[test]
fn incremental_layout_root_workspaces_use_existing_upper_bounds() {
    let source = include_str!("../incremental.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("incremental layout implementation before tests");

    assert!(
        implementation.contains("let mut roots = Vec::with_capacity(candidates.len());"),
        "filtered layout roots should reserve the candidate-set upper bound"
    );
    assert!(
        implementation.contains("arrangement_roots.reserve(tree.roots.len());"),
        "root resize arrangement should reserve the appended root upper bound"
    );
}

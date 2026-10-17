//! 这些源码断言检查候选根与安排根容器的保留方式，不执行布局或测量实际分配次数。
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

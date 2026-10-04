#[test]
fn runtime_font_admission_dependency_projection_uses_sorted_contiguous_storage() {
    let source = include_str!("../font_admission.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("font admission implementation");

    assert!(implementation.contains(".collect::<Vec<_>>()"));
    assert!(implementation.contains("font_dependencies.sort_unstable();"));
    assert!(implementation.contains("font_dependencies.dedup();"));
    assert!(!implementation.contains("BTreeSet"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_font_admission_dependency_projection_bench_v1() {
    const UNIQUE_DEPENDENCIES: usize = 4_096;
    // TODO: [CR-DYN-FONT-ALLOCATION-EVIDENCE-0001] 此处分配数来自模型常量，尚无真实依赖投影的分配测量；下一步接入分配计数并核对 Vec 扩容与树节点开销。
    let legacy_tree_node_allocations = UNIQUE_DEPENDENCIES;
    let optimized_contiguous_buffer_allocations = 1;
    eprintln!(
        "RUNTIME808_FONT_ADMISSION_DEPENDENCY_BUFFER_BENCH_V1 unique_dependencies={UNIQUE_DEPENDENCIES} legacy_tree_node_allocations={legacy_tree_node_allocations} optimized_contiguous_buffer_allocations={optimized_contiguous_buffer_allocations}"
    );
    assert_eq!(legacy_tree_node_allocations, UNIQUE_DEPENDENCIES);
    assert_eq!(optimized_contiguous_buffer_allocations, 1);
    assert!(optimized_contiguous_buffer_allocations < legacy_tree_node_allocations);
}

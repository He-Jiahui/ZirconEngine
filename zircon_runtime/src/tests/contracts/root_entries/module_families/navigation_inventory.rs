//! 导航目录清单由单独的源库存拥有者核对，行为和文档边界留在 navigation.rs。
pub(super) fn assert_navigation_inventory(root: &std::path::Path) {
    for (owner, files) in [
        (
            "",
            &[
                "mod.rs",
                "module.rs",
                "operation",
                "repath_budget.rs",
                "runtime",
                "runtime.rs",
                "tests",
            ][..],
        ),
        (
            "runtime",
            &[
                "avoidance.rs",
                "baked_mesh",
                "baked_mesh.rs",
                "math.rs",
                "state",
                "state.rs",
                "tests",
                "world_scan",
                "world_scan.rs",
            ][..],
        ),
        (
            "runtime/baked_mesh",
            &["query_scratch.rs", "spatial_index.rs", "tests"][..],
        ),
        ("tests", &["module.rs"][..]),
        ("runtime/state", &["tests"][..]),
        ("runtime/world_scan", &["tests"][..]),
        ("runtime/baked_mesh/tests", &["spatial_index.rs"][..]),
        ("runtime/state/tests", &["repath_entry_tests.rs"][..]),
        ("runtime/world_scan/tests", &["capacity_tests.rs"][..]),
        (
            "operation",
            &[
                "handler.rs",
                "history_retention.rs",
                "mod.rs",
                "registration.rs",
                "tests",
            ][..],
        ),
        ("operation/tests", &["handler.rs"][..]),
    ] {
        let expected = files
            .iter()
            .map(|name| String::from(*name))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            entry_names(&root.join(owner)),
            expected,
            "navigation {owner}: update navigation/runtime.md and architecture/runtime-root-surface-m1.md for layout changes"
        );
    }
}

fn entry_names(path: &std::path::Path) -> std::collections::BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .map(|entry| {
            entry
                .expect("directory entry")
                .file_name()
                .into_string()
                .expect("UTF-8 entry")
        })
        .collect()
}

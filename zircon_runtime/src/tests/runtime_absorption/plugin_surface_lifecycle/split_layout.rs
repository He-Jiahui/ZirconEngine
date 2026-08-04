const SLICE: &str = "Runtime 15 M3 plugin surface lifecycle guard folder-backed split";
const STATUS: &str =
    "runtime_15_plugin_surface_lifecycle_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_plugin_surface_lifecycle_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_plugin_surface_lifecycle_guard_is_folder_backed";

const PARENT_PATH: &str = "plugin_surface_lifecycle.rs";
const CHILD_PATHS: &[&str] = &[
    "plugin_surface_lifecycle/inventory.rs",
    "plugin_surface_lifecycle/lifecycle_fallback.rs",
    "plugin_surface_lifecycle/mirror_docs.rs",
    "plugin_surface_lifecycle/native_loader_namespace.rs",
    "plugin_surface_lifecycle/split_layout.rs",
    "plugin_surface_lifecycle/support.rs",
];

#[test]
fn runtime_15_plugin_surface_lifecycle_guard_is_folder_backed() {
    let parent = include_str!("../plugin_surface_lifecycle.rs");
    let children = [
        include_str!("inventory.rs"),
        include_str!("lifecycle_fallback.rs"),
        include_str!("mirror_docs.rs"),
        include_str!("native_loader_namespace.rs"),
        include_str!("split_layout.rs"),
        include_str!("support.rs"),
    ];

    assert_contains_all(
        "plugin surface lifecycle parent routes child owners",
        parent,
        &[
            r#"#[path = "plugin_surface_lifecycle/inventory.rs"]"#,
            r#"#[path = "plugin_surface_lifecycle/lifecycle_fallback.rs"]"#,
            r#"#[path = "plugin_surface_lifecycle/mirror_docs.rs"]"#,
            r#"#[path = "plugin_surface_lifecycle/native_loader_namespace.rs"]"#,
            r#"#[path = "plugin_surface_lifecycle/split_layout.rs"]"#,
            r#"#[path = "plugin_surface_lifecycle/support.rs"]"#,
        ],
    );

    for moved_anchor in [
        "EXPECTED_RUNTIME_06_SOURCE_FILES",
        "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
        "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
        "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace",
        "files_containing",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "plugin surface lifecycle parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.iter().any(|source| source.contains(moved_anchor)),
            "plugin surface lifecycle children should own moved owner `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 30usize),
        (CHILD_PATHS[0], children[0], 70),
        (CHILD_PATHS[1], children[1], 60),
        (CHILD_PATHS[2], children[2], 210),
        (CHILD_PATHS[3], children[3], 90),
        (CHILD_PATHS[4], children[4], 190),
        (CHILD_PATHS[5], children[5], 150),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    for (label, source) in [
        (
            "Runtime 06 status",
            concat!(
                include_str!("../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"),
                include_str!("../../../../../docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md")
            ),
        ),
        (
            "Runtime 15 status",
            concat!(
                include_str!("../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
                include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md")
            ),
        ),
        (
            "runtime index status",
            concat!(
                include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
                include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md")
            ),
        ),
        (
            "engine code structure status",
            concat!(
                include_str!("../../../../../docs/plans/engine-code-structure-convention.md"),
                include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md")
            ),
        ),
        (
            "engine code review status",
            concat!(
                include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
                include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
            ),
        ),
        (
            "module convention doc",
            include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[4]]);
    }

    let frameworks = concat!(
        include_str!(
            "../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
        ),
        include_str!(
            "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
        )
    );
    assert_contains_all(
        "frameworks plan records plugin surface lifecycle split",
        frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .copied()
        .filter(|needle| !source.contains(needle))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing expected anchors:\n{}",
        missing.join("\n")
    );
}

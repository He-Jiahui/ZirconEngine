const SLICE: &str = "Runtime 15 M3 ECS kernel data guard folder-backed split";
const STATUS: &str = "runtime_15_ecs_kernel_data_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_ecs_kernel_data_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_ecs_kernel_data_guard_is_folder_backed";

const PARENT_PATH: &str = "ecs_kernel_data.rs";
const CHILD_PATHS: &[&str] = &[
    "ecs_kernel_data/component_storage.rs",
    "ecs_kernel_data/docs.rs",
    "ecs_kernel_data/guard_coverage.rs",
    "ecs_kernel_data/identity_storage.rs",
    "ecs_kernel_data/inventory.rs",
    "ecs_kernel_data/runtime_flow.rs",
    "ecs_kernel_data/split_layout.rs",
    "ecs_kernel_data/support.rs",
];

#[test]
fn runtime_15_ecs_kernel_data_guard_is_folder_backed() {
    let parent = include_str!("../ecs_kernel_data.rs");
    let children = [
        include_str!("component_storage.rs"),
        include_str!("docs.rs"),
        include_str!("guard_coverage.rs"),
        include_str!("identity_storage.rs"),
        include_str!("inventory.rs"),
        include_str!("runtime_flow.rs"),
        include_str!("split_layout.rs"),
        include_str!("support.rs"),
    ];

    assert_contains_all(
        "ECS kernel data parent routes child owners",
        parent,
        &[
            "mod component_storage;",
            "mod docs;",
            "mod guard_coverage;",
            "mod identity_storage;",
            "mod inventory;",
            "mod runtime_flow;",
            "mod split_layout;",
            "mod support;",
        ],
    );

    for moved_anchor in [
        "EXPECTED_RUNTIME_08_SOURCE_FILES",
        "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
        "assert_component_storage_private_reexport_cleanup",
        "assert_files_exist",
        "assert_runtime_08_mirror_docs",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "ECS kernel data parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.iter().any(|source| source.contains(moved_anchor)),
            "ECS kernel data children should own moved owner `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 40usize),
        (CHILD_PATHS[0], children[0], 120),
        (CHILD_PATHS[1], children[1], 90),
        (CHILD_PATHS[2], children[2], 120),
        (CHILD_PATHS[3], children[3], 240),
        (CHILD_PATHS[4], children[4], 160),
        (CHILD_PATHS[5], children[5], 140),
        (CHILD_PATHS[6], children[6], 190),
        (CHILD_PATHS[7], children[7], 40),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs"
    );
    assert_contains_all(
        "module-convention row data records ECS kernel data guard split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[6],
            GUARD,
        ],
    );

    let status_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs"
    );
    assert_contains_all("structure route status map", status_map, &[SLICE, STATUS]);

    let date_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs"
    );
    assert_contains_all("structure route date map", date_map, &[SLICE, "2026-07-05"]);

    for (label, source) in [
        (
            "Runtime 15 subplan",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "module convention doc",
            include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!("../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[6]]);
    }

    let frameworks = include_str!(
        "../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
    );
    assert_contains_all(
        "frameworks plan records ECS kernel data guard split",
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

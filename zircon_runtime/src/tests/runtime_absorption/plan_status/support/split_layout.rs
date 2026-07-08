use super::assert_contains_all;

const STATUS: &str =
    "runtime_15_plan_status_support_helpers_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_plan_status_support_helpers_folder_backed_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 plan-status support helpers folder-backed split";
const GUARD: &str = "runtime_15_plan_status_support_helpers_are_folder_backed";

const PARENT_PATH: &str = "plan_status/support.rs";
const CHILD_PATHS: &[&str] = &[
    "plan_status/support/assertions.rs",
    "plan_status/support/file_inventory.rs",
    "plan_status/support/frontmatter.rs",
    "plan_status/support/index_markdown.rs",
    "plan_status/support/runtime_plan_sources.rs",
    "plan_status/support/split_layout.rs",
];

#[test]
fn runtime_15_plan_status_support_helpers_are_folder_backed() {
    let parent = include_str!("../support.rs");
    let child_sources = [
        include_str!("assertions.rs"),
        include_str!("file_inventory.rs"),
        include_str!("frontmatter.rs"),
        include_str!("index_markdown.rs"),
        include_str!("runtime_plan_sources.rs"),
        include_str!("split_layout.rs"),
    ];

    assert_contains_all(
        "plan-status support parent mounts helper families",
        parent,
        &[
            "mod assertions;",
            "mod file_inventory;",
            "mod frontmatter;",
            "mod index_markdown;",
            "mod runtime_plan_sources;",
            "mod split_layout;",
            "pub(super) use assertions::assert_contains_all;",
            "runtime_absorption_plan_status_support_files",
            "runtime_subplan_sources",
        ],
    );

    for moved_helper in [
        "fn frontmatter_value",
        "fn runtime_plan_dir",
        "fn runtime_index_problem_row_for",
        "fn collect_rust_files_relative_to",
        "fn markdown_table_cells",
    ] {
        assert!(
            !parent.contains(moved_helper),
            "plan-status support parent should not retain moved helper `{moved_helper}`"
        );
        assert!(
            child_sources
                .iter()
                .any(|source| source.contains(moved_helper)),
            "plan-status support children should retain moved helper `{moved_helper}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 40usize),
        (CHILD_PATHS[0], child_sources[0], 20),
        (CHILD_PATHS[1], child_sources[1], 80),
        (CHILD_PATHS[2], child_sources[2], 50),
        (CHILD_PATHS[3], child_sources[3], 90),
        (CHILD_PATHS[4], child_sources[4], 80),
        (CHILD_PATHS[5], child_sources[5], 190),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data_parent = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs"
    );
    assert_contains_all(
        "runtime index anchor row data parent exports support helper split",
        row_data_parent,
        &["support_inventory::SUPPORT_HELPERS_FOLDER_BACKED_SPLIT"],
    );
    let row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/support_inventory.rs"
    );
    assert_contains_all(
        "support inventory row data records support helper split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[5],
            GUARD,
        ],
    );

    let status_map = include_str!(
        "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
    );
    assert_contains_all(
        "runtime index anchor status map",
        status_map,
        &[SLICE, STATUS],
    );

    let date_map = include_str!(
        "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
    );
    assert_contains_all(
        "runtime index anchor date map",
        date_map,
        &[SLICE, "2026-07-05"],
    );

    for (label, source) in [
        (
            "Runtime 15 subplan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!(
                "../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
            ),
        ),
        (
            "module convention doc",
            include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!(
                "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[5]]);
    }

    let frameworks = include_str!(
        "../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
    );
    assert_contains_all(
        "frameworks plan records support helper split",
        frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}

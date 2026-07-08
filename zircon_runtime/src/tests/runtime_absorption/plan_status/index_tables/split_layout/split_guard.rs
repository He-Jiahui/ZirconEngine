use super::{
    assert_contains_all, CHILD_PATHS, PARENT_PATH, SPLIT_LAYOUT_CHILD_PATHS,
    SPLIT_LAYOUT_FRAMEWORKS_STATUS, SPLIT_LAYOUT_GUARD, SPLIT_LAYOUT_SLICE, SPLIT_LAYOUT_STATUS,
};

#[test]
fn runtime_15_plan_status_index_tables_split_layout_is_folder_backed() {
    let split_layout = include_str!("../split_layout.rs");
    let child_owner = include_str!("child_owner.rs");
    let parent_guard = include_str!("parent_guard.rs");
    let split_guard = include_str!("split_guard.rs");
    let children = format!("{child_owner}\n{parent_guard}\n{split_guard}");

    assert_contains_all(
        "plan-status index table split-layout route mounts child guards",
        split_layout,
        &["mod child_owner;", "mod parent_guard;", "mod split_guard;"],
    );
    for moved_guard in [
        "runtime_15_plan_status_index_tables_guard_child_owner_split",
        "runtime_15_plan_status_index_tables_parent_guard_is_folder_backed",
    ] {
        assert!(
            !split_layout.contains(&format!("fn {moved_guard}")),
            "plan-status index table split-layout route should not retain `{moved_guard}`"
        );
        assert!(
            children.contains(&format!("fn {moved_guard}")),
            "plan-status index table split-layout children should retain `{moved_guard}`"
        );
    }

    for (path, source, max_lines) in [
        (CHILD_PATHS[1], split_layout, 40usize),
        (SPLIT_LAYOUT_CHILD_PATHS[0], child_owner, 240),
        (SPLIT_LAYOUT_CHILD_PATHS[1], parent_guard, 170),
        (SPLIT_LAYOUT_CHILD_PATHS[2], split_guard, 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data_parent = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs"
    );
    assert_contains_all(
        "runtime index anchor row data parent exports split-layout row",
        row_data_parent,
        &["plan_status_children::INDEX_TABLES_SPLIT_LAYOUT_FOLDER_BACKED_SPLIT"],
    );
    let row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/plan_status_children.rs"
    );
    assert_contains_all(
        "plan-status child row data records split-layout folder-backed split",
        row_data,
        &[
            SPLIT_LAYOUT_SLICE,
            SPLIT_LAYOUT_STATUS,
            PARENT_PATH,
            CHILD_PATHS[1],
            SPLIT_LAYOUT_CHILD_PATHS[0],
            SPLIT_LAYOUT_CHILD_PATHS[1],
            SPLIT_LAYOUT_CHILD_PATHS[2],
            SPLIT_LAYOUT_GUARD,
        ],
    );

    let status_map = [
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
        ),
    ]
    .join("\n");
    assert_contains_all(
        "runtime index anchor status map records split-layout split",
        status_map.as_str(),
        &[SPLIT_LAYOUT_SLICE, SPLIT_LAYOUT_STATUS],
    );
    let date_map = [
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
        ),
    ]
    .join("\n");
    assert_contains_all(
        "runtime index anchor date map records split-layout split",
        date_map.as_str(),
        &[SPLIT_LAYOUT_SLICE, "2026-07-06"],
    );

    for (label, source) in [
        (
            "Runtime 15 subplan",
            include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "frameworks plan",
            include_str!(
                "../../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
            ),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!(
                "../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
            ),
        ),
        (
            "module convention doc",
            include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!(
                "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SPLIT_LAYOUT_SLICE,
                SPLIT_LAYOUT_STATUS,
                SPLIT_LAYOUT_GUARD,
                SPLIT_LAYOUT_FRAMEWORKS_STATUS,
                SPLIT_LAYOUT_CHILD_PATHS[0],
                SPLIT_LAYOUT_CHILD_PATHS[1],
                SPLIT_LAYOUT_CHILD_PATHS[2],
            ],
        );
    }
}

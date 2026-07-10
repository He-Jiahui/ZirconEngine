use super::super::super::support::runtime_numbered_archive_sources;
use super::{assert_contains_all, CHILD_PATHS, GUARD, PARENT_PATH, SLICE, STATUS};

#[test]
fn runtime_15_plan_status_index_tables_parent_guard_is_folder_backed() {
    let parent = include_str!("../../index_tables.rs");
    let split_layout = include_str!("../split_layout.rs");
    let child_owner = include_str!("child_owner.rs");

    assert_contains_all(
        "plan-status index table parent delegates split guard",
        parent,
        &[
            "mod index_consistency;",
            "mod split_layout;",
            "mod status_anchors;",
            "mod subplan_map;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_plan_status_index_tables_guard_child_owner_split"),
        "plan-status index table parent should not retain split guard body"
    );
    assert!(
        child_owner.contains("fn runtime_15_plan_status_index_tables_guard_child_owner_split"),
        "plan-status index table split child should own the existing split guard body"
    );

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 20usize),
        (CHILD_PATHS[0], include_str!("../index_consistency.rs"), 210),
        (CHILD_PATHS[1], split_layout, 40),
        (
            "plan_status/index_tables/split_layout/child_owner.rs",
            child_owner,
            240,
        ),
        (
            "plan_status/index_tables/split_layout/parent_guard.rs",
            include_str!("parent_guard.rs"),
            170,
        ),
        (
            "plan_status/index_tables/split_layout/split_guard.rs",
            include_str!("split_guard.rs"),
            180,
        ),
        (CHILD_PATHS[2], include_str!("../status_anchors.rs"), 20),
        (CHILD_PATHS[3], include_str!("../subplan_map.rs"), 230),
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
        "runtime index anchor row data parent exports index-table split guard row",
        row_data_parent,
        &["plan_status_children::INDEX_TABLES_PARENT_GUARD_FOLDER_BACKED_SPLIT"],
    );
    let row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/plan_status_children.rs"
    );
    assert_contains_all(
        "plan-status child row data records index-table parent guard split",
        row_data,
        &[SLICE, STATUS, PARENT_PATH, CHILD_PATHS[1], GUARD],
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
        "runtime index anchor status map",
        status_map.as_str(),
        &[SLICE, STATUS],
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
        "runtime index anchor date map",
        date_map.as_str(),
        &[SLICE, "2026-07-05"],
    );

    let archive_source = runtime_numbered_archive_sources();
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &[SLICE, STATUS, GUARD, CHILD_PATHS[1]],
    );
}

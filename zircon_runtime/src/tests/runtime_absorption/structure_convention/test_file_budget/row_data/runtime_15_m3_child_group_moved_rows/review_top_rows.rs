use super::*;

#[test]
fn runtime_15_m3_child_group_moved_review_top_rows_are_child_owned() {
    let child_groups_guard = read_runtime_src(CHILD_GROUPS_GUARD_PATH);
    let moved_row_guard_children = moved_row_guard_child_source_blob();
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_ROWS_PATH);
    let review_status_sync = [
        read_runtime_src(REVIEW_STATUS_SYNC_ROWS_PATH),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync/importer_fixture_rows.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync/p0_core_rows.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync/provider_lookup_rows.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync/typed_runtime_rows.rs",
        ),
    ]
    .join("\n");

    for moved_row_source in [
        "Runtime 15 M3 config store lock poison recovery",
        "Runtime 15 M3 module convention non-render debt guard",
        "Runtime 15 M3 D13 importer top-row closed status sync",
    ] {
        assert!(
            !child_groups_guard.contains(moved_row_source),
            "runtime_15_m3_child_groups.rs should delegate moved-row ownership checks for {moved_row_source}"
        );
        assert!(
            moved_row_guard_children.contains(moved_row_source),
            "runtime_15_m3_child_group_moved_rows child modules should own moved-row ownership check {moved_row_source}"
        );
    }

    for moved_top_row in [
        "Runtime 15 M3 D13 importer top-row closed status sync",
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync",
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync",
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync",
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync",
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync",
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync",
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync",
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
    ] {
        assert!(
            !foundation_guards.contains(moved_top_row),
            "foundation_guards.rs should delegate review top-row status rows to review_status_sync.rs instead of keeping {moved_top_row}"
        );
        assert!(
            review_status_sync.contains(moved_top_row),
            "review_status_sync child tree should own moved review top-row status row {moved_top_row}"
        );
    }
}

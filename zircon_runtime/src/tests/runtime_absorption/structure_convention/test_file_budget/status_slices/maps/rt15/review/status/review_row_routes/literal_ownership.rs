use super::*;

#[test]
fn runtime_15_status_support_review_guard_row_data_route_literals_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD);
    let status_paths = status_support_review_guard_row_data_child_paths();
    let date_paths = date_support_review_guard_row_data_child_paths();
    let status_children = read_sources(&status_paths).join("\n");
    let date_children = read_sources(&date_paths).join("\n");

    for moved_literal in [
        "Runtime 15 M3 status output review-guard row-data guard child-owner split",
        "Runtime 15 M3 review-guard moved-row guard folder-backed split",
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split",
        "Runtime 15 M3 plugin-importer status-output guard folder-backed split",
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split",
        "Runtime 15 M3 review-guard direct-assertion export-chain guard child split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status review_guard_row_data_maps.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date review_guard_row_data_maps.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "status-support review-guard row-data children own moved routes",
        &format!("{status_children}\n{date_children}"),
        &[
            "Runtime 15 M3 status-support review-guard row-data expected-slice maps folder-backed split",
            "runtime_15_status_support_review_guard_row_data_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
            "Runtime 15 M3 status output review-guard row-data guard child-owner split",
            "Runtime 15 M3 review-guard moved-row code-review rows child split",
            "Runtime 15 M3 review-guard code-review status-mirror child split",
            "Runtime 15 M3 review-guard row-data aggregation guard child split",
            "Runtime 15 M3 review-guard row-data status-doc root inventory child split",
            "Runtime 15 M3 review-guard direct-assertion export-chain guard child split",
        ],
    );
}

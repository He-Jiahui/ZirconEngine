use super::*;

#[test]
fn runtime_15_status_support_row_data_route_literals_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_SUPPORT_ROW_DATA_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_ROW_DATA_CHILD);
    let status_paths = status_support_row_data_child_paths();
    let date_paths = date_support_row_data_child_paths();
    let status_children = read_sources(&status_paths).join("\n");
    let date_children = read_sources(&date_paths).join("\n");

    for moved_literal in [
        "Runtime 15 M3 status output M3 row data child-owner split",
        "Runtime 15 M3 status-output row-data module-layout guard folder-backed split",
        "Runtime 15 M3 status output review-guard row-data guard child-owner split",
        "Runtime 15 M3 foundation row-data guard child-owner split",
        "Runtime 15 M3 child-groups status-doc guard child-owner split",
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status row_data_maps.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date row_data_maps.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "status-support row-data expected-slice children own moved routes",
        &format!("{status_children}\n{date_children}"),
        &[
            "Runtime 15 M3 status-support row-data route expected-slice guard folder-backed split",
            "runtime_15_status_support_row_data_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
            "Runtime 15 M3 status-support row-data expected-slice maps folder-backed split",
            "runtime_15_status_support_row_data_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
            "Runtime 15 M3 status output M3 row data child-owner split",
            "Runtime 15 M3 status-output row-data module-layout guard folder-backed split",
            "Runtime 15 M3 status output review-guard row-data guard child-owner split",
            "Runtime 15 M3 foundation row-data guard child-owner split",
            "Runtime 15 M3 child-groups status-doc guard child-owner split",
            "Runtime 15 M3 lock-poison status row-data guard folder-backed split",
            "runtime_15_scene_script_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ],
    );
}

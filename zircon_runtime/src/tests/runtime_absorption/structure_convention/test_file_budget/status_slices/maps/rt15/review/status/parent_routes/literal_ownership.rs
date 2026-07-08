use super::*;

#[test]
fn runtime_15_status_support_parent_route_literals_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_SUPPORT_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_CHILD);
    let status_children = read_sources(STATUS_SUPPORT_PARENT_ROUTE_CHILDREN);
    let date_children = read_sources(DATE_SUPPORT_PARENT_ROUTE_CHILDREN);
    let child_blob = format!("{status_children}\n{date_children}");

    for moved_literal in [
        "Runtime 15 M3 test file budget root-layout child split",
        "Runtime 15 M3 foundation row-data topic child-owner split",
        "Runtime 15 M3 M2 row-data guard child-owner split",
        "Runtime 15 M3 support Hub project-actions tests child-owner split",
        "Runtime 15 M3 render shader template assembly guard support child-owner split",
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
        "Runtime 15 M3 UI asset MUI web form style test folder split",
        "Runtime 15 M3 evidence anchors root inventory child split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status status_support_maps.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date status_support_maps.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "status-support expected-slice parent route children own moved routes",
        &child_blob,
        &[
            "Runtime 15 M3 status-support parent-route expected-slice guard folder-backed split",
            "runtime_15_status_support_parent_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
            "Runtime 15 M3 status-support expected-slice parent maps folder-backed split",
            "runtime_15_status_support_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
            "Runtime 15 M3 test file budget root-layout child split",
            "Runtime 15 M3 foundation row-data topic child-owner split",
            "Runtime 15 M3 M2 row-data guard child-owner split",
            "Runtime 15 M3 support Hub project-actions tests child-owner split",
            "Runtime 15 M3 render shader template assembly guard support child-owner split",
            "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
            "Runtime 15 M3 UI asset MUI web form style test folder split",
            "Runtime 15 M3 evidence anchors root inventory child split",
        ],
    );
}

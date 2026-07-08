use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_status_support_literals_are_child_owned() {
    let status_support_child = read_runtime_src(STATUS_SUPPORT_CHILD);
    let status_support_row_data_child = read_runtime_src(STATUS_SUPPORT_ROW_DATA_CHILD);
    let status_support_plan_doc_child = read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD);
    let status_support_plan_doc_route_children =
        read_sources(STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN);
    let date_support_child = read_runtime_src(DATE_SUPPORT_CHILD);
    let date_support_row_data_child = read_runtime_src(DATE_SUPPORT_ROW_DATA_CHILD);
    let date_support_plan_doc_child = read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD);
    let date_support_plan_doc_route_children = read_sources(DATE_SUPPORT_PLAN_DOC_ROUTE_CHILDREN);

    assert_contains_all(
        "status-support expected-slice children own status-support literals",
        &format!(
            "{status_support_child}\n{status_support_row_data_child}\n{status_support_plan_doc_child}\n{status_support_plan_doc_route_children}\n{date_support_child}\n{date_support_row_data_child}\n{date_support_plan_doc_child}\n{date_support_plan_doc_route_children}"
        ),
        &[
            "Runtime 15 M3 status output expected-slice guard child-owner split",
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 status-support expected-slice map child split",
            "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
            "Runtime 15 M3 structure-support expected-slice map child-owner split",
            "runtime_15_structure_support_expected_slice_map_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-30\")",
            "Some(\"2026-07-05\")",
        ],
    );
}

use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_literals_are_child_owned() {
    let status_support_child = read_runtime_src(STATUS_SUPPORT_CHILD);
    let date_support_child = read_runtime_src(DATE_SUPPORT_CHILD);
    let status_support_row_data_child = read_runtime_src(STATUS_SUPPORT_ROW_DATA_CHILD);
    let status_support_row_data_route_children =
        read_runtime_sources(STATUS_SUPPORT_ROW_DATA_ROUTE_CHILDREN);
    let status_support_plan_doc_child = read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD);
    let status_support_plan_doc_route_children =
        read_runtime_sources(STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN);
    let status_support_priority_plan_doc_route_children =
        read_runtime_sources(STATUS_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN);
    let date_support_row_data_child = read_runtime_src(DATE_SUPPORT_ROW_DATA_CHILD);
    let date_support_row_data_route_children =
        read_runtime_sources(DATE_SUPPORT_ROW_DATA_ROUTE_CHILDREN);
    let date_support_plan_doc_child = read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD);
    let date_support_plan_doc_route_children =
        read_runtime_sources(DATE_SUPPORT_PLAN_DOC_ROUTE_CHILDREN);
    let date_support_priority_plan_doc_route_children =
        read_runtime_sources(DATE_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN);

    for moved_literal in [
        "Runtime 15 M3 status output M3 row data child-owner split",
        "Runtime 15 M3 status output expected-slice legacy child-owner split",
        "Runtime 15 M3 priority plan docs row-data owner child split",
        "Runtime 15 M3 status-support row-data root inventory child split",
        "Runtime 15 M3 asset-budget row-data root inventory child split",
    ] {
        assert!(
            !status_support_child.contains(moved_literal),
            "status_support_maps.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_support_child.contains(moved_literal),
            "date status_support_maps.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "status-support expected-slice child maps own moved literals",
        &format!(
            "{status_support_row_data_child}\n{status_support_row_data_route_children}\n{status_support_plan_doc_child}\n{status_support_plan_doc_route_children}\n{status_support_priority_plan_doc_route_children}\n{date_support_row_data_child}\n{date_support_row_data_route_children}\n{date_support_plan_doc_child}\n{date_support_plan_doc_route_children}\n{date_support_priority_plan_doc_route_children}"
        ),
        &[
            "Runtime 15 M3 status-support expected-slice map child split",
            "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
            "Runtime 15 M3 status output M3 row data child-owner split",
            "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 status output expected-slice legacy child-owner split",
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 priority plan docs root inventory child split",
            "runtime_15_priority_plan_docs_root_inventory_child_split_static_passed_cargo_deferred",
            "Runtime 15 M3 asset-budget row-data root inventory child split",
            "runtime_15_asset_budget_row_data_root_inventory_child_split_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
        ],
    );
}

use super::*;

#[test]
fn runtime_15_status_support_expected_slice_maps_are_child_owned() {
    let status_support_child = read_runtime_src(STATUS_SUPPORT_CHILD);
    let date_support_child = read_runtime_src(DATE_SUPPORT_CHILD);

    assert_contains_all(
        "status-support expected-slice parents mount child maps",
        &format!("{status_support_child}\n{date_support_child}"),
        &[
            "#[path = \"status_support_maps/row_data_maps.rs\"]",
            "#[path = \"status_support_maps/plan_doc_support_maps.rs\"]",
            "row_data_maps::expected_status_for_slice(slice)",
            "plan_doc_support_maps::expected_status_for_slice(slice)",
            "row_data_maps::expected_date_for_slice(slice)",
            "plan_doc_support_maps::expected_date_for_slice(slice)",
        ],
    );
}

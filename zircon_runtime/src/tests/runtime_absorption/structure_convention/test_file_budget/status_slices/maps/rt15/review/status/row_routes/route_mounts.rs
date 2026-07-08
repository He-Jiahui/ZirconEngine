use super::*;

#[test]
fn runtime_15_status_support_row_data_expected_slice_maps_are_folder_backed() {
    let guard_parent = include_str!("../row_data_route_children.rs");
    let status_parent = read_runtime_src(STATUS_SUPPORT_ROW_DATA_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_ROW_DATA_CHILD);

    assert_contains_all(
        "status-support row-data route guard mounts folder-backed children",
        guard_parent,
        &[
            "#[path = \"row_routes/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"row_routes/child_paths.rs\"]",
            "mod child_paths;",
            "#[path = \"row_routes/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"row_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"row_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"row_routes/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"row_routes/source_reads.rs\"]",
            "mod source_reads;",
            "#[path = \"row_routes/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    assert_contains_all(
        "status-support row-data expected-slice parents mount route children",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"row_data_maps/root_runtime_maps.rs\"]",
            "mod root_runtime_maps;",
            "#[path = \"row_data_maps/module_layout_maps.rs\"]",
            "mod module_layout_maps;",
            "#[path = \"row_data_maps/review_guard_row_data_maps.rs\"]",
            "mod review_guard_row_data_maps;",
            "#[path = \"row_data_maps/foundation_row_data_maps.rs\"]",
            "mod foundation_row_data_maps;",
            "#[path = \"row_data_maps/child_group_row_data_maps.rs\"]",
            "mod child_group_row_data_maps;",
            "#[path = \"row_data_maps/other_row_data_maps.rs\"]",
            "mod other_row_data_maps;",
            "root_runtime_maps::expected_status_for_slice(slice)",
            "module_layout_maps::expected_status_for_slice(slice)",
            "review_guard_row_data_maps::expected_status_for_slice(slice)",
            "foundation_row_data_maps::expected_status_for_slice(slice)",
            "child_group_row_data_maps::expected_status_for_slice(slice)",
            "other_row_data_maps::expected_status_for_slice(slice)",
            "root_runtime_maps::expected_date_for_slice(slice)",
            "module_layout_maps::expected_date_for_slice(slice)",
            "review_guard_row_data_maps::expected_date_for_slice(slice)",
            "foundation_row_data_maps::expected_date_for_slice(slice)",
            "child_group_row_data_maps::expected_date_for_slice(slice)",
            "other_row_data_maps::expected_date_for_slice(slice)",
        ],
    );
}

use super::*;

#[test]
fn runtime_15_status_support_review_guard_row_data_expected_slice_maps_are_folder_backed() {
    let guard_parent = include_str!("../review_guard_row_data_route_children.rs");
    let status_parent = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD);

    assert_contains_all(
        "status-support review-guard row-data route guard mounts folder-backed children",
        guard_parent,
        &[
            "#[path = \"review_row_routes/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"review_row_routes/child_paths.rs\"]",
            "mod child_paths;",
            "#[path = \"review_row_routes/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"review_row_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"review_row_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"review_row_routes/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"review_row_routes/source_reads.rs\"]",
            "mod source_reads;",
            "#[path = \"review_row_routes/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    assert_contains_all(
        "status-support review-guard row-data parents mount topic children",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"review_guard_row_data_maps/base_child_owner_maps.rs\"]",
            "mod base_child_owner_maps;",
            "#[path = \"review_guard_row_data_maps/moved_row_maps.rs\"]",
            "mod moved_row_maps;",
            "#[path = \"review_guard_row_data_maps/code_review_maps.rs\"]",
            "mod code_review_maps;",
            "#[path = \"review_guard_row_data_maps/row_data_guard_maps.rs\"]",
            "mod row_data_guard_maps;",
            "#[path = \"review_guard_row_data_maps/status_doc_maps.rs\"]",
            "mod status_doc_maps;",
            "#[path = \"review_guard_row_data_maps/direct_assertion_maps.rs\"]",
            "mod direct_assertion_maps;",
            "base_child_owner_maps::expected_status_for_slice(slice)",
            "direct_assertion_maps::expected_status_for_slice(slice)",
            "base_child_owner_maps::expected_date_for_slice(slice)",
            "direct_assertion_maps::expected_date_for_slice(slice)",
        ],
    );
}

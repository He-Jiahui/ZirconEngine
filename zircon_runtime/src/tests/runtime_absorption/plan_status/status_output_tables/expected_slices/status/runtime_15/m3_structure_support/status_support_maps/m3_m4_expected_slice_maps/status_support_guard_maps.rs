pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support top-level support row data folder-backed split" => Some(
            "runtime_15_status_support_top_level_support_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support route metadata row data folder-backed split" => Some(
            "runtime_15_status_support_route_metadata_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support status support maps row data folder-backed split" => Some(
            "runtime_15_status_support_status_support_maps_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support route guard rows row-data owner child split" => Some(
            "runtime_15_status_support_route_guard_rows_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard structure row data folder-backed split" => Some(
            "runtime_15_review_guard_structure_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support M3/M4 expected-slice maps folder-backed split" => Some(
            "runtime_15_status_support_m3_m4_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice parent maps folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice owner paths folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_owner_paths_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice owner paths guard folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_owner_paths_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice guard route metadata child split" => Some(
            "runtime_15_status_support_expected_slice_guard_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice route metadata folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice route metadata status mirrors folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_route_metadata_status_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice guard body folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support expected-slice guard body status mirrors folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_guard_body_status_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support parent-route expected-slice guard folder-backed split" => Some(
            "runtime_15_status_support_parent_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support row-data route expected-slice guard folder-backed split" => Some(
            "runtime_15_status_support_row_data_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support plan-doc route expected-slice guard folder-backed split" => Some(
            "runtime_15_status_support_plan_doc_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support review-guard row-data route expected-slice guard folder-backed split" => Some(
            "runtime_15_status_support_review_guard_row_data_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support runtime-index anchor row-data child split" => Some(
            "runtime_15_status_support_runtime_index_anchor_row_data_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

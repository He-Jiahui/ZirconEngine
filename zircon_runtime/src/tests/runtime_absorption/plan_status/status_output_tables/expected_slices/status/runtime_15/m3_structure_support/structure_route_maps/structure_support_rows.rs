pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 structure-support expected-slice guard folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice status mirrors folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_status_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice literal ownership folder-backed split" => {
            Some("runtime_15_structure_support_expected_slice_literal_ownership_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 structure-support expected-slice literal ownership status mirrors folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_literal_ownership_status_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice budgets folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_budgets_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice parent maps folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice parent-route metadata child split" => Some(
            "runtime_15_structure_support_expected_slice_parent_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice parent-route metadata folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_parent_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice parent-route guard body folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_parent_route_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice parent-route guard body route metadata folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

// Guard: runtime_15_structure_support_expected_slice_parent_route_guard_body_is_child_owned.
// Guard: runtime_15_structure_support_expected_slice_parent_route_metadata_is_folder_backed.
// Guard: runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_is_folder_backed.
// Guard: runtime_15_structure_support_expected_slice_guard_is_folder_backed.
// Guard: runtime_15_structure_support_expected_slice_status_mirrors_are_folder_backed.
// Guard: runtime_15_structure_support_expected_slice_literal_ownership_is_folder_backed.

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output expected-slice maps split" => Some(
            "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard body folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard body budgets folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_budgets_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard-body route mounts folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard body folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard-body route mounts folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_route_mounts_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard route metadata child split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard route metadata folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard sources folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget route metadata child split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget route metadata folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget source inventory folder-backed split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_source_inventory_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice guard maps child-owner split" => Some(
            "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-output expected-slice guard maps folder-backed split" => Some(
            "runtime_15_status_output_expected_slice_guard_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation expected-slice maps guard folder-backed split" => Some(
            "runtime_15_foundation_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary render-graphics map rows guard folder-backed split" => Some(
            "runtime_15_naming_boundary_render_graphics_map_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation expected-slice maps status mirrors folder-backed split" => Some(
            "runtime_15_foundation_expected_slice_maps_status_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 naming-boundary expected-slice sources folder-backed split" => Some(
            "runtime_15_naming_boundary_expected_slice_sources_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split" => Some(
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget child-source current-route sync" => Some(
            "runtime_15_runtime_07_owner_budget_child_source_current_route_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split" => Some(
            "runtime_15_runtime_07_owner_budget_virtual_geometry_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split" => Some(
            "runtime_15_runtime_07_owner_budget_large_file_gate_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split" => Some(
            "runtime_15_runtime_07_owner_budget_mirror_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

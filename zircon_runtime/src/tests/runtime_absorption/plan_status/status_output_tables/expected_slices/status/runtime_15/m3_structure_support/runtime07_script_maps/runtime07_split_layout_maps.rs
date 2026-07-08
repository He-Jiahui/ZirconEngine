pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters split-layout guard folder-backed split" => Some(
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split" => Some(
            "runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 submit-context split-layout guard folder-backed split" => Some(
            "runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split" => Some(
            "runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split" => Some(
            "runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

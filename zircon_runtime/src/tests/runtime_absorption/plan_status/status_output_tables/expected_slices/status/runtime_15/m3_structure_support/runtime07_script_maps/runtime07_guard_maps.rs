pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" => Some(
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
        ),
        "Runtime 15 M3 Runtime 07 submit-context guard child-owner split" => Some(
            "runtime_15_runtime_07_submit_context_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split" => Some(
            "runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split" => Some(
            "runtime_15_runtime_07_owner_budget_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split" => Some(
            "runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 scene/project guard child-owner split" => Some(
            "runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters child-owner split" => Some(
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

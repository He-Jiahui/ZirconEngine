pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 root-surface route-owner split"
        | "Runtime 15 M3 job-system route-owner split"
        | "Runtime 15 M3 script-binding route-owner split"
        | "Runtime 15 M3 asset-pipeline route-owner split"
        | "Runtime 15 M3 asset-surface route-owner split"
        | "Runtime 15 M3 asset-worker-policy route-owner split"
        | "Runtime 15 M3 builtin-modules route-owner split"
        | "Runtime 15 M3 rayon-boundary route-owner split"
        | "Runtime 15 M3 schedule-frame-loop route-owner split"
        | "Runtime 15 M3 tech-stack route-owner split"
        | "Runtime 15 M3 script-absorption route-owner split"
        | "Runtime 15 M3 resource-foundation route-owner split"
        | "Runtime 15 M3 compatibility-shells route-owner split" => Some("2026-07-05"),
        "Runtime 15 M3 ui-architecture route-owner split"
        | "Runtime 15 M3 dynamic-scene route-owner split" => Some("2026-07-06"),
        _ => None,
    }
}

// Guard: runtime_15_root_surface_route_owner_is_folder_backed.
// Guard: runtime_15_job_system_route_owner_is_folder_backed.
// Guard: runtime_15_script_binding_route_owner_is_folder_backed.
// Guard: runtime_15_asset_pipeline_route_owner_is_folder_backed.
// Guard: runtime_15_asset_surface_route_owner_is_folder_backed.
// Guard: runtime_15_asset_worker_policy_route_owner_is_folder_backed.
// Guard: runtime_15_schedule_frame_loop_route_owner_is_folder_backed.
// Guard: runtime_15_tech_stack_route_owner_is_folder_backed.
// Guard: runtime_15_script_absorption_route_owner_is_folder_backed.
// Guard: runtime_15_resource_foundation_route_owner_is_folder_backed.
// Guard: runtime_15_compatibility_shells_route_owner_is_folder_backed.
// Guard: runtime_15_ui_architecture_route_owner_is_folder_backed.
// Guard: runtime_15_dynamic_scene_route_owner_is_folder_backed.

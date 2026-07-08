pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 generated-code guard folder-backed split"
        | "Runtime 15 M3 input-stack absorption guard folder-backed split"
        | "Runtime 15 M3 ECS kernel data guard folder-backed split"
        | "Runtime 15 M3 script host ledger guard folder-backed split"
        | "Runtime 15 M3 dynamic API session shared data folder-backed split"
        | "Runtime 15 M3 plugin surface lifecycle guard folder-backed split" => Some("2026-07-05"),
        "Runtime 15 M3 input-stack inventory guard folder-backed split" => Some("2026-07-06"),
        _ => None,
    }
}

// Guard: runtime_15_generated_code_guard_is_folder_backed.
// Guard: runtime_15_input_stack_absorption_guard_is_folder_backed.
// Guard: runtime_15_input_stack_inventory_guard_is_folder_backed.
// Guard: runtime_15_ecs_kernel_data_guard_is_folder_backed.
// Guard: runtime_15_script_host_ledger_guard_is_folder_backed.
// Guard: runtime_15_dynamic_api_session_shared_data_is_folder_backed.
// Guard: runtime_15_plugin_surface_lifecycle_guard_is_folder_backed.

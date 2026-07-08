pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split" => Some(
            "runtime_15_lock_poison_status_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 lock-poison status row-data status-mirror child split" => Some(
            "runtime_15_lock_poison_status_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 lock-poison status row-data root inventory child split" => Some(
            "runtime_15_lock_poison_status_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 lock-poison status row-data source/status-map sync" => Some(
            "runtime_15_lock_poison_status_row_data_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data owner child split" => Some(
            "runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        // Runtime 15 M3 scene-script row-data owner child split.
        // Status: runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files:
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs
        // - plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs
        // Guard: runtime_15_scene_script_row_data_owner_is_child_backed.
        "Runtime 15 M3 scene-script row-data guard folder-backed split" => Some(
            "runtime_15_scene_script_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data status-mirror child split" => Some(
            "runtime_15_scene_script_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data root inventory child split" => Some(
            "runtime_15_scene_script_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data source/status-map sync" => Some(
            "runtime_15_scene_script_row_data_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script Runtime 07 performance row-data child split" => Some(
            "runtime_15_scene_script_runtime_07_performance_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script Runtime 07 performance guard folder-backed split" => Some(
            "runtime_15_scene_script_runtime_07_performance_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 facade expected status-map source reconciliation" => Some(
            "runtime_15_facade_expected_status_map_source_reconciliation_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

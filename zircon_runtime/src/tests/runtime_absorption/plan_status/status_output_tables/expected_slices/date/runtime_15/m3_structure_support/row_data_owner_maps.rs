pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 foundation-guards row-data owner child split" {
        Some("2026-07-02")
    } else if slice == "Runtime 15 M3 scene-script row-data owner child split" {
        Some("2026-07-02")
    } else if slice == "Runtime 15 M3 lock-poison status row-data owner child split" {
        Some("2026-07-02")
    } else if slice == "Runtime 15 M3 foundation-guards row-data source/status-map sync" {
        Some("2026-07-08")
    } else {
        None
    }
}

// Status: runtime_15_foundation_guards_row_data_owner_child_split_static_passed_cargo_deferred.
// Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/dead_code_surface.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/plugin_importer_review.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/plugin_importer_migrations.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_absorption_followups.rs.
// Guard: runtime_15_foundation_guards_row_data_owner_is_child_backed.
// Status: runtime_15_lock_poison_status_row_data_owner_child_split_static_passed_cargo_deferred.
// Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/status_rows.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/core_runtime_recovery.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/runtime_services_recovery.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/resource_render_input_recovery.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs.
// Guard: runtime_15_lock_poison_status_row_data_owner_is_child_backed.

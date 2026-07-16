type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 script VM registry lock poison recovery",
        &[
            "runtime_15_script_vm_registry_lock_poison_recovery_static_passed_cargo_deferred",
            "script/vm/backend/backend_registry.rs",
            "script/vm/host/host_export_registry.rs",
            "script/vm/runtime/hot_reload_coordinator.rs",
            "hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
            "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries",
        ],
    ),
    (
        "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery",
        &[
            "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_static_passed_cargo_deferred",
            "script/vm/runtime/vm_plugin_manager.rs",
            "docs/zircon_runtime/script/vm/zr_vm_host_reflection.md",
            "vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock",
            "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector",
        ],
    ),
];

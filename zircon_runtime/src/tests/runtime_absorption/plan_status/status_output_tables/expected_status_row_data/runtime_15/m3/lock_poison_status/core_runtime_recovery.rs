type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 config store lock poison recovery",
        &[
            "runtime_15_config_store_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/config_store.rs",
            "docs/zircon_runtime/core/runtime/config_store.md",
            "runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store",
        ],
    ),
    (
        "Runtime 15 M3 core runtime devtools lock poison recovery",
        &[
            "runtime_15_core_runtime_devtools_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/diagnostics/devtools.rs",
            "devtools_snapshot_recovers_poisoned_runtime_registry_locks",
            "runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot",
        ],
    ),
    (
        "Runtime 15 M3 core handle diagnostics lock poison recovery",
        &[
            "runtime_15_core_handle_diagnostics_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/diagnostics.rs",
            "core_handle_diagnostic_accessors_recover_poisoned_store_lock",
            "runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store",
        ],
    ),
    (
        "Runtime 15 M3 core handle time lock poison recovery",
        &[
            "runtime_15_core_handle_time_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/time.rs",
            "core_handle_time_accessors_recover_poisoned_runtime_clocks",
            "runtime_15_core_handle_time_lock_poison_recovery_guard_covers_runtime_clocks",
        ],
    ),
    (
        "Runtime 15 M3 core handle states lock poison recovery",
        &[
            "runtime_15_core_handle_states_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/states.rs",
            "core_handle_state_accessors_recover_poisoned_state_registry_lock",
            "runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry",
        ],
    ),
    (
        "Runtime 15 M3 core runtime task lock poison recovery",
        &[
            "runtime_15_core_runtime_task_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/tasks/job_handle.rs",
            "core/runtime/tasks/job_scheduler.rs",
            "job_handle_accessors_recover_poisoned_state_lock",
            "pending_scheduled_job_recovers_poisoned_task_lock",
            "runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles",
        ],
    ),
    (
        "Runtime 15 M3 core runtime profiling lock poison recovery",
        &[
            "runtime_15_core_runtime_profiling_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/diagnostics/profiling/mod.rs",
            "profile_recorder_accessors_recover_poisoned_global_lock",
            "runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder",
        ],
    ),
    (
        "Runtime 15 M3 core handle registry lock poison recovery",
        &[
            "runtime_15_core_handle_registry_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/core_handle.rs",
            "core/runtime/handle/registration/register_module.rs",
            "core_handle_registry_accessors_recover_poisoned_runtime_locks",
            "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors",
        ],
    ),
];

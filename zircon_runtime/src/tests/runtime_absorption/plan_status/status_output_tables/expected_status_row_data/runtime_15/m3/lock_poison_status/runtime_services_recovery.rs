type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 plugin bridge table lock poison recovery",
        &[
            "runtime_15_plugin_bridge_table_lock_poison_recovery_static_passed_cargo_deferred",
            "plugin/bridge/table.rs",
            "docs/zircon_runtime/plugin/bridge.md",
            "bridge_entry_provider_accessors_recover_poisoned_provider_lock",
            "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot",
        ],
    ),
    (
        "Runtime 15 M3 native live-host bridge methods lock poison recovery",
        &[
            "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs",
            "docs/zircon_runtime/plugin/bridge.md",
            "native_live_host_bridge_method_bindings_recover_poisoned_lock",
            "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry",
        ],
    ),
    (
        "Runtime 15 M3 navigation lock poison recovery",
        &[
            "runtime_15_navigation_lock_poison_recovery_static_passed_cargo_deferred",
            "navigation/runtime.rs",
            "docs/zircon_runtime/navigation/runtime.md",
            "runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager",
        ],
    ),
    (
        "Runtime 15 M3 dynamic API session lock poison recovery",
        &[
            "runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/tests/lock_poison.rs",
            "runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry",
        ],
    ),
    (
        "Runtime 15 M3 dynamic scene spawn task lock poison recovery",
        &[
            "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_static_passed_cargo_deferred",
            "scene/dynamic_scene/spawn_task/task.rs",
            "scene/dynamic_scene/spawn_task/loader.rs",
            "dynamic_scene_spawn_task_accessors_recover_poisoned_locks",
            "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS parallel executor lock poison recovery",
        &[
            "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_static_passed_cargo_deferred",
            "scene/ecs/schedule_parallel_executor.rs",
            "docs/zircon_runtime/scene/ecs.md",
            "schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock",
            "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots",
        ],
    ),
];

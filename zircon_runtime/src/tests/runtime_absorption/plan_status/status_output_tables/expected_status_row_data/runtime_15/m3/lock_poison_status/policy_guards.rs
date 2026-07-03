type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 lock poison policy guard folder split",
        &[
            "runtime_15_lock_poison_policy_guard_folder_split_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy.rs",
            "structure_convention/lock_poison_policy/core_runtime.rs",
            "structure_convention/lock_poison_policy/runtime_services.rs",
            "structure_convention/lock_poison_policy/asset_render_input.rs",
            "runtime_15_lock_poison_policy_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 core runtime lock poison guard child-owner split",
        &[
            "runtime_15_core_runtime_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy/core_runtime.rs",
            "structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
            "runtime_15_core_runtime_lock_poison_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 asset/render/input lock-poison guard child-owner split",
        &[
            "runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy/asset_render_input.rs",
            "structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
            "structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
            "structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
            "runtime_15_asset_render_input_lock_poison_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 runtime services lock-poison guard child-owner split",
        &[
            "runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy/runtime_services.rs",
            "structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
            "structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
            "structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
            "runtime_15_runtime_services_lock_poison_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 F2 lock poison recovery guard",
        &[
            "runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending",
            "structure_convention/lock_poison_policy.rs",
            "scene/level_system.rs",
            "core/runtime/events.rs",
            "review_f2_scene_eventbus_locks_recover_after_poison",
            "scene/EventBus poison-safe lock recovery complete",
            "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus",
        ],
    ),
    (
        "Runtime 15 M3 production direct lock unwrap global gate",
        &[
            "runtime_15_production_direct_lock_unwrap_global_gate_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy/core_runtime.rs",
            "LOCK_UNWRAP_CALL",
            "runtime_15_production_sources_do_not_directly_unwrap_mutex_locks",
        ],
    ),
];

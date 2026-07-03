type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 core resource manager lock poison recovery",
        &[
            "runtime_15_core_resource_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "core/resource/manager/resource_manager.rs",
            "core/resource/manager/registry_ops.rs",
            "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager",
        ],
    ),
    (
        "Runtime 15 M3 asset project manager lock poison recovery",
        &[
            "runtime_15_asset_project_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "asset/pipeline/manager/project_asset_manager/runtime.rs",
            "asset/pipeline/manager/project_asset_manager/construction.rs",
            "project_asset_manager_runtime_accessors_recover_poisoned_locks",
            "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager",
        ],
    ),
    (
        "Runtime 15 M3 asset worker pool lock poison recovery",
        &[
            "runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred",
            "asset/pipeline/worker_pool.rs",
            "asset/pipeline/manager/service_contracts/asset_manager_contract.rs",
            "asset_worker_pool_accessors_recover_poisoned_locks",
            "runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool",
        ],
    ),
    (
        "Runtime 15 M3 WGPU render framework lock poison recovery",
        &[
            "runtime_15_wgpu_render_framework_lock_poison_recovery_static_passed_cargo_deferred",
            "graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "wgpu_render_framework_accessors_recover_poisoned_locks",
            "runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework",
        ],
    ),
    (
        "Runtime 15 M3 RHI WGPU render device lock poison recovery",
        &[
            "runtime_15_rhi_wgpu_render_device_lock_poison_recovery_static_passed_cargo_deferred",
            "rhi_wgpu/device.rs",
            "docs/zircon_runtime/rhi/descriptors.md",
            "wgpu_render_device_state_accessors_recover_poisoned_lock",
            "runtime_15_rhi_wgpu_render_device_lock_poison_recovery_guard_covers_device_state",
        ],
    ),
    (
        "Runtime 15 M3 animation manager lock poison recovery",
        &[
            "runtime_15_animation_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "animation/manager/mod.rs",
            "animation_manager_playback_settings_recover_poisoned_lock",
            "runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings",
        ],
    ),
    (
        "Runtime 15 M3 input runtime manager lock poison recovery",
        &[
            "runtime_15_input_runtime_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "input/runtime/default_input_manager.rs",
            "input/runtime/default_input_action_manager.rs",
            "input_manager_accessors_recover_poisoned_state_lock",
            "runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state",
        ],
    ),
];

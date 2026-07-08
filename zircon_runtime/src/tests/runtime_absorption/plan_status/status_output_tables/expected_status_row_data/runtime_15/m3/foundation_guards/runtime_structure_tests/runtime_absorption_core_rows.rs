type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 job-system route-owner split",
        &[
            "runtime_15_job_system_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/job_system.rs",
            "tests/runtime_absorption/job_system/inventory.rs",
            "tests/runtime_absorption/job_system/mirror_docs.rs",
            "tests/runtime_absorption/job_system/source_helpers.rs",
            "runtime_15_job_system_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script-binding route-owner split",
        &[
            "runtime_15_script_binding_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/script_binding.rs",
            "tests/runtime_absorption/script_binding/inventory.rs",
            "tests/runtime_absorption/script_binding/mirror_docs.rs",
            "tests/runtime_absorption/script_binding/gameplay_host.rs",
            "tests/runtime_absorption/script_binding/support.rs",
            "runtime_15_script_binding_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset-pipeline route-owner split",
        &[
            "runtime_15_asset_pipeline_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/asset_pipeline.rs",
            "tests/runtime_absorption/asset_pipeline/inventory.rs",
            "tests/runtime_absorption/asset_pipeline/mirror_docs.rs",
            "tests/runtime_absorption/asset_pipeline/support.rs",
            "runtime_15_asset_pipeline_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset-surface route-owner split",
        &[
            "runtime_15_asset_surface_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/asset_surface.rs",
            "tests/runtime_absorption/asset_surface/registration.rs",
            "tests/runtime_absorption/asset_surface/namespace_surface.rs",
            "tests/runtime_absorption/asset_surface/facade_query.rs",
            "tests/runtime_absorption/asset_surface/support.rs",
            "runtime_15_asset_surface_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset-worker-policy route-owner split",
        &[
            "runtime_15_asset_worker_policy_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/asset_worker_policy.rs",
            "tests/runtime_absorption/asset_worker_policy/worker_pool.rs",
            "tests/runtime_absorption/asset_worker_policy/split_layout.rs",
            "runtime_15_asset_worker_policy_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 builtin-modules route-owner split",
        &[
            "runtime_15_builtin_modules_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/builtin_modules.rs",
            "tests/runtime_absorption/builtin_modules/core_spine.rs",
            "tests/runtime_absorption/builtin_modules/plugin_selection.rs",
            "tests/runtime_absorption/builtin_modules/split_layout.rs",
            "runtime_15_builtin_modules_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 rayon-boundary route-owner split",
        &[
            "runtime_15_rayon_boundary_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/rayon_boundary.rs",
            "tests/runtime_absorption/rayon_boundary/production_scan.rs",
            "tests/runtime_absorption/rayon_boundary/cutover_status.rs",
            "tests/runtime_absorption/rayon_boundary/support.rs",
            "tests/runtime_absorption/rayon_boundary/split_layout.rs",
            "runtime_15_rayon_boundary_route_owner_is_folder_backed",
        ],
    ),
];

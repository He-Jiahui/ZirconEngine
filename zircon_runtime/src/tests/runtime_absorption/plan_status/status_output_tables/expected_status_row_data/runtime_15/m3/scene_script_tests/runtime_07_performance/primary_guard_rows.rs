type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
        &[
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
            "tests/runtime_absorption/performance_hotspots.rs",
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            "runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 submit-context guard child-owner split",
        &[
            "runtime_15_runtime_07_submit_context_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            "tests/runtime_absorption/performance_hotspots/submit_context/source_extract_payloads.rs",
            "tests/runtime_absorption/performance_hotspots/submit_context/camera_loop_sharing.rs",
            "tests/runtime_absorption/performance_hotspots/submit_context/feedback_sidebands.rs",
            "runtime_15_runtime_07_submit_context_guard_child_owner_split",
            "expected_test_file_count = 20",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split",
        &[
            "runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs",
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/profiling_trace_render.rs",
            "runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split",
            "expected_test_file_count = 25",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split",
        &[
            "runtime_15_runtime_07_owner_budget_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/parent_routes.rs",
            "tests/runtime_absorption/performance_hotspots/owner_budget/status_docs.rs",
            "runtime_15_runtime_07_owner_budget_guard_folder_backed_split",
            "expected_test_file_count = 32",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split",
        &[
            "runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs",
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/artifact_cache_payload.rs",
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/render_product_diagnostics.rs",
            "runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split",
            "expected_test_file_count = 35",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 07 scene/project guard child-owner split",
        &[
            "runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/performance_hotspots/scene_project_splits.rs",
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/scene_asset.rs",
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/project_io.rs",
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs",
            "runtime_15_runtime_07_scene_project_guard_child_owner_split",
            "expected_test_file_count = 39",
        ],
    ),
];

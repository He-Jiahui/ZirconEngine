type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 schedule-frame-loop route-owner split",
        &[
            "runtime_15_schedule_frame_loop_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/schedule_frame_loop.rs",
            "tests/runtime_absorption/schedule_frame_loop/inventory.rs",
            "tests/runtime_absorption/schedule_frame_loop/runtime_anchors.rs",
            "tests/runtime_absorption/schedule_frame_loop/mirror_docs.rs",
            "tests/runtime_absorption/schedule_frame_loop/split_layout.rs",
            "runtime_15_schedule_frame_loop_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 tech-stack route-owner split",
        &[
            "runtime_15_tech_stack_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/tech_stack.rs",
            "tests/runtime_absorption/tech_stack/manifest_inventory.rs",
            "tests/runtime_absorption/tech_stack/guard_anchors.rs",
            "tests/runtime_absorption/tech_stack/behavior_anchors.rs",
            "tests/runtime_absorption/tech_stack/mirror_docs.rs",
            "tests/runtime_absorption/tech_stack/split_layout.rs",
            "runtime_15_tech_stack_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script-absorption route-owner split",
        &[
            "runtime_15_script_absorption_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/script_absorption.rs",
            "tests/runtime_absorption/script_absorption/legacy_crate.rs",
            "tests/runtime_absorption/script_absorption/split_layout.rs",
            "runtime_15_script_absorption_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 resource-foundation route-owner split",
        &[
            "runtime_15_resource_foundation_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/resource_foundation.rs",
            "tests/runtime_absorption/resource_foundation/runtime_surface.rs",
            "tests/runtime_absorption/resource_foundation/split_layout.rs",
            "runtime_15_resource_foundation_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 compatibility-shells route-owner split",
        &[
            "runtime_15_compatibility_shells_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/compatibility_shells.rs",
            "tests/runtime_absorption/compatibility_shells/nested_crates.rs",
            "tests/runtime_absorption/compatibility_shells/split_layout.rs",
            "runtime_15_compatibility_shells_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 ui-architecture route-owner split",
        &[
            "runtime_15_ui_architecture_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/ui_architecture.rs",
            "tests/runtime_absorption/ui_architecture/architecture_boundaries.rs",
            "tests/runtime_absorption/ui_architecture/legacy_renames.rs",
            "tests/runtime_absorption/ui_architecture/mirror_docs.rs",
            "tests/runtime_absorption/ui_architecture/support.rs",
            "tests/runtime_absorption/ui_architecture/split_layout.rs",
            "runtime_15_ui_architecture_route_owner_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 dynamic-scene route-owner split",
        &[
            "runtime_15_dynamic_scene_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/dynamic_scene.rs",
            "tests/runtime_absorption/dynamic_scene/sources.rs",
            "tests/runtime_absorption/dynamic_scene/patch_preview_api.rs",
            "tests/runtime_absorption/dynamic_scene/patch_preview_behavior.rs",
            "tests/runtime_absorption/dynamic_scene/patch_preview_status_docs.rs",
            "tests/runtime_absorption/dynamic_scene/session_capture_persistence.rs",
            "tests/runtime_absorption/dynamic_scene/session_load_query_path.rs",
            "tests/runtime_absorption/dynamic_scene/session_retention_mutation_merge.rs",
            "tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs",
            "tests/runtime_absorption/dynamic_scene/split_layout.rs",
            "runtime_15_dynamic_scene_route_owner_is_folder_backed",
        ],
    ),
];

type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 graphics render-framework receiver guard child-owner split",
        &[
            "runtime_15_graphics_render_framework_receiver_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_framework_receiver.rs",
            "runtime_15_graphics_render_framework_receiver_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 graphics resource-streamer guard child-owner split",
        &[
            "runtime_15_graphics_resource_streamer_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/resource_streamer_construction.rs",
            "runtime_15_graphics_resource_streamer_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 graphics offscreen-target guard child-owner split",
        &[
            "runtime_15_graphics_offscreen_target_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/offscreen_target_construct.rs",
            "runtime_15_graphics_offscreen_target_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 graphics GPU-model guard child-owner split",
        &[
            "runtime_15_graphics_gpu_model_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/gpu_model_embedded_primitive.rs",
            "runtime_15_graphics_gpu_model_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 asset-schema material guard child-owner split",
        &[
            "runtime_15_asset_schema_material_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema/material_asset_schema_v1.rs",
            "runtime_15_asset_schema_material_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 core-scene render-layer schema-v1 guard child-owner split",
        &[
            "runtime_15_core_scene_render_layer_schema_v1_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/render_layer_schema_v1.rs",
            "runtime_15_core_scene_render_layer_schema_v1_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 core-scene runtime-state guard child-owner split",
        &[
            "runtime_15_core_scene_runtime_state_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/core_runtime_state.rs",
            "runtime_15_core_scene_runtime_state_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 scene-tests ECS systems guard child-owner split",
        &[
            "runtime_15_scene_tests_ecs_systems_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests/ecs_systems.rs",
            "runtime_15_scene_tests_ecs_systems_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Net HTTP policy guard child-owner split",
        &[
            "runtime_15_net_http_policy_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/net/http1_client_policy.rs",
            "runtime_15_net_http_policy_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Hub raw-text policy guard child-owner split",
        &[
            "runtime_15_hub_raw_text_policy_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub/raw_text_policy.rs",
            "runtime_15_hub_raw_text_policy_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 input mouse-wheel line-delta guard child-owner split",
        &[
            "runtime_15_input_mouse_wheel_line_delta_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/input.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/input/mouse_wheel_line_delta.rs",
            "runtime_15_input_mouse_wheel_line_delta_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 UI platform-input guard child-owner split",
        &[
            "runtime_15_ui_platform_input_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui/platform_input.rs",
            "runtime_15_ui_platform_input_guards_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 plugin static manifest naming guard child-owner split",
        &[
            "runtime_15_plugin_static_manifest_naming_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest/contract_owners.rs",
            "runtime_15_plugin_static_manifest_naming_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 banned-name scene-dynamic guard child-owner split",
        &[
            "runtime_15_banned_names_scene_dynamic_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs",
            "runtime_15_banned_names_scene_dynamic_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 banned-name graphics construction guard child-owner split",
        &[
            "runtime_15_banned_names_graphics_construction_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/graphics_construction.rs",
            "runtime_15_banned_names_graphics_construction_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 banned-name global module guard child-owner split",
        &[
            "runtime_15_banned_names_global_module_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/global_modules.rs",
            "runtime_15_banned_names_global_module_guard_is_child_owner",
        ],
    ),
];

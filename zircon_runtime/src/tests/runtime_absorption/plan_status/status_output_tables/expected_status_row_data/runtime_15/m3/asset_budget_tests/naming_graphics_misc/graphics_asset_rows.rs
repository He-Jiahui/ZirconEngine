type Slice = super::Slice;

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
];

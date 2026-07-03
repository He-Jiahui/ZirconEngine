type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 core-framework naming camera-controller guard child-owner split",
        &[
            "runtime_15_core_framework_naming_camera_controller_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs",
            "runtime_15_core_framework_naming_camera_controller_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 core-framework naming render-fixture guard child-owner split",
        &[
            "runtime_15_core_framework_naming_render_fixture_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_fixtures.rs",
            "runtime_15_core_framework_naming_render_fixture_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 core-framework render-layer schema-v1 guard child-owner split",
        &[
            "runtime_15_core_framework_render_layer_schema_v1_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_layer_schema_v1.rs",
            "runtime_15_core_framework_render_layer_schema_v1_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 graphics naming render-fixture guard child-owner split",
        &[
            "runtime_15_graphics_naming_render_fixture_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs",
            "runtime_15_graphics_naming_render_fixture_guards_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 core-scene naming render-contract guard child-owner split",
        &[
            "runtime_15_core_scene_naming_render_contract_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/render_contracts.rs",
            "runtime_15_core_scene_naming_render_contract_guards_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 core-scene naming ECS owner guard child-owner split",
        &[
            "runtime_15_core_scene_naming_ecs_owner_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs",
            "runtime_15_core_scene_naming_ecs_owner_guards_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 asset-dynamic naming texture-container guard child-owner split",
        &[
            "runtime_15_asset_dynamic_texture_container_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/texture_containers.rs",
            "runtime_15_asset_dynamic_texture_container_guards_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 asset-dynamic asset-watch guard child-owner split",
        &[
            "runtime_15_asset_dynamic_asset_watch_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/asset_watch.rs",
            "runtime_15_asset_dynamic_asset_watch_guards_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 asset-dynamic scene-ECS query guard child-owner split",
        &[
            "runtime_15_asset_dynamic_scene_ecs_query_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/scene_ecs_queries.rs",
            "runtime_15_asset_dynamic_scene_ecs_query_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 asset-dynamic dynamic-API vampire guard child-owner split",
        &[
            "runtime_15_asset_dynamic_dynamic_api_vampire_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs",
            "runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner",
        ],
    ),
];

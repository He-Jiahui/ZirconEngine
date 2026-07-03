pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 core-framework naming camera-controller guard child-owner split" => Some(
            "runtime_15_core_framework_naming_camera_controller_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 core-framework naming render-fixture guard child-owner split" => Some(
            "runtime_15_core_framework_naming_render_fixture_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 core-framework render-layer schema-v1 guard child-owner split" => Some(
            "runtime_15_core_framework_render_layer_schema_v1_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 graphics naming render-fixture guard child-owner split" => Some(
            "runtime_15_graphics_naming_render_fixture_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 core-scene naming render-contract guard child-owner split" => Some(
            "runtime_15_core_scene_naming_render_contract_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 core-scene naming ECS owner guard child-owner split" => Some(
            "runtime_15_core_scene_naming_ecs_owner_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset-dynamic naming texture-container guard child-owner split" => Some(
            "runtime_15_asset_dynamic_texture_container_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset-dynamic asset-watch guard child-owner split" => Some(
            "runtime_15_asset_dynamic_asset_watch_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset-dynamic scene-ECS query guard child-owner split" => Some(
            "runtime_15_asset_dynamic_scene_ecs_query_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset-dynamic dynamic-API vampire guard child-owner split" => Some(
            "runtime_15_asset_dynamic_dynamic_api_vampire_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 graphics render-framework receiver guard child-owner split" => Some(
            "runtime_15_graphics_render_framework_receiver_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 graphics resource-streamer guard child-owner split" => Some(
            "runtime_15_graphics_resource_streamer_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 graphics offscreen-target guard child-owner split" => Some(
            "runtime_15_graphics_offscreen_target_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 graphics GPU-model guard child-owner split" => Some(
            "runtime_15_graphics_gpu_model_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset-schema material guard child-owner split" => Some(
            "runtime_15_asset_schema_material_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 core-scene render-layer schema-v1 guard child-owner split" => Some(
            "runtime_15_core_scene_render_layer_schema_v1_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 core-scene runtime-state guard child-owner split" => Some(
            "runtime_15_core_scene_runtime_state_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-tests ECS systems guard child-owner split" => Some(
            "runtime_15_scene_tests_ecs_systems_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Net HTTP policy guard child-owner split" => {
            Some("runtime_15_net_http_policy_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Hub raw-text policy guard child-owner split" => {
            Some("runtime_15_hub_raw_text_policy_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 input mouse-wheel line-delta guard child-owner split" => Some(
            "runtime_15_input_mouse_wheel_line_delta_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI platform-input guard child-owner split" => Some(
            "runtime_15_ui_platform_input_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin static manifest naming guard child-owner split" => Some(
            "runtime_15_plugin_static_manifest_naming_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 banned-name scene-dynamic guard child-owner split" => Some(
            "runtime_15_banned_names_scene_dynamic_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 banned-name graphics construction guard child-owner split" => Some(
            "runtime_15_banned_names_graphics_construction_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 banned-name global module guard child-owner split" => Some(
            "runtime_15_banned_names_global_module_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

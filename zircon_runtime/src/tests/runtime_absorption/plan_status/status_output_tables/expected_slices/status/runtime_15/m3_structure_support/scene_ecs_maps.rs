pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS schedule conflict graph child folder split" {
        Some("runtime_15_scene_ecs_schedule_conflict_graph_child_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS systems test folder split" {
        Some("runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query test folder split" {
        Some("runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query structure test folder split" {
        Some("runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene derived-state test folder split" {
        Some("runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene session path-management test folder split" {
        Some("runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene component-structure test folder split" {
        Some("runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS reflect foundation test folder split" {
        Some("runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene root test folder split" {
        Some("runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene render extract test folder split" {
        Some("runtime_15_scene_render_extract_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene asset integration test folder split" {
        Some("runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene world basics test folder split" {
        Some("runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene property paths test folder split" {
        Some("runtime_15_scene_property_paths_tests_folder_split_static_passed_cargo_deferred")
    } else {
        None
    }
}

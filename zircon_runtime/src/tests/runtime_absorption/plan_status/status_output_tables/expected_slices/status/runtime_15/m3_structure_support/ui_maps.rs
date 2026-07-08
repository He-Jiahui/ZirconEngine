pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 input manager test folder split" {
        Some("runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI v2 style-runtime test folder split" {
        Some("runtime_15_ui_v2_style_runtime_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI shared core guard child-owner split" {
        Some("runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI shared core input visibility child folder split" {
        Some("runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI shared core scroll mutation child folder split" {
        Some("runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI shared core layout surface child folder split" {
        Some("runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI accessibility test folder split" {
        Some("runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI accessibility widget actions test folder split" {
        Some("runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI layout slots test folder split" {
        Some("runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI surface-frame authority test folder split" {
        Some(
            "runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI surface dirty domains test folder split" {
        Some("runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI material layout test folder split" {
        Some("runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI template test folder split" {
        Some("runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI component catalog test folder split" {
        Some("runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI boundary test folder split" {
        Some("runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI boundary ZUI surface projection guard sync" {
        Some(
            "runtime_15_ui_boundary_zui_surface_projection_guard_sync_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI component state test folder split" {
        Some("runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI component state keyboard test folder split" {
        Some("runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI Material foundation test folder split" {
        Some("runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI event routing test folder split" {
        Some("runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply routes test folder split" {
        Some("runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply route child folder split" {
        Some("runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply table pointer route folder split" {
        Some("runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply route guard child-owner split" {
        Some("runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred")
    } else {
        None
    }
}

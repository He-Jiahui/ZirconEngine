pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 test file budget root-layout status scan child split" => Some(
            "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split" => Some(
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
        ),
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split" => Some(
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split" => Some(
            "runtime_15_test_file_budget_root_layout_assertions_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync" => Some(
            "runtime_15_root_layout_status_output_runtime_15_row_data_child_source_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 root entries/root-layout current-child route sync" => Some(
            "runtime_15_m3_root_entries_root_layout_current_child_route_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget parent guard child-owner split" => Some(
            "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 historical oversized test roots closeout" => {
            Some("runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 asset test-budget guard child-owner split" => {
            Some("runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI asset test folder split" => {
            Some("runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI asset surface index test folder split" => {
            Some("runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI asset MUI web form style test folder split" => Some(
            "runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI asset MUI X web style test folder split" => Some(
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI asset MUI web style test folder split" => {
            Some("runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI taffy layout pass test folder split" => {
            Some("runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI runtime window input pump test folder split" => Some(
            "runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI runtime window event ABI child folder split" => Some(
            "runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget root-layout UI child split" => {
            Some("runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI widget text input keyboard test folder split" => Some(
            "runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI focus navigation test folder split" => {
            Some("runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI runtime input manager test folder split" => {
            Some("runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI runtime input ownership test folder split" => Some(
            "runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

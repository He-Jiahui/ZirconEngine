pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 support Hub project-actions tests child-owner split" => Some(
            "runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 support Hub runtime-state tests child-owner split" => Some(
            "runtime_15_support_hub_runtime_state_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split" => Some(
            "runtime_15_support_hub_view_model_quick_actions_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split" => Some(
            "runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard" => Some(
            "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

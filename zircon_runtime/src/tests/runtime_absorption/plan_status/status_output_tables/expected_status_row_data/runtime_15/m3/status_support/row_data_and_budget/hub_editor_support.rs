type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 support Hub project-actions tests child-owner split",
        &[
            "runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_hub/src/tauri_app/runtime_state/project_actions.rs",
            "zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs",
            "runtime_15_support_hub_project_actions_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 support Hub runtime-state tests child-owner split",
        &[
            "runtime_15_support_hub_runtime_state_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_hub/src/tauri_app/runtime_state.rs",
            "zircon_hub/src/tauri_app/runtime_state/tests.rs",
            "runtime_15_support_hub_runtime_state_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split",
        &[
            "runtime_15_support_hub_view_model_quick_actions_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_hub/src/tauri_app/view_model.rs",
            "zircon_hub/src/tauri_app/view_model/quick_actions.rs",
            "zircon_hub/src/tauri_app/view_model/tests.rs",
            "runtime_15_support_hub_view_model_quick_actions_tests_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split",
        &[
            "runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs",
            "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs",
            "runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard",
        &[
            "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs",
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs",
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs",
            "zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs",
            "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners",
        ],
    ),
];

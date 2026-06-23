use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 12 action context routing",
        &[
            "InputActionContext",
            "evaluate_with_active_contexts",
            "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
            "behavior_test_anchor_count = 7",
        ],
    ),
    (
        "Runtime 12 action axis value bindings",
        &[
            "gamepad_axis_binding_reports_continuous_action_value",
            "InputActionState::value",
            "public_surface_anchors = 13/13",
            "behavior_test_anchor_count = 8",
        ],
    ),
    (
        "Runtime 12 action map config source",
        &[
            "input_config_builds_action_evaluator_from_serialized_action_map",
            "InputConfig::action_evaluator",
            "public_surface_anchors = 14/14",
            "behavior_test_anchor_count = 9",
        ],
    ),
    (
        "Runtime 12 action manager registration path",
        &[
            "input_action_manager_resolves_from_runtime_module_descriptor",
            "resolve_input_action_manager",
            "public_surface_anchors = 17/17",
            "behavior_test_anchor_count = 10",
        ],
    ),
];

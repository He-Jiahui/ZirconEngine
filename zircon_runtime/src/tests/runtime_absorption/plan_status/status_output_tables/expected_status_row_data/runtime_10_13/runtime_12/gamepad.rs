use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 12 gamepad event-owner 漂移同步",
        [
            "session/events.rs",
            "InputEvent::Gamepad*",
            "missing_gamepad_abi_anchors = []",
            "standalone `input_stack.rs` rustc 4/4 passed",
        ],
    ),
    (
        "Runtime 12 gamepad bridge source guard event-owner sync",
        [
            "gamepad_host_bridge_uses_runtime_gamepad_abi_constructors",
            "session/events.rs",
            "public_surface_anchor_count = 11",
            "605s timeout no result",
        ],
    ),
    (
        "Runtime 12 gamepad axis transition edges",
        [
            "gamepad_axis_action_reports_deadzone_transition_edges",
            "GamepadAxisTransition",
            "public_surface_anchors = 19/19",
            "behavior_test_anchor_count = 12",
        ],
    ),
    (
        "Runtime 12 consumed gamepad axis arbitration",
        [
            "consumed_gamepad_axis_does_not_activate_gameplay_action",
            "GamepadAxisInput",
            "evaluate_with_consumed_input",
            "public_surface_anchors = 19/19",
        ],
    ),
];

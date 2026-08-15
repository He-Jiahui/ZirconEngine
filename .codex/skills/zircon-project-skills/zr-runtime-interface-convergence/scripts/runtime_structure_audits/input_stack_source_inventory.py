from __future__ import annotations


INPUT_PRODUCTION_MODULE_MAX_LINES = 500
INPUT_TEST_MODULE_MAX_LINES = 800
EXPECTED_INPUT_STACK_GUARD_FILE_COUNT = 6
INPUT_STACK_GUARD_FILES = (
    "zircon_runtime/src/tests/runtime_absorption/input_stack.rs",
    "zircon_runtime/src/tests/runtime_absorption/input_stack/contracts.rs",
    "zircon_runtime/src/tests/runtime_absorption/input_stack/action_mapping.rs",
    "zircon_runtime/src/tests/runtime_absorption/input_stack/gamepad_bridge.rs",
    "zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/behavior_anchors.rs",
    "zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs",
)
INPUT_RUNTIME_MODULES = (
    "zircon_runtime/src/input/mod.rs",
    "zircon_runtime/src/input/prelude.rs",
    "zircon_runtime/src/input/module/config.rs",
    "zircon_runtime/src/input/module/descriptor.rs",
    "zircon_runtime/src/input/module/mod.rs",
    "zircon_runtime/src/input/module/module_type.rs",
    "zircon_runtime/src/input/runtime/action_evaluator.rs",
    "zircon_runtime/src/input/runtime/action_evaluator/consumed_input_index.rs",
    "zircon_runtime/src/input/runtime/action_evaluator/generation.rs",
    "zircon_runtime/src/input/runtime/action_evaluator/workspace.rs",
    "zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs",
    "zircon_runtime/src/input/runtime/default_input_action_manager.rs",
    "zircon_runtime/src/input/runtime/default_input_manager.rs",
    "zircon_runtime/src/input/runtime/event_buffer/frame.rs",
    "zircon_runtime/src/input/runtime/event_buffer/mod.rs",
    "zircon_runtime/src/input/runtime/event_buffer/recorder.rs",
    "zircon_runtime/src/input/runtime/input_driver.rs",
    "zircon_runtime/src/input/runtime/recording.rs",
    "zircon_runtime/src/input/runtime/input_state.rs",
    "zircon_runtime/src/input/runtime/mod.rs",
)
FRAMEWORK_INPUT_MODULES = (
    "zircon_runtime/src/core/framework/input/button_input_state.rs",
    "zircon_runtime/src/core/framework/input/cursor.rs",
    "zircon_runtime/src/core/framework/input/event_retention/mod.rs",
    "zircon_runtime/src/core/framework/input/event_retention/queue_status.rs",
    "zircon_runtime/src/core/framework/input/event_retention/recording_config.rs",
    "zircon_runtime/src/core/framework/input/event_retention/recording_status.rs",
    "zircon_runtime/src/core/framework/input/file_drag_drop.rs",
    "zircon_runtime/src/core/framework/input/gamepad.rs",
    "zircon_runtime/src/core/framework/input/ime.rs",
    "zircon_runtime/src/core/framework/input/input_action.rs",
    "zircon_runtime/src/core/framework/input/input_action_context.rs",
    "zircon_runtime/src/core/framework/input/input_action_manager.rs",
    "zircon_runtime/src/core/framework/input/input_action_map.rs",
    "zircon_runtime/src/core/framework/input/input_action_state.rs",
    "zircon_runtime/src/core/framework/input/input_binding.rs",
    "zircon_runtime/src/core/framework/input/input_button.rs",
    "zircon_runtime/src/core/framework/input/input_event.rs",
    "zircon_runtime/src/core/framework/input/input_event_record.rs",
    "zircon_runtime/src/core/framework/input/input_frame_snapshot.rs",
    "zircon_runtime/src/core/framework/input/input_manager.rs",
    "zircon_runtime/src/core/framework/input/input_snapshot.rs",
    "zircon_runtime/src/core/framework/input/mod.rs",
    "zircon_runtime/src/core/framework/input/module_identity.rs",
    "zircon_runtime/src/core/framework/input/mouse_wheel.rs",
    "zircon_runtime/src/core/framework/input/touch.rs",
    "zircon_runtime/src/core/framework/input/window_status.rs",
)
INPUT_TEST_MODULES = (
    "zircon_runtime/src/input/tests/action_axis_transitions.rs",
    "zircon_runtime/src/input/tests/action_mapping.rs",
    "zircon_runtime/src/input/tests/boundary.rs",
    "zircon_runtime/src/input/tests/gamepad_bridge.rs",
    "zircon_runtime/src/input/tests/input_manager.rs",
    "zircon_runtime/src/input/tests/mod.rs",
    "zircon_runtime/src/input/tests/recording_replay.rs",
)

INPUT_MANAGER_CHILD_TEST_MODULES = (
    "zircon_runtime/src/input/tests/input_manager/event_buffer.rs",
    "zircon_runtime/src/input/tests/input_manager/frame_state.rs",
    "zircon_runtime/src/input/tests/input_manager/host_requests.rs",
)

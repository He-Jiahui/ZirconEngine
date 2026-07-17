use std::path::Path;

use super::super::support::assert_owner_files;

const INPUT_PRODUCTION_MODULE_MAX_LINES: usize = 500;
const INPUT_TEST_MODULE_MAX_LINES: usize = 800;
const EXPECTED_INPUT_RUNTIME_MODULES: &[&str] = &[
    "mod.rs",
    "prelude.rs",
    "module/config.rs",
    "module/descriptor.rs",
    "module/mod.rs",
    "module/module_type.rs",
    "runtime/action_evaluator.rs",
    "runtime/action_evaluator/binding_index.rs",
    "runtime/action_evaluator/frame_axis_index.rs",
    "runtime/default_input_action_manager.rs",
    "runtime/default_input_manager.rs",
    "runtime/event_buffer/frame.rs",
    "runtime/event_buffer/mod.rs",
    "runtime/event_buffer/recorder.rs",
    "runtime/input_driver.rs",
    "runtime/recording.rs",
    "runtime/input_state.rs",
    "runtime/mod.rs",
];
const EXPECTED_FRAMEWORK_INPUT_MODULES: &[&str] = &[
    "button_input_state.rs",
    "cursor.rs",
    "event_retention/mod.rs",
    "event_retention/queue_status.rs",
    "event_retention/recording_config.rs",
    "event_retention/recording_status.rs",
    "file_drag_drop.rs",
    "gamepad.rs",
    "ime.rs",
    "input_action.rs",
    "input_action_context.rs",
    "input_action_manager.rs",
    "input_action_map.rs",
    "input_action_state.rs",
    "input_binding.rs",
    "input_button.rs",
    "input_event.rs",
    "input_event_record.rs",
    "input_frame_snapshot.rs",
    "input_manager.rs",
    "input_snapshot.rs",
    "mod.rs",
    "mouse_wheel.rs",
    "touch.rs",
    "window_status.rs",
];
const EXPECTED_INPUT_TEST_MODULES: &[&str] = &[
    "action_axis_transitions.rs",
    "action_mapping.rs",
    "boundary.rs",
    "gamepad_bridge.rs",
    "input_manager.rs",
    "mod.rs",
    "recording_replay.rs",
];

#[test]
fn runtime_12_input_stack_inventory_owner_files_match_structure_audit_counts() {
    assert_eq!(EXPECTED_INPUT_RUNTIME_MODULES.len(), 18);
    assert_eq!(EXPECTED_FRAMEWORK_INPUT_MODULES.len(), 25);
    assert_eq!(EXPECTED_INPUT_TEST_MODULES.len(), 7);

    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_owner_files(
        &runtime_root.join("src").join("input"),
        EXPECTED_INPUT_RUNTIME_MODULES,
        INPUT_PRODUCTION_MODULE_MAX_LINES,
        "Runtime 12 input runtime owner",
    );
    assert_owner_files(
        &runtime_root
            .join("src")
            .join("core")
            .join("framework")
            .join("input"),
        EXPECTED_FRAMEWORK_INPUT_MODULES,
        INPUT_PRODUCTION_MODULE_MAX_LINES,
        "Runtime 12 framework input contract",
    );
    assert_owner_files(
        &runtime_root.join("src").join("input").join("tests"),
        EXPECTED_INPUT_TEST_MODULES,
        INPUT_TEST_MODULE_MAX_LINES,
        "Runtime 12 input test owner",
    );
}

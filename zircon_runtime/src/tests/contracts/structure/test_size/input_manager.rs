//! 核对输入管理器测试迁移后的父子分工与数量快照，并检查文件预算；出现次数来自源码文本，后续新增测试需按当前职责复核快照。
use super::*;

#[test]
fn runtime_15_input_manager_tests_are_folder_backed() {
    let parent = read_runtime_src("input/tests/input_manager.rs");
    let event_buffer = read_runtime_src("input/tests/input_manager/event_buffer.rs");
    let frame_state = read_runtime_src("input/tests/input_manager/frame_state.rs");
    let touch_gamepad = read_runtime_src("input/tests/input_manager/touch_gamepad.rs");
    let host_requests = read_runtime_src("input/tests/input_manager/host_requests.rs");

    assert_contains_all(
        "input manager parent keeps shared imports and mounts children",
        &parent,
        &[
            "mod event_buffer;",
            "mod frame_state;",
            "mod host_requests;",
            "mod touch_gamepad;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "input/tests/input_manager.rs should only keep shared imports and mount child owners"
    );
    for moved_test in [
        "fn input_manager_tracks_state_and_drains_events",
        "fn input_manager_records_sequences_and_timestamps_for_ui_bridge_consumers",
        "fn button_input_state_tracks_bevy_style_frame_transitions",
        "fn input_snapshot_just_pressed_is_true_for_exactly_one_frame",
        "fn input_manager_frame_snapshot_tracks_transitions_and_motion",
        "fn focus_loss_releases_active_input_without_disconnect",
        "fn input_manager_tracks_ime_preedit_and_frame_commits",
        "fn input_manager_tracks_touch_and_gamepad_state",
        "fn input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad",
        "fn gamepad_button_values_use_runtime_thresholds_and_hysteresis",
        "fn gamepad_axis_values_use_deadzone_livezone_and_change_threshold",
        "fn gamepad_rumble_requests_are_frame_local_and_drainable",
        "fn cursor_host_requests_are_frame_local_and_drainable",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved input manager test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "event-buffer child owns transient queues, pointer coalescing, and raw recording",
        &event_buffer,
        &[
            "fn pointer_event_streams_are_frame_bounded_at_common_polling_rates",
            "fn pointer_coalescing_preserves_button_and_touch_edges_in_order",
            "fn focus_loss_is_a_barrier_to_pointer_event_coalescing",
            "fn begin_frame_discards_undrained_transient_events",
            "fn recording_is_opt_in_bounded_and_reports_discarded_raw_records",
        ],
    );
    assert_contains_all(
        "frame-state child owns basic input, frame, focus, and IME coverage",
        &frame_state,
        &[
            "fn input_manager_tracks_state_and_drains_events",
            "fn input_manager_records_sequences_and_timestamps_for_ui_bridge_consumers",
            "fn button_input_state_tracks_bevy_style_frame_transitions",
            "fn input_snapshot_just_pressed_is_true_for_exactly_one_frame",
            "fn input_manager_frame_snapshot_tracks_transitions_and_motion",
            "fn focus_loss_releases_active_input_without_disconnect",
            "fn input_manager_tracks_ime_preedit_and_frame_commits",
        ],
    );
    assert_contains_all(
        "touch/gamepad child owns device state, event log, and filtering coverage",
        &touch_gamepad,
        &[
            "fn input_manager_tracks_touch_and_gamepad_state",
            "fn input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad",
            "fn runtime_input_event_log_harness",
            "fn mouse_button_label",
            "fn gamepad_button_values_use_runtime_thresholds_and_hysteresis",
            "fn gamepad_axis_values_use_deadzone_livezone_and_change_threshold",
        ],
    );
    assert_contains_all(
        "host request child owns frame-local host request queues",
        &host_requests,
        &[
            "fn gamepad_rumble_requests_are_frame_local_and_drainable",
            "fn cursor_host_requests_are_frame_local_and_drainable",
        ],
    );
    let child_test_total = [
        event_buffer.as_str(),
        frame_state.as_str(),
        touch_gamepad.as_str(),
        host_requests.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 23,
        "input manager children should retain the current 23-test inventory"
    );

    for (path, source) in [
        ("input/tests/input_manager.rs", parent.as_str()),
        (
            "input/tests/input_manager/event_buffer.rs",
            event_buffer.as_str(),
        ),
        (
            "input/tests/input_manager/frame_state.rs",
            frame_state.as_str(),
        ),
        (
            "input/tests/input_manager/touch_gamepad.rs",
            touch_gamepad.as_str(),
        ),
        (
            "input/tests/input_manager/host_requests.rs",
            host_requests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _input_doc = read_repo("docs/crates/zircon_runtime/input/input_state.md");
}

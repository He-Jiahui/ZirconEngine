use super::*;

#[test]
fn runtime_15_input_manager_tests_are_folder_backed() {
    let parent = read_runtime_src("input/tests/input_manager.rs");
    let frame_state = read_runtime_src("input/tests/input_manager/frame_state.rs");
    let touch_gamepad = read_runtime_src("input/tests/input_manager/touch_gamepad.rs");
    let host_requests = read_runtime_src("input/tests/input_manager/host_requests.rs");

    assert_contains_all(
        "input manager parent keeps shared imports and mounts children",
        &parent,
        &[
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
        "fn frame_input_clears_after_level_tick_not_before",
        "fn input_manager_frame_snapshot_tracks_transitions_and_motion",
        "fn keyboard_focus_lost_releases_keyboard_buttons_only",
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
        "frame-state child owns basic input, frame, focus, and IME coverage",
        &frame_state,
        &[
            "fn input_manager_tracks_state_and_drains_events",
            "fn input_manager_records_sequences_and_timestamps_for_ui_bridge_consumers",
            "fn button_input_state_tracks_bevy_style_frame_transitions",
            "fn input_snapshot_just_pressed_is_true_for_exactly_one_frame",
            "fn frame_input_clears_after_level_tick_not_before",
            "fn input_manager_frame_snapshot_tracks_transitions_and_motion",
            "fn keyboard_focus_lost_releases_keyboard_buttons_only",
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
        frame_state.as_str(),
        touch_gamepad.as_str(),
        host_requests.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 14,
        "input manager children should preserve all 14 parent tests"
    );

    for (path, source) in [
        ("input/tests/input_manager.rs", parent.as_str()),
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

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let input_doc = read_repo("docs/zircon_runtime/input/input_state.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );
    let status_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/ui_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/ui_maps.rs",
        ),
    ]
    .join("\n");

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("input state doc", input_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 input manager test folder split",
                "runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred",
                "input/tests/input_manager.rs",
                "input/tests/input_manager/frame_state.rs",
                "input/tests/input_manager/touch_gamepad.rs",
                "runtime_15_input_manager_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M3 input manager test folder split",
            "runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M3 input manager test folder split",
            "2026-06-24",
        ],
    );
}

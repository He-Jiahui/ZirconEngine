const EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS: &[&str] = &[
    "input_snapshot_just_pressed_is_true_for_exactly_one_frame",
    "frame_input_clears_after_level_tick_not_before",
    "action_map_resolves_chords_and_reports_just_activated",
    "replacing_action_map_rebuilds_bindings_automatically",
    "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
    "gamepad_axis_binding_reports_continuous_action_value",
    "consumed_gamepad_axis_does_not_activate_gameplay_action",
    "gamepad_axis_action_reports_deadzone_transition_edges",
    "input_config_builds_action_evaluator_from_serialized_action_map",
    "input_action_manager_resolves_from_runtime_module_descriptor",
    "gamepad_disconnect_clears_held_state_without_panic",
    "gamepad_host_bridge_uses_runtime_gamepad_abi_constructors",
    "input_recording_captures_drainable_event_records_by_frame",
    "input_replay_restores_frame_snapshots_in_recorded_order",
    "cursor_host_requests_are_frame_local_and_drainable",
    "action_evaluator_indexes_10_100_1000_and_10000_bindings_once",
    "action_evaluator_indexes_axis_frame_sources_once_for_10_100_1000_and_10000_bindings",
    "action_evaluator_records_generation_builds_and_distinct_projected_actions",
    "action_evaluator_reuses_workspace_after_axis_warmup",
    "action_evaluator_reuses_consumed_button_index_at_10000_bindings",
    "action_evaluator_reuses_consumed_axis_index_at_10000_bindings",
];

#[test]
fn runtime_12_input_stack_behavior_anchors_remain_visible() {
    assert_eq!(EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS.len(), 21);

    let behavior_test_sources = [
        include_str!("../../../../input/tests/input_manager.rs"),
        include_str!("../../../../input/tests/input_manager/frame_state.rs"),
        include_str!("../../../../input/tests/input_manager/host_requests.rs"),
        include_str!("../../../../input/tests/action_mapping.rs"),
        include_str!("../../../../input/tests/action_axis_transitions.rs"),
        include_str!("../../../../input/tests/gamepad_bridge.rs"),
        include_str!("../../../../input/tests/recording_replay.rs"),
    ];
    for behavior_anchor in EXPECTED_RUNTIME_12_BEHAVIOR_TEST_ANCHORS {
        assert!(
            behavior_test_sources
                .iter()
                .any(|source| source.contains(behavior_anchor)),
            "Runtime 12 behavior test anchor `{behavior_anchor}` should stay visible to input_stack_boundary"
        );
    }
}

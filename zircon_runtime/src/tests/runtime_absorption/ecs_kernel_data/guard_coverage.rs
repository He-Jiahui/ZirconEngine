use super::support::assert_source_anchors;

pub(super) const EXPECTED_RUNTIME_08_BEHAVIOR_TEST_ANCHORS: &[&str] = &[
    "despawned_entity_handle_is_rejected_by_world_access",
    "entity_id_reuse_does_not_alias_previous_generation_handle",
    "stable_entity_location_survives_archetype_move_and_invalidates_on_despawn",
    "component_removal_emits_removal_record_in_same_frame",
    "lifecycle_observer_fires_immediately_during_component_mutation",
    "entity_event_observer_only_fires_for_target_entity",
    "observer_remove_during_dispatch_does_not_skip_or_double_fire",
    "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
    "deferred_command_success_report_counts_applied_commands_without_errors",
    "events_require_explicit_update_and_keep_next_queue_hidden",
    "first_stage_updates_all_registered_event_channels",
    "clear_events_prunes_current_and_next_event_queues",
    "messages_are_retained_until_explicit_clear_independent_of_event_updates",
    "event_and_message_clear_boundaries_do_not_cross_channels",
    "change_tick_comparison_survives_wraparound",
    "tick_window_clamps_stale_ticks",
];

pub(super) fn assert_runtime_08_guard_and_behavior_anchors() {
    assert_source_anchors(
        "Runtime 08 guard/test",
        &[
            include_str!("../../../scene/tests/ecs_identity_storage.rs"),
            include_str!("../../../scene/tests/ecs_observers_messages.rs"),
            include_str!("../../../scene/tests/ecs_commands.rs"),
            include_str!("../../../scene/tests/ecs_events_messages.rs"),
            include_str!("../../../scene/tests/ecs_change_detection.rs"),
            include_str!("../../../scene/tests/component_structure/runtime_08_owner_tree.rs"),
            include_str!("../ecs_kernel_data.rs"),
            include_str!("inventory.rs"),
            include_str!("guard_coverage.rs"),
            include_str!("identity_storage.rs"),
            include_str!("runtime_flow.rs"),
            include_str!("docs.rs"),
            include_str!("component_storage.rs"),
        ],
        &[
            "despawned_entity_handle_is_rejected_by_world_access",
            "entity_id_reuse_does_not_alias_previous_generation_handle",
            "stable_entity_location_survives_archetype_move_and_invalidates_on_despawn",
            "component_removal_emits_removal_record_in_same_frame",
            "lifecycle_observer_fires_immediately_during_component_mutation",
            "entity_event_observer_only_fires_for_target_entity",
            "observer_remove_during_dispatch_does_not_skip_or_double_fire",
            "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
            "deferred_command_success_report_counts_applied_commands_without_errors",
            "events_require_explicit_update_and_keep_next_queue_hidden",
            "first_stage_updates_all_registered_event_channels",
            "clear_events_prunes_current_and_next_event_queues",
            "messages_are_retained_until_explicit_clear_independent_of_event_updates",
            "event_and_message_clear_boundaries_do_not_cross_channels",
            "change_tick_comparison_survives_wraparound",
            "tick_window_clamps_stale_ticks",
            "runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover",
            "runtime_08_ecs_change_detection_owner_tree_stays_folder_backed_after_cutover",
            "runtime_08_ecs_root_leaf_owners_stay_explicit_after_data_cutover",
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
        ],
    );
    assert_source_anchors(
        "Runtime 08 behavior test",
        &[
            include_str!("../../../scene/tests/ecs_identity_storage.rs"),
            include_str!("../../../scene/tests/ecs_observers_messages.rs"),
            include_str!("../../../scene/tests/ecs_commands.rs"),
            include_str!("../../../scene/tests/ecs_events_messages.rs"),
            include_str!("../../../scene/tests/ecs_change_detection.rs"),
        ],
        EXPECTED_RUNTIME_08_BEHAVIOR_TEST_ANCHORS,
    );
}

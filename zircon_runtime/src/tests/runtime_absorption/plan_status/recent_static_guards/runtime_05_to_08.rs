use super::super::support::assert_contains_all;
use super::document_sources::RecentStaticGuardSources;

pub(super) fn assert_runtime_05_to_08_anchors(sources: &RecentStaticGuardSources) {
    let runtime_05_anchors = [
        "runtime_05_closeout_status_records_completed_scene_cargo_gate",
        "runtime_05_scene_1642_structure_1304_review_298_pmrem_parity_passed_closeout_acceptance_complete",
        "1642 passed / 0 failed / 5 ignored",
        "runtime_02_generated_status_guard_present = true",
        "runtime_07_owner_budget_status_guard_present = true",
        "large_file_hotspot_count = 42",
        "runtime-framework-render=4",
        "runtime-other=15",
    ];
    let runtime_06_anchors = [
        "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
        "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        "native_plugin_public_surface",
        "root_reexport_count = 0",
        "native_namespace_reexport_count = 68",
        "native loader test files 4/4",
        "native test namespace import files 3/3",
        "plugin_surface_lifecycle_boundary",
        "fallback lifecycle failure tests 4/4",
        "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
        "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
    ];
    let runtime_07_plan_anchors = [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "frame_spans_static_passed_trace_pending",
        "runtime_frame_schedule_stage.<SystemStage>",
        "SceneScheduleRunner",
    ];
    let runtime_07_doc_anchors = [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "runtime_frame_schedule_stage",
        "SceneScheduleRunner",
        "stage-level span",
    ];
    let runtime_08_anchors = [
        "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
        "despawned_entity_handle_is_rejected_by_world_access",
        "lifecycle_observer_fires_immediately_during_component_mutation",
        "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
        "events_require_explicit_update_and_keep_next_queue_hidden",
        "change_tick_comparison_survives_wraparound",
    ];

    assert_contains_all("Runtime 05 subplan", &sources.archives, &runtime_05_anchors);
    assert_contains_all("Runtime 06 subplan", &sources.archives, &runtime_06_anchors);
    assert_contains_all(
        "Runtime 06 native plugin doc",
        sources.runtime_06_native_doc,
        &runtime_06_anchors,
    );
    assert_contains_all(
        "Runtime 06 interface convergence doc",
        sources.runtime_06_interface_doc,
        &runtime_06_anchors[..3],
    );
    assert_contains_all(
        "Runtime 07 subplan",
        &sources.archives,
        &runtime_07_plan_anchors,
    );
    assert_contains_all(
        "Runtime 07 mirror doc",
        sources.runtime_07_doc,
        &runtime_07_doc_anchors,
    );
    assert_contains_all("Runtime 08 subplan", &sources.archives, &runtime_08_anchors);
    assert_contains_all(
        "Runtime 08 ECS doc",
        sources.runtime_08_doc,
        &runtime_08_anchors,
    );
}

pub(super) fn assert_runtime_01_guard_anchors() {
    let tech_stack_guard = include_str!("../../extensions/tech_stack_dependency_guard.rs");
    let tech_stack_mirror_guard = include_str!("mirror_docs.rs");
    let cargo_gate_guard = include_str!("../plan_status/cargo_gates/early/runtime_01.rs");
    let recent_static_guard = include_str!("../plan_status/recent_static_guards.rs");
    for guard_anchor in [
        "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
        "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
        "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
        "interface_and_editor_dependency_boundaries_stay_documented_and_guarded",
        "removed_or_editor_only_dependencies_do_not_silently_enter_runtime_stack",
        "export_archive_policy_allows_zip_only_for_archive_materializer",
        "physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned",
        "editor_only_dependency_candidates_have_editor_backlog_owner",
        "fontdue_editor_retained_host_dependency_has_migration_owner",
        "complex_text_backends_can_only_enter_through_ui_text_shaper",
        "runtime_text_doc_records_three_layer_stack_and_cross_reference",
        "runtime_01_tech_stack_cargo_gate_records_completed_dependency_validation",
        "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
    ] {
        assert!(
            tech_stack_guard.contains(guard_anchor)
                || tech_stack_mirror_guard.contains(guard_anchor)
                || cargo_gate_guard.contains(guard_anchor)
                || recent_static_guard.contains(guard_anchor),
            "Runtime 01 guard anchor `{guard_anchor}` should stay visible to tech_stack_boundary"
        );
    }
}

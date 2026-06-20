use std::path::Path;

const EXPECTED_RUNTIME_01_MANIFESTS: &[&str] = &[
    "../Cargo.toml",
    "Cargo.toml",
    "../zircon_runtime_interface/Cargo.toml",
    "../zircon_editor/Cargo.toml",
    "../zircon_plugins/physics/runtime/Cargo.toml",
];

#[test]
fn runtime_01_tech_stack_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_01_MANIFESTS.len(), 5);

    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for manifest in EXPECTED_RUNTIME_01_MANIFESTS {
        let path = runtime_root.join(manifest);
        assert!(
            path.exists(),
            "Runtime 01 audited manifest `{}` is missing; update tech_stack_boundary before changing dependency ownership",
            path.display()
        );
    }

    let tech_stack_guard = include_str!("../extensions/tech_stack_dependency_guard.rs");
    let tech_stack_mirror_guard = include_str!("tech_stack.rs");
    let cargo_gate_guard = include_str!("plan_status/cargo_gates/early.rs");
    let recent_static_guard = include_str!("plan_status/recent_static_guards.rs");
    for guard_anchor in [
        "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
        "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
        "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
        "interface_and_editor_dependency_boundaries_stay_documented_and_guarded",
        "removed_or_editor_only_dependencies_do_not_silently_enter_runtime_stack",
        "export_archive_policy_allows_zip_only_for_archive_materializer",
        "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
        "editor_only_dependency_candidates_have_editor_backlog_owner",
        "fontdue_editor_retained_host_dependency_has_migration_owner",
        "complex_text_backends_can_only_enter_through_ui_text_shaper",
        "runtime_text_doc_records_three_layer_stack_and_cross_reference",
        "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
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

    let text_shaper_tests = include_str!("../../ui/tests/text_shaper.rs");
    let physics_contract_mod = include_str!(
        "../../../../zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs"
    );
    let physics_contract_step = include_str!(
        "../../../../zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/step.rs"
    );
    for behavior_test_anchor in [
        "heuristic_text_shaper_matches_public_layout_entrypoint",
        "text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land",
        "empty_jolt_feature_slot_reports_unavailable_not_ready",
        "unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick",
    ] {
        assert!(
            text_shaper_tests.contains(behavior_test_anchor)
                || physics_contract_mod.contains(behavior_test_anchor)
                || physics_contract_step.contains(behavior_test_anchor),
            "Runtime 01 behavior test anchor `{behavior_test_anchor}` should stay visible to tech_stack_boundary"
        );
    }

    let mirror_docs = [
        (
            "runtime tech-stack doc",
            include_str!("../../../../docs/engine-architecture/runtime-tech-stack.md"),
        ),
        (
            "Runtime 01 plan",
            include_str!(
                "../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "tech_stack_boundary",
            "expected_manifest_count = 5",
            "expected_non_dependency_count = 5",
            "zip_dependency_count = 1",
            "expected_zip_dependency_count = 1",
            "zip_dependency_violations = []",
            "tech_stack_guard_count = 12",
            "editor_only_candidate_count = 3",
            "behavior_test_anchor_count = 4",
            "missing_behavior_test_anchors = []",
            "jolt_feature_slot_count = 2",
            "declared_removed_dependencies = []",
            "rapier_or_avian_dependencies = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 01 tech-stack audit anchor `{required_anchor}`"
            );
        }
    }
}

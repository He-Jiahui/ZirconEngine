#[test]
fn runtime_01_tech_stack_cargo_gate_records_completed_dependency_validation() {
    let runtime_01_plan = runtime_plan_source_with_archive("01", include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
    ));
    let runtime_01_plan = runtime_01_plan.as_str();
    let runtime_index = runtime_index_with_numbered_archives(include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/index.md"
    ));
    let runtime_index = runtime_index.as_str();
    let tech_stack =
        include_str!("../../../../../../../docs/engine-architecture/runtime-tech-stack.md");
    let text_doc = include_str!("../../../../../../../docs/zircon_runtime/ui/text.md");
    let physics_options =
        include_str!("../../../../../../../docs/zircon_plugins/physics-plugin-options.md");
    let editor_backlog = include_str!(
        "../../../../../../../docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md"
    );
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_01_plan),
        Some("completed"),
        "Runtime 01 should be complete after every declared dependency gate closes"
    );

    assert_contains_all(
        "Runtime 01 completion evidence",
        runtime_01_plan,
        &[
            "runtime_01_all_declared_cargo_gates_passed_completed",
            "`tech_stack`: 14 passed / 0 failed",
            "`extensions`: 443 passed / 0 failed",
            "`text_shaper`: 7 passed / 0 failed",
            "`export_build_plan`: 67 passed / 0 failed",
            "unit 10/10 plus runtime contract integration 33/33",
        ],
    );

    assert_contains_all(
        "Runtime 01 validation gate commands",
        runtime_01_plan,
        &[
            "cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib extensions --locked",
            "cargo test -p zircon_runtime --lib text_shaper --locked -- --nocapture",
            "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked",
            "runtime_01_tech_stack_cargo_gate_records_completed_dependency_validation",
            "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
            "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
            "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
            "runtime_text_doc_records_three_layer_stack_and_cross_reference",
            "complex_text_backends_can_only_enter_through_ui_text_shaper",
            "fontdue_editor_retained_host_dependency_has_migration_owner",
            "physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned",
            "export_archive_policy_allows_zip_only_for_archive_materializer",
            "editor_only_dependency_candidates_have_editor_backlog_owner",
        ],
    );

    assert!(
        runtime_index.lines().any(|line| {
            line.contains("01-tech-stack-and-dependency-governance.md")
                && line.contains("completed")
                && line.contains("五项")
                && line.contains("Cargo gate 已闭合")
        }),
        "Runtime 01 index should expose the current completed five-gate row"
    );

    assert_contains_all(
        "Runtime index P10 archived evidence",
        runtime_index,
        &[
            "physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned",
            "runtime_01_tech_stack_cargo_gate_records_completed_dependency_validation",
            "tech_stack/text_shaper/plugin physics Cargo gates",
        ],
    );

    assert_contains_all(
        "Runtime tech-stack authority",
        tech_stack,
        &[
            "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
            "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
            "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
            "export_archive_policy_allows_zip_only_for_archive_materializer",
            "editor_only_dependency_candidates_have_editor_backlog_owner",
        ],
    );
    assert_contains_all(
        "Runtime UI text doc",
        text_doc,
        &[
            "Backend Responsibility Matrix",
            "runtime_text_doc_records_three_layer_stack_and_cross_reference",
            "text_shaper_stack_uses_shared_text_service_for_font_backends",
        ],
    );
    assert_contains_all(
        "Runtime physics option doc",
        physics_options,
        &[
            "selected native backend",
            "backend-jolt",
            "physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned",
        ],
    );
    assert_contains_all(
        "Editor-only dependency backlog",
        editor_backlog,
        &[
            "editor_only_dependency_candidates_have_editor_backlog_owner",
            "fontdue_editor_retained_host_dependency_has_migration_owner",
            "rfd",
            "arboard",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 01 gate",
        review,
        &[
            "Runtime 01 Tech Stack Guard",
            "Runtime 01 as `completed`",
            "`tech_stack`, `extensions`, `text_shaper`, `export_build_plan`",
            "Physics plugin Cargo gates closed",
        ],
    );
}

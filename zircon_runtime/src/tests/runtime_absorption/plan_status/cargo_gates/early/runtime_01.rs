#[test]
fn runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation() {
    let runtime_01_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
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
        Some("in_progress"),
        "Runtime 01 should stay in progress until tech_stack/text_shaper/plugin validation closes"
    );

    for row_name in [
        "1.1 选型文档",
        "1.2 winit/notify 策略",
        "1.3 zr_vm 治理决策",
        "1.4 依赖守卫测试",
        "2.1 三层职责矩阵",
        "2.2 cosmic-text 决策",
        "2.3 fontdue 裁决",
        "3.1 物理选型 spike",
        "3.2 导出归档决策",
        "3.3 rfd/arboard 裁决",
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_01_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 01 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 01 pending status row", row, &["Cargo", "待"]);
    }

    assert_contains_all(
        "Runtime 01 validation gate commands",
        runtime_01_plan,
        &[
            "cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib extensions --locked",
            "cargo test -p zircon_runtime --lib text_shaper --locked -- --nocapture",
            "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
            "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
            "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
            "runtime_text_doc_records_three_layer_stack_and_cross_reference",
            "complex_text_backends_can_only_enter_through_ui_text_shaper",
            "fontdue_editor_retained_host_dependency_has_migration_owner",
            "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
            "export_archive_policy_allows_zip_only_for_archive_materializer",
            "editor_only_dependency_candidates_have_editor_backlog_owner",
        ],
    );

    let runtime_01_index_row =
        runtime_index_row_for(runtime_index, "01-tech-stack-and-dependency-governance.md");
    assert_contains_all(
        "Runtime 01 index row",
        runtime_01_index_row,
        &[
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "tech_stack/text_shaper/plugin physics Cargo gates",
            "Cargo 待 active lanes 清空",
        ],
    );

    let runtime_01_problem_row =
        runtime_index_problem_row_for(runtime_index, "P10", "tech-stack completeness");
    assert_contains_all(
        "Runtime index P10 row",
        runtime_01_problem_row,
        &[
            "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
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
            "only executable V1 backend",
            "Jolt native backend",
            "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
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
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "tech_stack/text_shaper/plugin physics",
        ],
    );
}

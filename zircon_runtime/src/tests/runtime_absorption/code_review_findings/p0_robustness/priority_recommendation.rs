#[test]
fn review_priority_recommendation_tracks_current_remaining_work() {
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let priority_section = review_findings
        .split("## 5. 优先级建议")
        .nth(1)
        .and_then(|tail| tail.split("## 6. 审查和修复记录").next())
        .expect("review findings should keep priority recommendation section");

    for required in [
        "当前最高优先剩余项",
        "F1/F2/F4 已有闭合守卫",
        "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        "review_f2_scene_eventbus_locks_recover_after_poison",
        "review_f4_render_submit_capability_gaps_return_typed_errors",
        "F3 仍需 Runtime 07 full FPS/profiling/full gate",
        "render_camera_loop_source_payload_slot_owned_static_passed_cargo_deferred",
        "D-S7 静态插件 manifest 生成/parity 已由 `ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred` 闭合",
        "36/37 generated manifest",
        "plugins_12_static_plugin_manifest_is_generated",
        "plugins_12_manifest_schema_uniform_audit_report_is_clean",
        "plugins_12_feature_enabled_runtime_descriptor_manifest_parity",
        "native_dynamic_fixture importer 自描述已闭合",
        "D-S8/D3 native fixture SDK macro/manifest 单源已闭合",
        "D8 runtime registration builder 原始证据路径已由 `d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred` 闭合",
        "D13 runtime export macro/selection helper 已由 `d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred` 闭合",
        "D13 importer runtime manifest builder 已由 `d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred` 闭合",
        "D13 importer manifest parity guard 已由 `d13_importer_manifest_parity_guard_static_passed_cargo_deferred` 闭合",
        "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
        "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
        "review_priority_recommendation_tracks_current_remaining_work",
    ] {
        assert!(
            priority_section.contains(required),
            "priority recommendation should record current remaining-work anchor `{required}`"
        );
    }
    for stale_priority in [
        "最高优先两项是安全/崩溃类：**F1",
        "D-S8（native 零 SDK）",
        "D13 importer targets/platforms/module/dist-module builder/parity 系列收编",
        "D13 importer manifest parity guard 仍需",
        "D8 `builder.module(desc).system(sys).register(registry)` 封装三步顺序",
        "DX 侧剩余高 ROI 是 **D-S7 静态插件 manifest 生成/parity**",
        "后续 DX 高 ROI 回到 D-S7 静态插件 manifest 生成/parity",
        "应在 Plugins 12 M1/M2 优先落地",
    ] {
        assert!(
            !priority_section.contains(stale_priority),
            "priority recommendation should not keep stale unclosed wording `{stale_priority}`"
        );
    }

    let ds7_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D-S7 |"))
        .expect("review findings should keep D-S7 top-table row");
    for required in [
        "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred",
        "plugins_12_static_manifest_generated_marker_static_passed_cargo_timeout",
        "plugins_12_missing_importer_manifests_guard_passed",
        "plugins_12_manifest_schema_uniform_supported_platforms_guard_passed",
        "plugins_12_manifest_structure_audit_guard_passed",
        "plugins_12_feature_descriptor_parity_guard_passed",
        "plugins_12_static_plugin_manifest_is_generated",
        "plugins_12_manifest_schema_uniform_audit_report_is_clean",
        "plugins_12_feature_enabled_runtime_descriptor_manifest_parity",
        "expected_manifest_count = 37",
        "manifest_count = 37",
        "generated_manifest_count = 36",
        "hand_written_native_manifest_count = 1",
        "manifest_schema_violations = 0",
        "generated_manifest_header_violations = 0",
        "m1_gate_status = classified-and-clear",
    ] {
        assert!(
            ds7_row.contains(required),
            "D-S7 top-table row should record generated/parity closure anchor `{required}`"
        );
    }
    assert!(
        review_findings.contains("D-S7 static plugin manifest generation/parity review sync")
            && review_findings.contains(
                "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred"
            ),
        "review findings should record the D-S7 generation/parity sync completion row"
    );
    let d7_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D7 |"))
        .expect("review findings should keep D7 top-table row");
    for required in [
        "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred",
        "d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred",
        "zircon_plugins/Cargo.lock",
        "core_workspace_dependency_status = core-workspace-deps-clean",
        "core_workspace_dependency_count = 117",
        "core_workspace_dependency_violation_count = 0",
        "cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --offline",
        "`zircon_runtime = { workspace = true }`",
        "`zircon_editor = { workspace = true }`",
        "`zircon_runtime_interface = { workspace = true }`",
        "插件间 path 依赖仍归后续切片",
    ] {
        assert!(
            d7_row.contains(required),
            "D7 top-table row should record core workspace dependency anchor `{required}`"
        );
    }
    assert!(
        !d7_row.contains("zircon_plugins/Cargo.toml` 无 `[workspace.dependencies]"),
        "D7 top-table row should not keep stale no-workspace-dependencies wording"
    );
    assert!(
        d7_row.ends_with("| M2 / closed |"),
        "D7 top-table row should mark core workspace dependency inheritance closed"
    );
    assert!(
        review_findings.contains("D7 core workspace dependency inheritance guard")
            && review_findings.contains(
                "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred"
            ),
        "review findings should record the D7 core workspace dependency completion row"
    );
    assert!(
        review_findings.contains("D7 core workspace dependency top-row closed status sync")
            && review_findings.contains(
                "d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred"
            ),
        "review findings should record the D7 top-row closed status sync completion row"
    );
    let d8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D8 |"))
        .expect("review findings should keep D8 top-table row");
    for required in [
        "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
        "animation/physics/net 原始证据路径已收敛",
        "RuntimePluginRegistrationBuilder",
        "RuntimePluginModuleRegistration::event",
        "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
        "runtime_registration_builder_violation_count = 0",
        "m3_t2_runtime_registration_builder_status = runtime-registration-builder-clean",
    ] {
        assert!(
            d8_row.contains(required),
            "D8 top-table row should record runtime registration builder anchor `{required}`"
        );
    }
    assert!(
        !d8_row.contains("三步样板每插件手抄"),
        "D8 top-table row should not keep stale hand-written registration wording"
    );
    assert!(
        review_findings.contains("D8 runtime registration builder original evidence paths")
            && review_findings.contains(
                "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred"
            ),
        "review findings should record the D8 registration builder completion row"
    );
    assert!(
        review_findings.contains("P0/DX priority recommendation current remaining-work sync")
            && review_findings
                .contains("review_priority_recommendation_tracks_current_remaining_work"),
        "review findings should record the priority recommendation sync completion row"
    );
    assert!(
        review_findings.contains("P0/DX priority recommendation D13 manifest parity sync")
            && review_findings.contains(
                "review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred"
            ),
        "review findings should record the D13 parity priority recommendation sync row"
    );
}

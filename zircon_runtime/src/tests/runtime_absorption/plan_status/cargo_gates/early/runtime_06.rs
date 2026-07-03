#[test]
fn runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation() {
    let runtime_06_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let native_boundary_doc =
        include_str!("../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let runtime_interface_doc = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-interface-convergence.md"
    );
    let runtime_05_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_06_plan),
        Some("in_progress"),
        "Runtime 06 should stay in progress until script VM and plugin/native validation closes"
    );

    for (row_name, required_anchors) in [
        (
            "1.1 空参数修复",
            &[
                "代码完成，runtime Cargo 待验证",
                "call_module_export_accepts_empty_argument_slice",
                "sentinel pointer",
            ][..],
        ),
        (
            "1.2 失败路径测试",
            &[
                "code_static_passed_real_backend_pending",
                "fallback lifecycle failure tests 4/4",
                "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
                "300s 编译超时",
                "runtime real-backend",
            ][..],
        ),
        (
            "2.1 native 收口",
            &[
                "code_static_passed_cargo_pending",
                "plugin::native",
                "root_reexport_count = 0",
                "native_namespace_reexport_count = 64",
            ][..],
        ),
        (
            "2.2 测试/文档迁移",
            &[
                "code_static_passed_cargo_pending",
                "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace",
                "native loader test files 4/4",
                "native test namespace import files 3/3",
                "native test root import leaks 0/0",
            ][..],
        ),
        (
            "3.1 V1/V2 处置",
            &[
                "code_static_passed_cargo_pending",
                "V3-only",
                "unknown ABI rejection",
                "native_loader_v1_v2_file_count = 0",
                "plugin_v1_v2_usage_files = 0",
            ][..],
        ),
        (
            "3.2 回滚失败注入",
            &[
                "code_static_passed_cargo_pending",
                "hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance",
                "hot_reload_state_restore_failure_rolls_back_and_reports",
                "hot reload failure injection",
            ][..],
        ),
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_06_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 06 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 06 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 06 validation gate commands",
        runtime_06_plan,
        &[
            "cargo test -p zircon_runtime --lib script::vm --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib vampire_project_session --features zr-vm-real-backend --locked -- --nocapture --test-threads=1",
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib plugin --locked -- --nocapture",
            "cargo test -p zircon_app --locked",
            "cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked",
            "cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture",
            "cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
        ],
    );

    let runtime_06_index_row =
        runtime_index_row_for(runtime_index, "06-plugin-surface-and-lifecycle.md");
    assert_contains_all(
        "Runtime 06 index row",
        runtime_06_index_row,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
            "fallback lifecycle failure tests 4/4",
            "Release ZrVM focused",
            "完整 `vampire_project_session` 组",
        ],
    );

    let runtime_06_problem_row =
        runtime_index_problem_row_for(runtime_index, "P4", "plugin surface");
    assert_contains_all(
        "Runtime index P4 row",
        runtime_06_problem_row,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "native_plugin_public_surface.m4_gate_status=classified-and-clear",
            "root_reexport_count = 0",
            "native_namespace_reexport_count = 64",
        ],
    );

    assert_contains_all(
        "Native plugin boundary doc",
        native_boundary_doc,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "m4_gate_status",
            "classified-and-clear",
            "root_reexport_count = 0",
            "native_namespace_reexport_count = 64",
            "native loader test files 4/4",
            "native test namespace import files 3/3",
            "native test root import leaks 0/0",
            "fallback lifecycle failure tests 4/4",
        ],
    );

    assert_contains_all(
        "Runtime interface convergence doc",
        runtime_interface_doc,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "native_plugin_public_surface",
            "classified-and-clear",
            "native namespace re-export 64/64",
            "native test namespace import files 3/3",
            "fallback lifecycle failure tests 4/4",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "Runtime 06 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 06 gate",
        review,
        &[
            "Runtime 06 Plugin Surface Lifecycle Gate",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
            "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        ],
    );
}

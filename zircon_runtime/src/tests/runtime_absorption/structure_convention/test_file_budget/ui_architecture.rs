use super::*;

#[test]
fn runtime_15_ui_architecture_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/ui_architecture.rs");
    let architecture_boundaries =
        read_runtime_src("tests/runtime_absorption/ui_architecture/architecture_boundaries.rs");
    let legacy_renames =
        read_runtime_src("tests/runtime_absorption/ui_architecture/legacy_renames.rs");
    let mirror_docs = read_runtime_src("tests/runtime_absorption/ui_architecture/mirror_docs.rs");

    assert_contains_all(
        "UI architecture parent mounts folder-backed children",
        &parent,
        &[
            "mod architecture_boundaries;",
            "mod legacy_renames;",
            "mod mirror_docs;",
            "fn repo_root()",
            "fn top_level_entry_names(",
            "fn rust_files_under(",
            "fn production_ui_file(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "tests/runtime_absorption/ui_architecture.rs should only mount child owners and shared helpers"
    );
    for moved_test in [
        "runtime_09_ui_architecture_doc_records_current_boundaries",
        "runtime_09_taffy_layout_pass_order_uses_bridge_authority",
        "runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt",
        "runtime_09_ui_input_events_route_through_single_dispatch_authority",
        "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI architecture absorption test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI architecture boundary child owns M0/M2/M3 architecture guards",
        &architecture_boundaries,
        &[
            "fn runtime_09_ui_architecture_doc_records_current_boundaries",
            "fn runtime_09_ui_architecture_baselines_match_current_source_scan",
            "fn runtime_09_v2_verdict_matches_runtime_and_interface_modules",
            "fn runtime_09_taffy_layout_pass_order_uses_bridge_authority",
            "fn runtime_09_virtualization_scroll_boundary_records_invalidation_authority",
            "fn runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority",
        ],
    );
    assert_contains_all(
        "UI architecture legacy rename child owns M1.1/M1.2 debt guards",
        &legacy_renames,
        &[
            "fn runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt",
            "fn runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt",
            "fn runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt",
            "fn runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt",
            "fn runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt",
            "fn runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt",
            "fn runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt",
            "fn runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt",
            "fn runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt",
            "fn runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt",
            "fn runtime_09_ui_input_events_route_through_single_dispatch_authority",
        ],
    );
    assert_contains_all(
        "UI architecture mirror-doc child owns audit guard and reads child guard sources",
        &mirror_docs,
        &[
            "fn runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
            "include_str!(\"architecture_boundaries.rs\")",
            "include_str!(\"legacy_renames.rs\")",
            "include_str!(\"mirror_docs.rs\")",
        ],
    );

    let child_test_total = [
        architecture_boundaries.as_str(),
        legacy_renames.as_str(),
        mirror_docs.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 18,
        "UI architecture children should preserve all 18 Runtime 09 absorption guards"
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/ui_architecture.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/ui_architecture/architecture_boundaries.rs",
            architecture_boundaries.as_str(),
        ),
        (
            "tests/runtime_absorption/ui_architecture/legacy_renames.rs",
            legacy_renames.as_str(),
        ),
        (
            "tests/runtime_absorption/ui_architecture/mirror_docs.rs",
            mirror_docs.as_str(),
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
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let audit_script = read_repo(
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py",
    );
    let status_rows = ui_tests_first_status_row_source();
    assert_contains_all(
        "UI architecture audit script reads folder-backed guard owners",
        &audit_script,
        &[
            "ui_architecture/architecture_boundaries.rs",
            "ui_architecture/legacy_renames.rs",
            "ui_architecture/mirror_docs.rs",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI architecture test folder split",
                "runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred",
                "tests/runtime_absorption/ui_architecture.rs",
                "tests/runtime_absorption/ui_architecture/architecture_boundaries.rs",
                "tests/runtime_absorption/ui_architecture/legacy_renames.rs",
                "runtime_15_ui_architecture_tests_are_folder_backed",
            ],
        );
    }
}

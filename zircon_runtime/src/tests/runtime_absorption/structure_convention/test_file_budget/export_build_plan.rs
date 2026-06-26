use super::*;

#[test]
fn runtime_15_export_build_plan_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/export_build_plan.rs");
    let catalog_projection =
        read_runtime_src("tests/plugin_extensions/export_build_plan/catalog_projection.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let export_build_plan_doc = read_repo("docs/zircon_runtime/plugin/export_build_plan.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "export build plan parent mounts catalog projection child owner",
        &parent,
        &[
            "#[path = \"export_build_plan/catalog_projection.rs\"]",
            "mod catalog_projection;",
        ],
    );

    for moved_test in [
        "fn source_template_preserves_builtin_catalog_target_modes_after_manifest_completion",
        "fn source_template_completes_builtin_catalog_selection_before_projection",
        "fn source_template_links_rendering_default_owner_features",
        "fn library_embed_links_advanced_runtime_render_plugins",
        "fn source_template_with_native_dynamic_merges_native_loader_reports",
    ] {
        assert!(
            !parent.contains(moved_test),
            "export build plan parent should mount the catalog projection child owner instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "catalog projection child owns builtin catalog/render/native projection contracts",
        &catalog_projection,
        &[
            "use super::*;",
            "fn source_template_preserves_builtin_catalog_target_modes_after_manifest_completion",
            "fn source_template_links_rendering_default_owner_features",
            "fn source_template_with_native_dynamic_merges_native_loader_reports",
        ],
    );

    assert_eq!(
        parent.matches("#[test]").count() + catalog_projection.matches("#[test]").count(),
        16,
        "export build plan parent plus split child should preserve the original 16 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/export_build_plan.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/export_build_plan/catalog_projection.rs",
            catalog_projection.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("export build plan doc", export_build_plan_doc.as_str()),
        ("status-output scene/script row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 export build plan test folder split",
                "runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred",
                "tests/plugin_extensions/export_build_plan.rs",
                "tests/plugin_extensions/export_build_plan/catalog_projection.rs",
                "runtime_15_export_build_plan_tests_are_folder_backed",
            ],
        );
    }
}

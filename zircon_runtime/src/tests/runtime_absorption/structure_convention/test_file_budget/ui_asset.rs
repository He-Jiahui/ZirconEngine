use super::*;

#[test]
fn runtime_15_ui_asset_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/asset.rs");
    let component_schema = read_runtime_src("ui/tests/asset/component_schema.rs");
    let document_compiler = read_runtime_src("ui/tests/asset/document_compiler.rs");
    let fixture_migration = read_runtime_src("ui/tests/asset/fixture_migration.rs");
    let loader_validation = read_runtime_src("ui/tests/asset/loader_validation.rs");
    let style_rule_ids = read_runtime_src("ui/tests/asset/style_rule_ids.rs");
    let style_write_apis = read_runtime_src("ui/tests/asset/style_write_apis.rs");

    assert_contains_all(
        "UI asset parent mounts folder-backed children",
        &parent,
        &[
            "mod component_schema;",
            "mod document_compiler;",
            "mod fixture_migration;",
            "mod loader_validation;",
            "mod style_rule_ids;",
            "mod style_write_apis;",
            "const STYLE_WITH_RULE_IDS",
            "const LAYOUT_ASSET_TOML",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/asset.rs should only mount child test owners and shared fixtures"
    );
    for moved_test in [
        "ui_asset_stylesheet_rules_preserve_stable_rule_ids",
        "ui_asset_stylesheet_rule_write_apis_reject_invalid_selectors_atomically",
        "ui_asset_loader_rejects_duplicate_stable_style_rule_ids",
        "ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets",
        "ui_asset_loader_rejects_source_template_documents_without_asset_header",
        "ui_asset_compiler_applies_runtime_component_schema_defaults",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI asset test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI asset style-id child owns stable id contracts",
        &style_rule_ids,
        &[
            "fn ui_asset_stylesheet_rules_preserve_stable_rule_ids",
            "fn ui_asset_stylesheet_rules_can_be_renamed_without_breaking_id_uniqueness",
            "fn ui_asset_stylesheets_can_be_removed_by_stable_id_for_editor_undo",
        ],
    );
    assert_contains_all(
        "UI asset style write child owns write API contracts",
        &style_write_apis,
        &[
            "fn ui_asset_stylesheet_rule_write_apis_reject_invalid_selectors_atomically",
            "fn ui_asset_stylesheets_can_be_replaced_atomically_for_editor_replay",
            "fn ui_asset_style_positions_follow_editor_reorder_operations",
        ],
    );
    assert_contains_all(
        "UI asset loader-validation child owns loader rejection contracts",
        &loader_validation,
        &[
            "fn ui_asset_loader_rejects_duplicate_stable_style_rule_ids",
            "fn ui_asset_loader_rejects_duplicate_stable_stylesheet_ids",
            "fn ui_asset_loader_rejects_invalid_style_rule_selectors",
        ],
    );
    assert_contains_all(
        "UI asset document-compiler child owns import/tree contracts",
        &document_compiler,
        &[
            "fn ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets",
            "fn ui_asset_loader_materializes_recursive_tree_authority_in_memory",
        ],
    );
    assert_contains_all(
        "UI asset fixture-migration child owns migration and rejection contracts",
        &fixture_migration,
        &[
            "fn ui_asset_loader_rejects_source_template_documents_without_asset_header",
            "fn ui_flat_fixture_migration_converts_flat_assets_into_tree_authority_source",
            "fn ui_asset_compiler_is_split_into_folder_backed_pipeline_modules",
        ],
    );
    assert_contains_all(
        "UI asset component-schema child owns runtime schema contracts",
        &component_schema,
        &[
            "fn ui_asset_compiler_applies_runtime_component_schema_defaults",
            "fn ui_asset_compiler_rejects_runtime_component_props_with_wrong_type",
            "fn ui_asset_compiler_preserves_style_attributes_unknown_to_component_schema",
        ],
    );

    let child_test_total = [
        component_schema.as_str(),
        document_compiler.as_str(),
        fixture_migration.as_str(),
        loader_validation.as_str(),
        style_rule_ids.as_str(),
        style_write_apis.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 33,
        "UI asset children should retain the current contract set after retiring source-template conversion"
    );

    for (path, source) in [
        ("ui/tests/asset.rs", parent.as_str()),
        (
            "ui/tests/asset/component_schema.rs",
            component_schema.as_str(),
        ),
        (
            "ui/tests/asset/document_compiler.rs",
            document_compiler.as_str(),
        ),
        (
            "ui/tests/asset/fixture_migration.rs",
            fixture_migration.as_str(),
        ),
        (
            "ui/tests/asset/loader_validation.rs",
            loader_validation.as_str(),
        ),
        ("ui/tests/asset/style_rule_ids.rs", style_rule_ids.as_str()),
        (
            "ui/tests/asset/style_write_apis.rs",
            style_write_apis.as_str(),
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
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI asset test folder split",
                "runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/asset.rs",
                "ui/tests/asset/style_rule_ids.rs",
                "ui/tests/asset/component_schema.rs",
                "runtime_15_ui_asset_tests_are_folder_backed",
            ],
        );
    }
}

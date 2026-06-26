use super::*;

#[test]
fn runtime_15_asset_ui_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/ui.rs");
    let fixture_validation = read_runtime_src("asset/tests/assets/ui/fixture_validation.rs");
    let importer = read_runtime_src("asset/tests/assets/ui/importer.rs");
    let project_manager = read_runtime_src("asset/tests/assets/ui/project_manager.rs");
    let references = read_runtime_src("asset/tests/assets/ui/references.rs");
    let wrappers = read_runtime_src("asset/tests/assets/ui/wrappers.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_asset_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );

    assert_contains_all(
        "asset UI parent test module mounts",
        &parent,
        &[
            "mod fixture_validation;",
            "mod importer;",
            "mod project_manager;",
            "mod references;",
            "mod wrappers;",
            "fn importer_with_first_wave_plugin_fixtures",
            "fn legacy_v2_component_toml",
        ],
    );
    for moved_test in [
        "fn ui_asset_wrappers_parse_and_validate_kind",
        "fn ui_theme_asset_round_trips_toml_and_registers_facade_label",
        "fn ui_icon_asset_round_trips_toml_and_registers_facade_label",
        "fn ui_v2_asset_wrappers_parse_and_validate_kind",
        "fn ui_asset_direct_references_include_collected_resource_dependencies",
        "fn ui_asset_direct_references_deduplicate_imported_and_resource_locators",
        "fn ui_v2_asset_direct_references_include_imports_and_resources",
        "fn importer_decodes_ui_theme_assets_from_theme_toml",
        "fn importer_decodes_ui_layout_widget_and_style_assets_from_ui_toml",
        "fn importer_decodes_ui_icon_assets_from_icon_toml",
        "fn importer_decodes_zui_component_assets_from_zui",
        "fn project_manager_scans_ui_theme_assets_and_restores_theme_payloads",
        "fn project_manager_scans_ui_icon_assets_and_restores_icon_payloads",
        "fn project_manager_scans_ui_assets_and_assigns_ui_asset_kinds",
        "fn project_manager_scans_zui_assets_and_restores_component_payloads",
        "fn fixture_v2_toml_importer_rejects_component_kind_in_favor_of_zui",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/ui.rs should mount child owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/ui.rs should not keep executable tests in the parent module"
    );

    let migrated_child_sources = [
        wrappers.as_str(),
        references.as_str(),
        importer.as_str(),
        project_manager.as_str(),
        fixture_validation.as_str(),
    ];
    assert_eq!(
        migrated_child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        16,
        "asset UI child modules should preserve the original 16 parent tests"
    );

    assert_contains_all(
        "asset UI wrappers child owns asset wrapper contracts",
        &wrappers,
        &[
            "use super::*;",
            "fn ui_asset_wrappers_parse_and_validate_kind",
            "fn ui_theme_asset_round_trips_toml_and_registers_facade_label",
            "fn ui_icon_asset_round_trips_toml_and_registers_facade_label",
            "fn ui_v2_asset_wrappers_parse_and_validate_kind",
        ],
    );
    assert_contains_all(
        "asset UI references child owns dependency collection contracts",
        &references,
        &[
            "use super::*;",
            "fn ui_asset_direct_references_include_collected_resource_dependencies",
            "fn ui_asset_direct_references_deduplicate_imported_and_resource_locators",
            "fn ui_v2_asset_direct_references_include_imports_and_resources",
        ],
    );
    assert_contains_all(
        "asset UI importer child owns direct importer contracts",
        &importer,
        &[
            "use super::*;",
            "fn importer_decodes_ui_theme_assets_from_theme_toml",
            "fn importer_decodes_ui_layout_widget_and_style_assets_from_ui_toml",
            "fn importer_decodes_zui_component_assets_from_zui",
        ],
    );
    assert_contains_all(
        "asset UI project manager child owns project scan contracts",
        &project_manager,
        &[
            "use super::*;",
            "fn project_manager_scans_ui_theme_assets_and_restores_theme_payloads",
            "fn project_manager_scans_ui_icon_assets_and_restores_icon_payloads",
            "fn project_manager_scans_zui_assets_and_restores_component_payloads",
        ],
    );
    assert_contains_all(
        "asset UI fixture validation child owns legacy fixture rejection",
        &fixture_validation,
        &[
            "use super::*;",
            "fn fixture_v2_toml_importer_rejects_component_kind_in_favor_of_zui",
            "legacy_v2_component_toml",
        ],
    );

    for (path, source) in [
        ("asset/tests/assets/ui.rs", parent.as_str()),
        (
            "asset/tests/assets/ui/fixture_validation.rs",
            fixture_validation.as_str(),
        ),
        ("asset/tests/assets/ui/importer.rs", importer.as_str()),
        (
            "asset/tests/assets/ui/project_manager.rs",
            project_manager.as_str(),
        ),
        ("asset/tests/assets/ui/references.rs", references.as_str()),
        ("asset/tests/assets/ui/wrappers.rs", wrappers.as_str()),
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
        ("render asset doc", render_asset_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset UI test folder split",
                "runtime_15_asset_ui_tests_folder_split_static_passed_cargo_deferred",
                "asset/tests/assets/ui.rs",
                "asset/tests/assets/ui/importer.rs",
                "asset/tests/assets/ui/project_manager.rs",
                "runtime_15_asset_ui_tests_are_folder_backed",
            ],
        );
    }
}

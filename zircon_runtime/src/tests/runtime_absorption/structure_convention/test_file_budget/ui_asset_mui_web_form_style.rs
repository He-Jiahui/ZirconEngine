use super::*;

#[test]
fn runtime_15_ui_asset_mui_web_form_style_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/asset_mui_web_form_style.rs");
    let form_controls = read_runtime_src("ui/tests/asset_mui_web_form_style/form_controls.rs");

    assert_contains_all(
        "UI asset MUI web form style parent mounts folder-backed children",
        &parent,
        &[
            "mod form_controls;",
            "const FORM_STYLE_TOML",
            "const FORM_LAYOUT_TOML",
            "fn find_node<",
            "fn assert_classes(",
            "fn assert_not_classes(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/asset_mui_web_form_style.rs should only mount child test owners and shared fixtures"
    );
    assert!(
        !parent.contains("mui_web_form_utility_classes_match_local_material_contracts"),
        "moved UI asset MUI web form style test should not return to the parent"
    );

    assert_contains_all(
        "UI asset MUI web form controls child owns form selector contracts",
        &form_controls,
        &[
            "fn mui_web_form_utility_classes_match_local_material_contracts",
            "ButtonBaseRoot",
            "TextFieldRoot",
            "AutocompleteRoot",
        ],
    );
    assert_eq!(
        form_controls.matches("#[test]").count(),
        1,
        "UI asset MUI web form style child should preserve the comprehensive form-control test"
    );

    for (path, source) in [
        ("ui/tests/asset_mui_web_form_style.rs", parent.as_str()),
        (
            "ui/tests/asset_mui_web_form_style/form_controls.rs",
            form_controls.as_str(),
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
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
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
                "Runtime 15 M3 UI asset MUI web form style test folder split",
                "runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/asset_mui_web_form_style.rs",
                "ui/tests/asset_mui_web_form_style/form_controls.rs",
                "runtime_15_ui_asset_mui_web_form_style_tests_are_folder_backed",
            ],
        );
    }
}

use super::*;

#[test]
fn runtime_15_ui_asset_mui_web_style_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/asset_mui_web_style.rs");
    let data_display = read_runtime_src("ui/tests/asset_mui_web_style/data_display.rs");
    let feedback = read_runtime_src("ui/tests/asset_mui_web_style/feedback.rs");
    let slots_native = read_runtime_src("ui/tests/asset_mui_web_style/slots_native.rs");
    let state_icons = read_runtime_src("ui/tests/asset_mui_web_style/state_icons.rs");
    let surface = read_runtime_src("ui/tests/asset_mui_web_style/surface.rs");

    assert_contains_all(
        "UI asset MUI web style parent mounts folder-backed children",
        &parent,
        &[
            "mod data_display;",
            "mod feedback;",
            "mod slots_native;",
            "mod state_icons;",
            "mod surface;",
            "const MUI_WEB_STYLE_TOML",
            "fn assert_classes(",
            "fn assert_no_classes(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/asset_mui_web_style.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "mui_sx_merges_as_high_priority_style_override_and_state_selectors_match",
        "mui_slot_props_apply_to_root_and_named_slot_children",
        "mui_feedback_utility_classes_match_alert_and_snackbar_selectors",
        "mui_surface_utility_classes_match_paper_card_and_app_bar_selectors",
        "mui_data_display_utility_classes_match_local_mui_selectors",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI asset MUI web style test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI asset MUI web style state/icon child owns state and icon class contracts",
        &state_icons,
        &[
            "fn mui_sx_merges_as_high_priority_style_override_and_state_selectors_match",
            "fn mui_state_classes_match_stylesheet_rules",
            "fn mui_readonly_alias_generates_mui_state_class",
            "fn mui_icon_utility_classes_match_local_mui_selectors",
        ],
    );
    assert_contains_all(
        "UI asset MUI web style slot/native child owns slot and native alias contracts",
        &slots_native,
        &[
            "fn mui_slot_props_apply_to_root_and_named_slot_children",
            "fn mui_native_customization_aliases_match_web_prop_names",
        ],
    );
    assert_contains_all(
        "UI asset MUI web style feedback child owns alert/snackbar contracts",
        &feedback,
        &["fn mui_feedback_utility_classes_match_alert_and_snackbar_selectors"],
    );
    assert_contains_all(
        "UI asset MUI web style surface child owns paper/card/app-bar contracts",
        &surface,
        &["fn mui_surface_utility_classes_match_paper_card_and_app_bar_selectors"],
    );
    assert_contains_all(
        "UI asset MUI web style data-display child owns display selector contracts",
        &data_display,
        &["fn mui_data_display_utility_classes_match_local_mui_selectors"],
    );

    let child_test_total = [
        data_display.as_str(),
        feedback.as_str(),
        slots_native.as_str(),
        state_icons.as_str(),
        surface.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 9,
        "UI asset MUI web style children should preserve all 9 parent tests"
    );

    for (path, source) in [
        ("ui/tests/asset_mui_web_style.rs", parent.as_str()),
        (
            "ui/tests/asset_mui_web_style/data_display.rs",
            data_display.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_style/feedback.rs",
            feedback.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_style/slots_native.rs",
            slots_native.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_style/state_icons.rs",
            state_icons.as_str(),
        ),
        ("ui/tests/asset_mui_web_style/surface.rs", surface.as_str()),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );
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
                "Runtime 15 M3 UI asset MUI web style test folder split",
                "runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/asset_mui_web_style.rs",
                "ui/tests/asset_mui_web_style/state_icons.rs",
                "ui/tests/asset_mui_web_style/data_display.rs",
                "runtime_15_ui_asset_mui_web_style_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI asset MUI web style test folder split",
            "runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset_mui_web_style.rs",
            "ui/tests/asset_mui_web_style/state_icons.rs",
            "runtime_15_ui_asset_mui_web_style_tests_are_folder_backed",
        ],
    );
}

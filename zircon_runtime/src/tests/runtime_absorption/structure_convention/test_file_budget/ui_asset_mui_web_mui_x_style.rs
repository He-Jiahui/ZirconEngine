use super::*;

#[test]
fn runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/asset_mui_web_mui_x_style.rs");
    let agent_chat = read_runtime_src("ui/tests/asset_mui_web_mui_x_style/agent_chat.rs");
    let charts = read_runtime_src("ui/tests/asset_mui_web_mui_x_style/charts.rs");
    let data_grid = read_runtime_src("ui/tests/asset_mui_web_mui_x_style/data_grid.rs");
    let date_time_pickers =
        read_runtime_src("ui/tests/asset_mui_web_mui_x_style/date_time_pickers.rs");
    let tree_view = read_runtime_src("ui/tests/asset_mui_web_mui_x_style/tree_view.rs");

    assert_contains_all(
        "UI asset MUI X web style parent mounts folder-backed children",
        &parent,
        &[
            "mod agent_chat;",
            "mod charts;",
            "mod data_grid;",
            "mod date_time_pickers;",
            "mod tree_view;",
            "const MUI_X_STYLE_TOML",
            "const MUI_X_LAYOUT_TOML",
            "fn find_node<'a>(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/asset_mui_web_mui_x_style.rs should only mount child test owners and shared fixtures"
    );
    for moved_test in [
        "mui_x_utility_classes_match_retained_x_targets",
        "mui_x_data_grid_utility_classes_match_retained_targets",
        "mui_x_tree_view_utility_classes_match_retained_targets",
        "mui_x_date_time_picker_utility_classes_match_retained_targets",
        "mui_x_chart_and_gauge_utility_classes_match_retained_targets",
        "mui_x_agent_chat_utility_classes_match_retained_targets",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI asset MUI X style test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI asset MUI X DataGrid child owns grid contracts",
        &data_grid,
        &[
            "fn mui_x_data_grid_utility_classes_match_retained_targets",
            "DataGridRoot",
            "DataGridEditingColumnsRoot",
        ],
    );
    assert_contains_all(
        "UI asset MUI X TreeView child owns tree contracts",
        &tree_view,
        &[
            "fn mui_x_tree_view_utility_classes_match_retained_targets",
            "TreeViewRoot",
            "TreeViewFeatureFlagsRoot",
        ],
    );
    assert_contains_all(
        "UI asset MUI X Date/Time child owns picker contracts",
        &date_time_pickers,
        &[
            "fn mui_x_date_time_picker_utility_classes_match_retained_targets",
            "DateTimePickersRoot",
            "DateTimePickerPopper",
        ],
    );
    assert_contains_all(
        "UI asset MUI X chart child owns chart/gauge contracts",
        &charts,
        &[
            "fn mui_x_chart_and_gauge_utility_classes_match_retained_targets",
            "LineChartRoot",
            "GaugeTooltip",
        ],
    );
    assert_contains_all(
        "UI asset MUI X agent chat child owns chat contracts",
        &agent_chat,
        &[
            "fn mui_x_agent_chat_utility_classes_match_retained_targets",
            "AgentChatRoot",
            "ChatComposerRoot",
        ],
    );

    let child_test_total = [
        agent_chat.as_str(),
        charts.as_str(),
        data_grid.as_str(),
        date_time_pickers.as_str(),
        tree_view.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 5,
        "UI asset MUI X style children should preserve the split component-family coverage"
    );

    for (path, source) in [
        ("ui/tests/asset_mui_web_mui_x_style.rs", parent.as_str()),
        (
            "ui/tests/asset_mui_web_mui_x_style/agent_chat.rs",
            agent_chat.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_mui_x_style/charts.rs",
            charts.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_mui_x_style/data_grid.rs",
            data_grid.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_mui_x_style/date_time_pickers.rs",
            date_time_pickers.as_str(),
        ),
        (
            "ui/tests/asset_mui_web_mui_x_style/tree_view.rs",
            tree_view.as_str(),
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
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI asset MUI X web style test folder split",
                "runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/asset_mui_web_mui_x_style.rs",
                "ui/tests/asset_mui_web_mui_x_style/data_grid.rs",
                "ui/tests/asset_mui_web_mui_x_style/agent_chat.rs",
                "runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI asset MUI X web style test folder split",
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset_mui_web_mui_x_style.rs",
            "ui/tests/asset_mui_web_mui_x_style/data_grid.rs",
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed",
        ],
    );
}

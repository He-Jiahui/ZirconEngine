use super::*;

#[test]
fn runtime_15_ui_runtime_input_reply_table_pointer_routes_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/runtime_input_reply_routes/table_pointer_routes.rs");
    let resize =
        read_runtime_src("ui/tests/runtime_input_reply_routes/table_pointer_routes/resize.rs");
    let sorting =
        read_runtime_src("ui/tests/runtime_input_reply_routes/table_pointer_routes/sorting.rs");
    let selection =
        read_runtime_src("ui/tests/runtime_input_reply_routes/table_pointer_routes/selection.rs");
    let virtualization = read_runtime_src(
        "ui/tests/runtime_input_reply_routes/table_pointer_routes/virtualization.rs",
    );

    assert_contains_all(
        "table pointer reply-route parent mounts folder-backed children",
        &parent,
        &[
            "mod resize;",
            "mod selection;",
            "mod sorting;",
            "mod virtualization;",
            "fn table_pointer_route_surface_with_virtualization_options(",
            "fn insert_table_sort_header(",
            "fn column_width_payload(",
            "fn assert_table_sort_state(",
        ],
    );
    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count(),
        0,
        "table_pointer_routes.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "table_column_resize_drag_updates_widths_and_emits_value_changed",
        "table_sort_header_click_toggles_direction_and_sorts_rows",
        "table_row_click_selects_row_on_owner",
        "table_scroll_updates_virtual_window_and_emits_visible_range",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved table pointer route test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "table pointer resize child owns column resize tests",
        &resize,
        &[
            "fn table_column_resize_drag_updates_widths_and_emits_value_changed",
            "fn data_grid_column_resize_drag_updates_mui_grid_widths",
            "fn data_grid_disable_column_resize_blocks_default_resize_drag",
        ],
    );
    assert_contains_all(
        "table pointer sorting child owns header sorting tests",
        &sorting,
        &[
            "fn table_sort_header_click_toggles_direction_and_sorts_rows",
            "fn data_grid_server_sort_header_click_updates_sort_model_without_reordering_rows",
        ],
    );
    assert_contains_all(
        "table pointer selection child owns row selection tests",
        &selection,
        &[
            "fn table_row_click_selects_row_on_owner",
            "fn data_grid_row_click_updates_row_selection_model",
            "fn data_grid_disable_row_selection_on_click_blocks_row_selection",
        ],
    );
    assert_contains_all(
        "table pointer virtualization child owns virtual scroll tests",
        &virtualization,
        &[
            "fn table_scroll_updates_virtual_window_and_emits_visible_range",
            "fn data_grid_scroll_updates_mui_virtual_window_aliases",
            "fn data_grid_disable_virtualization_blocks_default_virtual_scroll",
        ],
    );

    let table_test_total = [
        resize.as_str(),
        sorting.as_str(),
        selection.as_str(),
        virtualization.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        table_test_total, 11,
        "table pointer route children should preserve all 11 parent tests"
    );

    for (path, source) in [
        (
            "ui/tests/runtime_input_reply_routes/table_pointer_routes.rs",
            parent.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/table_pointer_routes/resize.rs",
            resize.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/table_pointer_routes/sorting.rs",
            sorting.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/table_pointer_routes/selection.rs",
            selection.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/table_pointer_routes/virtualization.rs",
            virtualization.as_str(),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first.rs",
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
                "Runtime 15 M3 UI runtime input reply table pointer route folder split",
                "runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_input_reply_routes/table_pointer_routes.rs",
                "ui/tests/runtime_input_reply_routes/table_pointer_routes/resize.rs",
                "ui/tests/runtime_input_reply_routes/table_pointer_routes/virtualization.rs",
                "runtime_15_ui_runtime_input_reply_table_pointer_routes_are_folder_backed",
            ],
        );
    }
}

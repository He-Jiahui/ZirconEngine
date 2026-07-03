use super::*;

#[test]
fn runtime_15_ui_material_layout_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/material_layout.rs");
    let asset_icon_roles = read_runtime_src("ui/tests/material_layout/asset_icon_roles.rs");
    let button_icon_metrics = read_runtime_src("ui/tests/material_layout/button_icon_metrics.rs");
    let constraints_children = read_runtime_src("ui/tests/material_layout/constraints_children.rs");
    let field_values = read_runtime_src("ui/tests/material_layout/field_values.rs");
    let row_label_metrics = read_runtime_src("ui/tests/material_layout/row_label_metrics.rs");

    assert_contains_all(
        "UI material-layout parent mounts folder-backed children",
        &parent,
        &[
            "mod asset_icon_roles;",
            "mod button_icon_metrics;",
            "mod constraints_children;",
            "mod field_values;",
            "mod row_label_metrics;",
            "fn measure_material_leaf(",
            "fn render_material_leaf_command(",
            "fn intrinsic_constraints()",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/material_layout.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "material_button_measures_text_plus_layout_padding",
        "material_menu_item_uses_list_row_height_and_horizontal_padding",
        "material_fields_measure_visible_value_placeholder_and_options_text",
        "asset_value_nodes_render_as_image_or_icon_not_text",
        "material_button_respects_authored_fixed_constraints",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI material-layout test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI material-layout button/icon child owns button and icon sizing contracts",
        &button_icon_metrics,
        &[
            "fn material_button_measures_text_plus_layout_padding",
            "fn material_button_long_text_expands_beyond_default_frame_width",
            "fn material_button_with_icon_adds_icon_size_and_spacing",
            "fn material_icon_button_without_visual_icon_keeps_label_accessibility_only",
        ],
    );
    assert_contains_all(
        "UI material-layout row/label child owns row and label sizing contracts",
        &row_label_metrics,
        &[
            "fn material_menu_item_uses_list_row_height_and_horizontal_padding",
            "fn material_tab_uses_control_height_and_text_width_plus_padding",
            "fn material_table_row_uses_list_row_height_and_text_width_plus_padding",
        ],
    );
    assert_contains_all(
        "UI material-layout field-values child owns field value measurement contracts",
        &field_values,
        &[
            "fn material_fields_measure_visible_value_placeholder_and_options_text",
            "fn text_field_placeholder_measures_without_becoming_editable_value",
            "fn material_vector_fields_measure_visible_value_text",
        ],
    );
    assert_contains_all(
        "UI material-layout asset/icon role child owns asset and role metrics contracts",
        &asset_icon_roles,
        &[
            "fn asset_value_nodes_render_as_image_or_icon_not_text",
            "fn icon_button_label_is_accessibility_text_not_rendered_text",
            "fn common_native_material_roles_use_authored_layout_metrics",
        ],
    );
    assert_contains_all(
        "UI material-layout constraints child owns authored constraint contracts",
        &constraints_children,
        &[
            "fn material_button_respects_authored_fixed_constraints",
            "fn material_button_with_child_content_receives_padding_and_minimum_height",
            "fn material_list_field_and_switch_controls_keep_min_height",
        ],
    );

    let child_test_total = [
        asset_icon_roles.as_str(),
        button_icon_metrics.as_str(),
        constraints_children.as_str(),
        field_values.as_str(),
        row_label_metrics.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 23,
        "UI material-layout children should preserve all 23 parent tests"
    );

    for (path, source) in [
        ("ui/tests/material_layout.rs", parent.as_str()),
        (
            "ui/tests/material_layout/asset_icon_roles.rs",
            asset_icon_roles.as_str(),
        ),
        (
            "ui/tests/material_layout/button_icon_metrics.rs",
            button_icon_metrics.as_str(),
        ),
        (
            "ui/tests/material_layout/constraints_children.rs",
            constraints_children.as_str(),
        ),
        (
            "ui/tests/material_layout/field_values.rs",
            field_values.as_str(),
        ),
        (
            "ui/tests/material_layout/row_label_metrics.rs",
            row_label_metrics.as_str(),
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
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI material layout test folder split",
                "runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/material_layout.rs",
                "ui/tests/material_layout/button_icon_metrics.rs",
                "ui/tests/material_layout/field_values.rs",
                "runtime_15_ui_material_layout_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI material layout test folder split",
            "runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/material_layout.rs",
            "ui/tests/material_layout/button_icon_metrics.rs",
            "runtime_15_ui_material_layout_tests_are_folder_backed",
        ],
    );
}

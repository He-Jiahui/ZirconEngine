use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_surface_table_column_helpers_are_child_owner() {
    let parent = read_runtime_src("ui/surface/surface/default_interactions/table/mod.rs");
    let columns = read_runtime_src("ui/surface/surface/default_interactions/table/columns.rs");
    let mutation = read_runtime_src("ui/surface/surface/default_interactions/table/mutation.rs");
    let selection = read_runtime_src("ui/surface/surface/default_interactions/table/selection.rs");
    let virtualization =
        read_runtime_src("ui/surface/surface/default_interactions/table/virtualization.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "table parent keeps pointer flow, mutation entry points, and child owner mounts",
        &parent,
        &[
            "mod columns;",
            "mod mutation;",
            "mod selection;",
            "mod virtualization;",
            "pub(in crate::ui::surface::surface) fn apply_default_table_pointer_action(",
            "fn apply_default_table_column_resize_press(",
            "fn apply_default_table_sort_header_release(",
            "self.apply_table_column_widths_mutation(",
            "columns::encode_table_column_resize_drag(",
            "columns::table_column_width(",
            "columns::is_table_column_resize_handle",
            "columns::is_table_column_sort_header",
        ],
    );
    for moved_owner in [
        "const TABLE_COLUMN_RESIZE_DRAG_PREFIX",
        "const DEFAULT_MIN_COLUMN_WIDTH",
        "const TABLE_COLUMN_FIELD_PROPERTIES",
        "fn table_column_width(",
        "fn table_column_matches(",
        "fn compare_table_row_value(",
        "fn encode_table_column_resize_drag(",
        "fn decode_table_column_resize_drag(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "table/mod.rs should delegate column helper owner `{moved_owner}` to columns.rs"
        );
    }

    assert_contains_all(
        "columns child owns table column metadata, sort, width, and drag-token helpers",
        &columns,
        &[
            "const TABLE_COLUMN_RESIZE_DRAG_PREFIX",
            "const DEFAULT_MIN_COLUMN_WIDTH",
            "const TABLE_COLUMN_FIELD_PROPERTIES",
            "pub(super) fn is_table_column_resize_handle(",
            "pub(super) fn is_table_column_sort_header(",
            "pub(super) fn table_column_width(",
            "pub(super) fn table_min_column_width(",
            "pub(super) fn table_column_matches(",
            "pub(super) fn compare_table_row_value(",
            "pub(super) fn encode_table_column_resize_drag(",
            "pub(super) fn decode_table_column_resize_drag(",
            "fn table_sort_column(",
            "fn toml_column_matches(",
        ],
    );

    assert_contains_all(
        "mutation child owns table property mutation and accepted binding-report flow",
        &mutation,
        &[
            "pub(super) fn apply_table_column_widths_mutation(",
            "pub(super) fn apply_table_columns_width_mutation(",
            "pub(super) fn apply_table_sort_model_mutation(",
            "pub(super) fn apply_table_columns_sort_direction_mutation(",
            "pub(super) fn apply_table_rows_sort_mutation(",
            "pub(super) fn apply_table_mutation(",
            "UiPropertyMutationStatus::Accepted",
        ],
    );

    for (path, source) in [
        (
            "ui/surface/surface/default_interactions/table/mod.rs",
            parent.as_str(),
        ),
        (
            "ui/surface/surface/default_interactions/table/columns.rs",
            columns.as_str(),
        ),
        (
            "ui/surface/surface/default_interactions/table/mutation.rs",
            mutation.as_str(),
        ),
        (
            "ui/surface/surface/default_interactions/table/selection.rs",
            selection.as_str(),
        ),
        (
            "ui/surface/surface/default_interactions/table/virtualization.rs",
            virtualization.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

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
                "Runtime 15 M4 UI surface table column helper owner split",
                "runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred",
                "ui/surface/surface/default_interactions/table/mod.rs",
                "ui/surface/surface/default_interactions/table/columns.rs",
                "runtime_15_ui_surface_table_column_helpers_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI surface table column helper owner split",
            "runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions/table/mod.rs",
            "ui/surface/surface/default_interactions/table/columns.rs",
            "runtime_15_ui_surface_table_column_helpers_are_child_owner",
        ],
    );
}

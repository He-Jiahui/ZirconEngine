use super::*;

#[test]
fn mui_x_data_grid_utility_classes_match_retained_targets() {
    let style = UiAssetLoader::load_toml_str(MUI_X_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_X_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_mui_x_style.ui", style)
        .unwrap();
    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let grid = find_node(root, "DataGridRoot");
    assert_eq!(str_attr(grid, "surface_variant"), Some("data-grid-loading"));
    assert_eq!(
        str_attr(grid, "validation_level"),
        Some("grid-loading-empty")
    );
    assert_classes(
        grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-withBorderColor",
            "MuiDataGrid-root--densityCompact",
            "MuiDataGrid-loading",
            "MuiDataGrid-emptyRows",
            "MuiDataGrid-emptyColumns",
        ],
    );

    let header = find_node(root, "DataGridHeader");
    assert_eq!(str_attr(header, "text_tone"), Some("grid-header"));
    assert_classes(
        header,
        &[
            "MuiDataGrid-header",
            "MuiDataGrid-columnHeaders",
            "header-extra",
        ],
    );

    let row = find_node(root, "DataGridRow");
    assert_eq!(str_attr(row, "validation_level"), Some("grid-row-loading"));
    assert_classes(row, &["MuiDataGrid-row", "MuiDataGrid-row--loading"]);

    let standard_grid = find_node(root, "DataGridStandardRoot");
    assert_eq!(str_attr(standard_grid, "density"), Some("standard"));
    assert_classes(
        standard_grid,
        &["MuiDataGrid-root", "MuiDataGrid-root--densityStandard"],
    );

    let comfortable_grid = find_node(root, "DataGridComfortableRoot");
    assert_eq!(str_attr(comfortable_grid, "density"), Some("comfortable"));
    assert_classes(
        comfortable_grid,
        &["MuiDataGrid-root", "MuiDataGrid-root--densityComfortable"],
    );

    let feature_grid = find_node(root, "DataGridFeatureFlagsRoot");
    assert_eq!(
        str_attr(feature_grid, "text_tone"),
        Some("grid-feature-flags")
    );
    assert_classes(
        feature_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-checkboxSelection",
            "MuiDataGrid-disableColumnMenu",
            "MuiDataGrid-disableRowSelectionOnClick",
        ],
    );

    let server_mode_grid = find_node(root, "DataGridServerModesRoot");
    assert_eq!(
        str_attr(server_mode_grid, "validation_level"),
        Some("grid-server-mode-selection")
    );
    assert_eq!(
        str_attr(server_mode_grid, "text_tone"),
        Some("grid-data-bound")
    );
    assert_classes(
        server_mode_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-sortingModeServer",
            "MuiDataGrid-filterModeServer",
            "MuiDataGrid-rowSelection",
            "MuiDataGrid-hasRows",
            "MuiDataGrid-hasColumns",
        ],
    );

    let controlled_model_grid = find_node(root, "DataGridControlledModelsRoot");
    assert_eq!(
        str_attr(controlled_model_grid, "validation_level"),
        Some("grid-controlled-models")
    );
    assert_classes(
        controlled_model_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-hasSortModel",
            "MuiDataGrid-hasFilterModel",
            "MuiDataGrid-hasPaginationModel",
            "MuiDataGrid-hasQuickFilter",
        ],
    );

    let chrome_slots_grid = find_node(root, "DataGridChromeSlotsRoot");
    assert_eq!(
        str_attr(chrome_slots_grid, "validation_level"),
        Some("grid-chrome-customized")
    );
    assert_classes(
        chrome_slots_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-showToolbar",
            "MuiDataGrid-hideFooterPagination",
            "MuiDataGrid-hideFooterSelectedRowCount",
            "MuiDataGrid-withVerticalBorder",
            "MuiDataGrid-showCellVerticalBorder",
            "MuiDataGrid-rowSpacingBorder",
            "MuiDataGrid-hasScrollbarSize",
            "MuiDataGrid-hasLabel",
        ],
    );

    let column_header = find_node(root, "DataGridColumnHeader");
    assert_eq!(
        str_attr(column_header, "text_tone"),
        Some("grid-column-header-state")
    );
    assert_classes(
        column_header,
        &[
            "MuiDataGrid-columnHeader",
            "MuiDataGrid-columnHeader--sortable",
            "MuiDataGrid-columnHeader--sorted",
            "MuiDataGrid-columnHeader--withRightBorder",
        ],
    );

    let cell = find_node(root, "DataGridCell");
    assert_eq!(str_attr(cell, "text_tone"), Some("grid-cell-state"));
    assert_classes(
        cell,
        &[
            "MuiDataGrid-cell",
            "MuiDataGrid-cell--editable",
            "MuiDataGrid-cell--editing",
            "MuiDataGrid-cell--withRightBorder",
        ],
    );

    let pagination_grid = find_node(root, "DataGridPaginationRoot");
    assert_eq!(
        str_attr(pagination_grid, "validation_level"),
        Some("grid-paginated")
    );
    assert_classes(
        pagination_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-hasPage",
            "MuiDataGrid-hasPageSize",
        ],
    );

    let virtualization_grid = find_node(root, "DataGridVirtualizationRoot");
    assert_eq!(
        str_attr(virtualization_grid, "validation_level"),
        Some("grid-virtualization-config")
    );
    assert_classes(
        virtualization_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-hasRowCount",
            "MuiDataGrid-hasRowHeight",
            "MuiDataGrid-hasOverscan",
            "MuiDataGrid-disableVirtualization",
        ],
    );

    let visible_range_grid = find_node(root, "DataGridVisibleRangeRoot");
    assert_eq!(
        str_attr(visible_range_grid, "validation_level"),
        Some("grid-visible-range")
    );
    assert_classes(
        visible_range_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-hasViewportRange",
            "MuiDataGrid-hasRequestedRange",
            "MuiDataGrid-hasScrollOffset",
        ],
    );

    let editing_columns_grid = find_node(root, "DataGridEditingColumnsRoot");
    assert_eq!(
        str_attr(editing_columns_grid, "validation_level"),
        Some("grid-editing-column-state")
    );
    assert_classes(
        editing_columns_grid,
        &[
            "MuiDataGrid-root",
            "MuiDataGrid-editModeRow",
            "MuiDataGrid-hasCellModes",
            "MuiDataGrid-hasRowModes",
            "MuiDataGrid-hasColumnVisibilityModel",
            "MuiDataGrid-hasPinnedColumns",
        ],
    );
}

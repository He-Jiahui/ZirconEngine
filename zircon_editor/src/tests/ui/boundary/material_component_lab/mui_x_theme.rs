use std::{collections::BTreeSet, fs};

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::support::{editor_asset, MUI_X_PROTOTYPES};

const MUI_X_THEME_SELECTORS: &[&str] = &[
    ".MuiDataGrid-root",
    ".MuiDataGrid-root.MuiDataGrid-loading",
    ".MuiDataGrid-root.MuiDataGrid-autoHeight",
    ".MuiDataGrid-root.MuiDataGrid-root--densityCompact",
    ".MuiDataGrid-root.MuiDataGrid-root--densityStandard",
    ".MuiDataGrid-root.MuiDataGrid-root--densityComfortable",
    ".MuiDataGrid-root.MuiDataGrid-checkboxSelection.MuiDataGrid-disableColumnMenu.MuiDataGrid-disableRowSelectionOnClick",
    ".MuiDataGrid-root.MuiDataGrid-sortingModeServer.MuiDataGrid-filterModeServer.MuiDataGrid-rowSelection",
    ".MuiDataGrid-root.MuiDataGrid-hasSortModel.MuiDataGrid-hasFilterModel.MuiDataGrid-hasPaginationModel.MuiDataGrid-hasQuickFilter",
    ".MuiDataGrid-root.MuiDataGrid-showToolbar.MuiDataGrid-hideFooterPagination.MuiDataGrid-hideFooterSelectedRowCount.MuiDataGrid-withVerticalBorder.MuiDataGrid-showCellVerticalBorder.MuiDataGrid-rowSpacingBorder.MuiDataGrid-hasScrollbarSize.MuiDataGrid-hasLabel",
    ".MuiDataGrid-root.MuiDataGrid-emptyRows.MuiDataGrid-emptyColumns.MuiDataGrid-loading",
    ".MuiDataGrid-root.MuiDataGrid-hasRows.MuiDataGrid-hasColumns.MuiDataGrid-rowSelection",
    ".MuiDataGrid-root.MuiDataGrid-hasPage.MuiDataGrid-hasPageSize",
    ".MuiDataGrid-root.MuiDataGrid-hasRowCount.MuiDataGrid-hasRowHeight.MuiDataGrid-hasOverscan.MuiDataGrid-disableVirtualization",
    ".MuiDataGrid-root.MuiDataGrid-hasViewportRange.MuiDataGrid-hasRequestedRange.MuiDataGrid-hasScrollOffset",
    ".MuiDataGrid-root.MuiDataGrid-editModeRow.MuiDataGrid-hasCellModes.MuiDataGrid-hasRowModes.MuiDataGrid-hasColumnVisibilityModel.MuiDataGrid-hasPinnedColumns",
    ".MuiDataGrid-columnHeaders",
    ".MuiDataGrid-columnHeader",
    ".MuiDataGrid-columnHeader--sortable",
    ".MuiDataGrid-columnHeader--sorted",
    ".MuiDataGrid-columnHeader--withRightBorder",
    ".MuiDataGrid-row",
    ".MuiDataGrid-row.MuiDataGrid-row--loading",
    ".MuiDataGrid-cell",
    ".MuiDataGrid-cell--editable",
    ".MuiDataGrid-cell--editing",
    ".MuiDataGrid-cell--withRightBorder",
    ".material-mui-x-header",
    ".material-mui-x-column-header",
    ".material-mui-x-row",
    ".material-mui-x-cell",
    ".MuiTreeView-root",
    ".MuiTreeView-root.MuiTreeView-multiSelect",
    ".MuiTreeView-root.MuiTreeView-checkboxSelection.MuiTreeView-disabledItemsFocusable.MuiTreeView-editable",
    ".MuiTreeView-root.MuiTreeView-hasExpandedItems.MuiTreeView-hasSelectedItems.MuiTreeView-hasItemIndentation",
    ".material-mui-x-tree-item",
    ".material-mui-x-tree-content",
    ".material-mui-x-tree-label",
    ".material-mui-x-tree-icon",
    ".material-mui-x-tree-checkbox",
    ".MuiTreeItem-root",
    ".MuiTreeItem-root.MuiTreeItem-expanded.MuiTreeItem-selected.MuiTreeItem-editable.MuiTreeItem-disabledItemsFocusable",
    ".MuiTreeItem-content",
    ".MuiTreeItem-content.MuiTreeItem-expanded.MuiTreeItem-selected",
    ".MuiTreeItem-label",
    ".MuiTreeItem-labelInput",
    ".MuiTreeItem-label.MuiTreeItem-labelInput",
    ".MuiTreeItem-iconContainer",
    ".MuiTreeItem-checkbox",
    ".MuiTreeItem-checkbox.MuiTreeItem-checkboxSelection",
    ".MuiTreeItem-expanded",
    ".MuiTreeItem-selected",
    ".MuiTreeItem-editable",
    ".MuiTreeItem-disabledItemsFocusable",
    ".MuiTreeItem-checkboxSelection",
    ".MuiDateTimePicker-root",
    ".MuiPickersLayout-Desktop",
    ".MuiDateTimePicker-root.MuiPickersLayout-Desktop",
    ".MuiDatePicker-root.MuiPickersLayout-Mobile",
    ".MuiTimePicker-root.MuiPickersLayout-Static",
    ".MuiDateTimePicker-root.MuiPickers-readOnly.MuiPickers-ampm.MuiPickers-hasDateBounds.MuiPickers-hasViews",
    ".MuiDateTimePicker-root.MuiPickers-hasValue.MuiPickers-hasView.MuiPickers-hasFormat",
    ".material-mui-x-field",
    ".material-mui-x-layout",
    ".material-mui-x-toolbar",
    ".material-mui-x-popper",
    ".MuiChartsSurface-root",
    ".MuiCharts-root",
    ".MuiChartsSurface-root.MuiCharts-loading",
    ".MuiCharts-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
    ".MuiLineChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
    ".MuiBarChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
    ".MuiPieChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
    ".MuiSparkLineChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
    ".MuiGauge-root.MuiChartsSurface-root.MuiGauge-hasValue.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
    ".MuiLineChart-root",
    ".MuiBarChart-root",
    ".MuiPieChart-root",
    ".MuiSparkLineChart-root",
    ".MuiGauge-root",
    ".MuiChartsLegend.MuiChartsLegend-hasSeries.MuiChartsLegend-hasCustomColors",
    ".MuiChartsTooltip.MuiChartsTooltip-loading.MuiChartsTooltip-hasSeries.MuiChartsTooltip-hasAxes.MuiChartsTooltip-hasMargin.MuiChartsTooltip-interactionHover",
    ".MuiChartsTooltip.MuiChartsTooltip-hasValue.MuiChartsTooltip-hasMargin",
    ".material-mui-x-legend",
    ".material-mui-x-tooltip",
    ".MuiAgentChat-root",
    ".MuiAgentChat-root.MuiAgentChat-streaming",
    ".MuiAgentChat-root.MuiAgentChat-error",
    ".MuiAgentChat-root.MuiAgentChat-hasMessages.MuiAgentChat-hasComposerText",
    ".MuiAgentChat-messages.MuiAgentChat-messagesPopulated.MuiAgentChat-messagesStreaming.MuiAgentChat-messagesError",
    ".MuiAgentChat-composer.MuiAgentChat-composerStreaming.MuiAgentChat-composerError.MuiAgentChat-composerHasText",
    ".MuiChatMessage-root.MuiChatMessage-populated",
    ".MuiChatConversationList-root",
    ".MuiChatConversationList-root.MuiChatConversationList-populated",
    ".MuiChatMessageList-root",
    ".MuiChatMessageList-root.MuiChatMessageList-populated",
    ".MuiChatComposer-root",
    ".MuiChatComposer-root.MuiChatComposer-streaming",
    ".MuiChatComposer-root.MuiChatComposer-streaming.MuiChatComposer-hasText",
    ".material-mui-x-messages",
    ".material-mui-x-composer",
];

const MUI_X_RUNTIME_SLOT_SELECTORS: &[&str] = &[
    ".MuiDataGrid-withBorderColor",
    ".MuiDataGrid-Toolbar",
    ".MuiDataGrid-Footer",
    ".MuiDataGrid-LoadingOverlay",
    ".MuiDataGrid-NoRowsOverlay",
    ".MuiDataGrid-columnHeader",
    ".MuiDataGrid-columnHeader--sortable",
    ".MuiDataGrid-columnHeader--sorted",
    ".MuiDataGrid-columnHeader--withRightBorder",
    ".MuiDataGrid-cell",
    ".MuiDataGrid-cell--editable",
    ".MuiDataGrid-cell--editing",
    ".MuiDataGrid-cell--withRightBorder",
    ".MuiTreeItem-root",
    ".MuiTreeItem-content",
    ".MuiTreeItem-label",
    ".MuiTreeItem-labelInput",
    ".MuiTreeItem-iconContainer",
    ".MuiTreeItem-checkbox",
    ".MuiTreeItem-expanded",
    ".MuiTreeItem-selected",
    ".MuiTreeItem-editable",
    ".MuiTreeItem-disabledItemsFocusable",
    ".MuiTreeItem-checkboxSelection",
    ".MuiPickersField",
    ".MuiPickersLayout",
    ".MuiPickersToolbar",
    ".MuiPickersPopper",
    ".MuiPickersField-readOnly",
    ".MuiPickersField-hasValue",
    ".MuiPickersField-hasFormat",
    ".MuiPickersField.MuiPickersField-readOnly.MuiPickersField-hasValue.MuiPickersField-hasFormat",
    ".MuiPickersLayout-hasViews",
    ".MuiPickersLayout.MuiPickersLayout-Desktop.MuiPickersLayout-hasViews",
    ".MuiPickersToolbar-ampm",
    ".MuiPickersToolbar-hasViews",
    ".MuiPickersToolbar.MuiPickersToolbar-ampm.MuiPickersToolbar-hasViews",
    ".MuiPickersPopper-open",
    ".MuiPickersPopper-hasDateBounds",
    ".MuiPickersPopper.MuiPickersPopper-open.MuiPickersPopper-hasDateBounds",
    ".MuiChartsLegend",
    ".MuiChartsLegend-hasSeries",
    ".MuiChartsLegend-hasCustomColors",
    ".MuiChartsLegend-loading",
    ".MuiChartsLegend.MuiChartsLegend-hasSeries.MuiChartsLegend-hasCustomColors",
    ".MuiChartsTooltip",
    ".MuiChartsTooltip-loading",
    ".MuiChartsTooltip-hasSeries",
    ".MuiChartsTooltip-hasAxes",
    ".MuiChartsTooltip-hasMargin",
    ".MuiChartsTooltip-interactionHover",
    ".MuiChartsTooltip-hasValue",
    ".MuiChartsTooltip.MuiChartsTooltip-loading.MuiChartsTooltip-hasSeries.MuiChartsTooltip-hasAxes.MuiChartsTooltip-hasMargin.MuiChartsTooltip-interactionHover",
    ".MuiChartsTooltip.MuiChartsTooltip-hasValue.MuiChartsTooltip-hasMargin",
    ".MuiAgentChat-messages",
    ".MuiAgentChat-messagesPopulated",
    ".MuiAgentChat-messagesStreaming",
    ".MuiAgentChat-messagesError",
    ".MuiAgentChat-messages.MuiAgentChat-messagesPopulated.MuiAgentChat-messagesStreaming.MuiAgentChat-messagesError",
    ".MuiAgentChat-composer",
    ".MuiAgentChat-composerStreaming",
    ".MuiAgentChat-composerError",
    ".MuiAgentChat-composerHasText",
    ".MuiAgentChat-composer.MuiAgentChat-composerStreaming.MuiAgentChat-composerError.MuiAgentChat-composerHasText",
    ".MuiChatMessage-root",
    ".MuiChatMessage-populated",
    ".MuiChatMessage-root.MuiChatMessage-populated",
];

const MUI_X_RUNTIME_ROOT_STATE_SELECTORS: &[&str] = &[
    ".MuiDataGrid-loading",
    ".MuiDataGrid-autoHeight",
    ".MuiDataGrid-root--densityCompact",
    ".MuiDataGrid-root--densityStandard",
    ".MuiDataGrid-root--densityComfortable",
    ".MuiDataGrid-checkboxSelection",
    ".MuiDataGrid-disableColumnMenu",
    ".MuiDataGrid-disableRowSelectionOnClick",
    ".MuiDataGrid-hasRows",
    ".MuiDataGrid-hasColumns",
    ".MuiDataGrid-emptyRows",
    ".MuiDataGrid-emptyColumns",
    ".MuiDataGrid-sortingModeClient",
    ".MuiDataGrid-sortingModeServer",
    ".MuiDataGrid-filterModeClient",
    ".MuiDataGrid-filterModeServer",
    ".MuiDataGrid-hasSortModel",
    ".MuiDataGrid-hasFilterModel",
    ".MuiDataGrid-hasPaginationModel",
    ".MuiDataGrid-hasQuickFilter",
    ".MuiDataGrid-showToolbar",
    ".MuiDataGrid-hideFooter",
    ".MuiDataGrid-hideFooterPagination",
    ".MuiDataGrid-hideFooterSelectedRowCount",
    ".MuiDataGrid-withVerticalBorder",
    ".MuiDataGrid-showCellVerticalBorder",
    ".MuiDataGrid-rowSpacingBorder",
    ".MuiDataGrid-rowSpacingMargin",
    ".MuiDataGrid-hasScrollbarSize",
    ".MuiDataGrid-hasLabel",
    ".MuiDataGrid-rowSelection",
    ".MuiDataGrid-hasPage",
    ".MuiDataGrid-hasPageSize",
    ".MuiDataGrid-hasRowCount",
    ".MuiDataGrid-hasRowHeight",
    ".MuiDataGrid-hasOverscan",
    ".MuiDataGrid-disableVirtualization",
    ".MuiDataGrid-hasViewportRange",
    ".MuiDataGrid-hasRequestedRange",
    ".MuiDataGrid-hasScrollOffset",
    ".MuiDataGrid-editModeRow",
    ".MuiDataGrid-editModeCell",
    ".MuiDataGrid-hasCellModes",
    ".MuiDataGrid-hasRowModes",
    ".MuiDataGrid-hasColumnVisibilityModel",
    ".MuiDataGrid-hasPinnedColumns",
    ".MuiMaterialTreeView-root",
    ".MuiTreeView-multiSelect",
    ".MuiTreeView-checkboxSelection",
    ".MuiTreeView-disabledItemsFocusable",
    ".MuiTreeView-editable",
    ".MuiTreeView-hasExpandedItems",
    ".MuiTreeView-hasSelectedItems",
    ".MuiTreeView-hasItemIndentation",
    ".MuiDateTimePickers-root",
    ".MuiDatePicker-root",
    ".MuiTimePicker-root",
    ".MuiPickersLayout-Mobile",
    ".MuiPickersLayout-Static",
    ".MuiPickers-readOnly",
    ".MuiPickers-ampm",
    ".MuiPickers-hasDateBounds",
    ".MuiPickers-hasViews",
    ".MuiPickers-hasValue",
    ".MuiPickers-hasView",
    ".MuiPickers-hasFormat",
    ".MuiCharts-loading",
    ".MuiCharts-hasSeries",
    ".MuiCharts-hasAxes",
    ".MuiCharts-hasCustomColors",
    ".MuiCharts-hasMargin",
    ".MuiGauge-hasValue",
    ".MuiAgentChat-streaming",
    ".MuiAgentChat-error",
    ".MuiAgentChat-hasMessages",
    ".MuiAgentChat-hasComposerText",
    ".MuiChatConversationList-populated",
    ".MuiChatMessageList-populated",
    ".MuiChatComposer-streaming",
    ".MuiChatComposer-hasText",
];

struct SampleStateContract {
    prototype_key: &'static str,
    bool_props: &'static [&'static str],
    string_props: &'static [(&'static str, &'static str)],
    array_props: &'static [&'static str],
    map_props: &'static [&'static str],
    number_props: &'static [(&'static str, f64)],
    selectors: &'static [&'static str],
}

const MUI_X_SAMPLE_STATE_SELECTORS: &[SampleStateContract] = &[
    SampleStateContract {
        prototype_key: "mui_x_data_grid",
        bool_props: &[
            "loading",
            "autoHeight",
            "checkboxSelection",
            "disableVirtualization",
            "showToolbar",
            "hideFooterPagination",
            "hideFooterSelectedRowCount",
            "showCellVerticalBorder",
            "showColumnVerticalBorder",
        ],
        string_props: &[
            ("density", "compact"),
            ("sortingMode", "client"),
            ("filterMode", "client"),
            ("editMode", "row"),
            ("rowSpacingType", "border"),
            ("label", "Material grid"),
        ],
        array_props: &[
            "rows",
            "columns",
            "rowSelectionModel",
            "sortModel",
            "quickFilterValues",
        ],
        map_props: &[
            "filterModel",
            "paginationModel",
            "cellModesModel",
            "rowModesModel",
            "columnVisibilityModel",
            "pinnedColumns",
        ],
        number_props: &[
            ("page", 1.0),
            ("pageSize", 25.0),
            ("rowCount", 100.0),
            ("rowHeight", 32.0),
            ("overscanCount", 4.0),
            ("viewport_start", 5.0),
            ("viewport_count", 20.0),
            ("requested_start", 4.0),
            ("requested_count", 24.0),
            ("scrollTop", 128.0),
            ("scrollbarSize", 12.0),
        ],
        selectors: &[
            ".MuiDataGrid-root.MuiDataGrid-loading",
            ".MuiDataGrid-root.MuiDataGrid-autoHeight",
            ".MuiDataGrid-root.MuiDataGrid-root--densityCompact",
            ".MuiDataGrid-checkboxSelection",
            ".MuiDataGrid-sortingModeClient",
            ".MuiDataGrid-filterModeClient",
            ".MuiDataGrid-hasSortModel",
            ".MuiDataGrid-hasFilterModel",
            ".MuiDataGrid-hasPaginationModel",
            ".MuiDataGrid-hasQuickFilter",
            ".MuiDataGrid-showToolbar",
            ".MuiDataGrid-hideFooterPagination",
            ".MuiDataGrid-hideFooterSelectedRowCount",
            ".MuiDataGrid-withVerticalBorder",
            ".MuiDataGrid-showCellVerticalBorder",
            ".MuiDataGrid-rowSpacingBorder",
            ".MuiDataGrid-hasScrollbarSize",
            ".MuiDataGrid-hasLabel",
            ".MuiDataGrid-rowSelection",
            ".MuiDataGrid-hasRows",
            ".MuiDataGrid-hasColumns",
            ".MuiDataGrid-hasPage",
            ".MuiDataGrid-hasPageSize",
            ".MuiDataGrid-hasRowCount",
            ".MuiDataGrid-hasRowHeight",
            ".MuiDataGrid-hasOverscan",
            ".MuiDataGrid-disableVirtualization",
            ".MuiDataGrid-hasViewportRange",
            ".MuiDataGrid-hasRequestedRange",
            ".MuiDataGrid-hasScrollOffset",
            ".MuiDataGrid-editModeRow",
            ".MuiDataGrid-hasCellModes",
            ".MuiDataGrid-hasRowModes",
            ".MuiDataGrid-hasColumnVisibilityModel",
            ".MuiDataGrid-hasPinnedColumns",
            ".MuiDataGrid-root.MuiDataGrid-hasRows.MuiDataGrid-hasColumns.MuiDataGrid-rowSelection",
            ".MuiDataGrid-root.MuiDataGrid-hasSortModel.MuiDataGrid-hasFilterModel.MuiDataGrid-hasPaginationModel.MuiDataGrid-hasQuickFilter",
            ".MuiDataGrid-root.MuiDataGrid-showToolbar.MuiDataGrid-hideFooterPagination.MuiDataGrid-hideFooterSelectedRowCount.MuiDataGrid-withVerticalBorder.MuiDataGrid-showCellVerticalBorder.MuiDataGrid-rowSpacingBorder.MuiDataGrid-hasScrollbarSize.MuiDataGrid-hasLabel",
            ".MuiDataGrid-root.MuiDataGrid-hasPage.MuiDataGrid-hasPageSize",
            ".MuiDataGrid-root.MuiDataGrid-hasRowCount.MuiDataGrid-hasRowHeight.MuiDataGrid-hasOverscan.MuiDataGrid-disableVirtualization",
            ".MuiDataGrid-root.MuiDataGrid-hasViewportRange.MuiDataGrid-hasRequestedRange.MuiDataGrid-hasScrollOffset",
            ".MuiDataGrid-root.MuiDataGrid-editModeRow.MuiDataGrid-hasCellModes.MuiDataGrid-hasRowModes.MuiDataGrid-hasColumnVisibilityModel.MuiDataGrid-hasPinnedColumns",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_tree_view",
        bool_props: &[
            "multiSelect",
            "checkboxSelection",
            "disabledItemsFocusable",
            "editable",
        ],
        string_props: &[],
        array_props: &["defaultExpandedItems", "selectedItems"],
        map_props: &["slotProps"],
        number_props: &[("itemChildrenIndentation", 18.0)],
        selectors: &[
            ".MuiTreeView-root.MuiTreeView-multiSelect",
            ".MuiTreeView-root.MuiTreeView-checkboxSelection.MuiTreeView-disabledItemsFocusable.MuiTreeView-editable",
            ".MuiTreeView-hasExpandedItems",
            ".MuiTreeView-hasSelectedItems",
            ".MuiTreeView-root.MuiTreeView-hasExpandedItems.MuiTreeView-hasSelectedItems.MuiTreeView-hasItemIndentation",
            ".MuiTreeItem-root.MuiTreeItem-expanded.MuiTreeItem-selected.MuiTreeItem-editable.MuiTreeItem-disabledItemsFocusable",
            ".MuiTreeItem-content.MuiTreeItem-expanded.MuiTreeItem-selected",
            ".MuiTreeItem-label.MuiTreeItem-labelInput",
            ".MuiTreeItem-checkbox.MuiTreeItem-checkboxSelection",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_date_time_pickers",
        bool_props: &["ampm", "readOnly"],
        string_props: &[
            ("variant", "desktop"),
            ("minDate", "2026-01-01"),
            ("maxDate", "2026-12-31"),
            ("value", "2026-05-17 09:30"),
            ("view", "day"),
            ("format", "yyyy-MM-dd HH:mm"),
        ],
        array_props: &["views"],
        map_props: &["slotProps"],
        number_props: &[],
        selectors: &[
            ".MuiDateTimePicker-root.MuiPickersLayout-Desktop",
            ".MuiDateTimePicker-root.MuiPickers-readOnly.MuiPickers-ampm.MuiPickers-hasDateBounds.MuiPickers-hasViews",
            ".MuiPickers-hasDateBounds",
            ".MuiPickers-hasViews",
            ".MuiPickers-hasValue",
            ".MuiPickers-hasView",
            ".MuiPickers-hasFormat",
            ".MuiDateTimePicker-root.MuiPickers-hasValue.MuiPickers-hasView.MuiPickers-hasFormat",
            ".MuiPickersField.MuiPickersField-readOnly.MuiPickersField-hasValue.MuiPickersField-hasFormat",
            ".MuiPickersLayout.MuiPickersLayout-Desktop.MuiPickersLayout-hasViews",
            ".MuiPickersToolbar.MuiPickersToolbar-ampm.MuiPickersToolbar-hasViews",
            ".MuiPickersPopper.MuiPickersPopper-open.MuiPickersPopper-hasDateBounds",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_charts",
        bool_props: &["loading"],
        string_props: &[("interaction", "hover")],
        array_props: &["series", "x_axis", "y_axis", "colors"],
        map_props: &["margin"],
        number_props: &[],
        selectors: &[
            ".MuiChartsSurface-root.MuiCharts-loading",
            ".MuiCharts-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
            ".MuiChartsLegend.MuiChartsLegend-hasSeries.MuiChartsLegend-hasCustomColors",
            ".MuiChartsTooltip.MuiChartsTooltip-loading.MuiChartsTooltip-hasSeries.MuiChartsTooltip-hasAxes.MuiChartsTooltip-hasMargin.MuiChartsTooltip-interactionHover",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_line_chart",
        bool_props: &["loading"],
        string_props: &[("interaction", "hover")],
        array_props: &["series", "x_axis", "y_axis", "colors"],
        map_props: &["margin"],
        number_props: &[],
        selectors: &[
            ".MuiChartsSurface-root.MuiCharts-loading",
            ".MuiLineChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
            ".MuiChartsLegend.MuiChartsLegend-hasSeries.MuiChartsLegend-hasCustomColors",
            ".MuiChartsTooltip.MuiChartsTooltip-loading.MuiChartsTooltip-hasSeries.MuiChartsTooltip-hasAxes.MuiChartsTooltip-hasMargin.MuiChartsTooltip-interactionHover",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_bar_chart",
        bool_props: &[],
        string_props: &[],
        array_props: &["series", "x_axis", "y_axis", "colors"],
        map_props: &["margin"],
        number_props: &[],
        selectors: &[
            ".MuiBarChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_pie_chart",
        bool_props: &[],
        string_props: &[],
        array_props: &["series", "x_axis", "y_axis", "colors"],
        map_props: &["margin"],
        number_props: &[],
        selectors: &[
            ".MuiPieChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_sparkline",
        bool_props: &[],
        string_props: &[],
        array_props: &["series", "x_axis", "y_axis", "colors"],
        map_props: &["margin"],
        number_props: &[],
        selectors: &[
            ".MuiSparkLineChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_gauge",
        bool_props: &[],
        string_props: &[("interaction", "hover")],
        array_props: &["series", "x_axis", "y_axis", "colors"],
        map_props: &["margin"],
        number_props: &[("value", 0.68)],
        selectors: &[
            ".MuiGauge-hasValue",
            ".MuiGauge-root.MuiChartsSurface-root.MuiGauge-hasValue.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin",
            ".MuiChartsTooltip.MuiChartsTooltip-hasValue.MuiChartsTooltip-hasMargin",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_agent_chat",
        bool_props: &["streaming", "error"],
        string_props: &[("composer_text", "Generate compact Material states")],
        array_props: &["messages"],
        map_props: &["slotProps"],
        number_props: &[],
        selectors: &[
            ".MuiAgentChat-root.MuiAgentChat-streaming",
            ".MuiAgentChat-root.MuiAgentChat-error",
            ".MuiAgentChat-root.MuiAgentChat-hasMessages.MuiAgentChat-hasComposerText",
            ".MuiAgentChat-messages.MuiAgentChat-messagesPopulated.MuiAgentChat-messagesStreaming.MuiAgentChat-messagesError",
            ".MuiAgentChat-composer.MuiAgentChat-composerStreaming.MuiAgentChat-composerError.MuiAgentChat-composerHasText",
        ],
    },
    SampleStateContract {
        prototype_key: "mui_x_chat_composer",
        bool_props: &["streaming"],
        string_props: &[("composer_text", "Apply compact density")],
        array_props: &[],
        map_props: &[],
        number_props: &[],
        selectors: &[
            ".MuiChatComposer-root.MuiChatComposer-streaming",
            ".MuiChatComposer-root.MuiChatComposer-streaming.MuiChatComposer-hasText",
        ],
    },
];

#[test]
fn material_component_lab_theme_styles_mui_x_runtime_utility_classes() {
    let source = editor_theme_source();
    let selectors = theme_selectors(&source);

    for selector in MUI_X_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style MUI X selector `{selector}`"
        );
    }

    for token in [
        "$mui_web_palette_primary_main",
        "$mui_web_palette_secondary_main",
        "$mui_web_palette_info_main",
        "$mui_web_palette_success_main",
        "$mui_web_palette_warning_main",
        "$mui_web_palette_error_main",
        "$material_info_container",
        "$material_warning_container",
        "$material_error_container",
    ] {
        assert!(
            source.contains(token),
            "MUI X theme rules should reuse MUI Web/editor token `{token}`"
        );
    }
}

#[test]
fn material_component_lab_theme_styles_mui_x_runtime_slot_utility_classes() {
    let source = editor_theme_source();
    let selectors = theme_selectors(&source);

    for selector in MUI_X_RUNTIME_SLOT_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style runtime-generated MUI X slot selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_theme_styles_mui_x_runtime_root_state_classes() {
    let source = editor_theme_source();
    let selectors = theme_selectors(&source);

    for selector in MUI_X_RUNTIME_ROOT_STATE_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style runtime-generated MUI X root/state selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_mui_x_state_props_have_themed_feedback_selectors() {
    let source = editor_theme_source();
    let selectors = theme_selectors(&source);

    for contract in MUI_X_SAMPLE_STATE_SELECTORS {
        let path = editor_asset(&format!(
            "assets/ui/editor/material_components/material_{}.zui",
            contract.prototype_key
        ));
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let sample = document
            .nodes
            .get("sample")
            .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

        for &prop in contract.bool_props {
            assert_eq!(
                sample.props.get(prop).and_then(toml::Value::as_bool),
                Some(true),
                "{} sample should keep state prop `{prop} = true`",
                path.display()
            );
        }
        for &(prop, expected_value) in contract.string_props {
            assert_eq!(
                sample.props.get(prop).and_then(toml::Value::as_str),
                Some(expected_value),
                "{} sample should keep state prop `{prop} = {expected_value:?}`",
                path.display()
            );
        }
        for &prop in contract.array_props {
            assert!(
                sample
                    .props
                    .get(prop)
                    .and_then(toml::Value::as_array)
                    .is_some_and(|values| !values.is_empty()),
                "{} sample should keep non-empty state prop array `{prop}`",
                path.display()
            );
        }
        for &prop in contract.map_props {
            assert!(
                sample
                    .props
                    .get(prop)
                    .and_then(toml::Value::as_table)
                    .is_some_and(|values| !values.is_empty()),
                "{} sample should keep non-empty state prop map `{prop}`",
                path.display()
            );
        }
        for &(prop, expected_value) in contract.number_props {
            let actual_value = sample
                .props
                .get(prop)
                .and_then(|value| {
                    value
                        .as_float()
                        .or_else(|| value.as_integer().map(|value| value as f64))
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} sample should keep numeric state prop `{prop}`",
                        path.display()
                    )
                });
            assert!(
                (actual_value - expected_value).abs() < 0.000_001,
                "{} sample should keep state prop `{prop} = {expected_value}`",
                path.display()
            );
        }
        for &selector in contract.selectors {
            assert!(
                selectors.contains(selector),
                "{} sample state should map to theme feedback selector `{selector}`",
                path.display()
            );
        }
    }
}

#[test]
fn material_component_lab_mui_x_samples_only_use_themed_utility_classes() {
    let source = editor_theme_source();
    let selectors = theme_selectors(&source);

    for prototype_key in MUI_X_PROTOTYPES {
        let path = editor_asset(&format!(
            "assets/ui/editor/material_components/material_{prototype_key}.zui"
        ));
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let sample = document
            .nodes
            .get("sample")
            .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

        let mut class_names = sample
            .classes
            .iter()
            .filter(|class_name| is_mui_x_theme_class(class_name))
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(slot_props) = sample.props.get("slotProps") {
            collect_slot_class_names(slot_props, &mut class_names);
        }

        assert!(
            !class_names.is_empty(),
            "{} sample should expose at least one MUI X utility class",
            path.display()
        );
        for class_name in class_names {
            let selector = format!(".{class_name}");
            assert!(
                selectors.contains(&selector),
                "{} sample utility class `{class_name}` should map to theme selector `{selector}`",
                path.display()
            );
        }
    }
}

fn editor_theme_source() -> String {
    let path = editor_asset("assets/ui/theme/editor_material.v2.ui.toml");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn theme_selectors(source: &str) -> BTreeSet<String> {
    toml::from_str::<toml::Value>(source)
        .unwrap_or_else(|error| panic!("Editor Material theme should parse as TOML: {error}"))
        .get("stylesheets")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|stylesheet| {
            stylesheet
                .get("rules")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|rule| rule.get("selector").and_then(toml::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn collect_slot_class_names(value: &toml::Value, class_names: &mut BTreeSet<String>) {
    match value {
        toml::Value::Array(values) => {
            for value in values {
                collect_slot_class_names(value, class_names);
            }
        }
        toml::Value::Table(table) => {
            for (key, value) in table {
                if key == "className" {
                    if let Some(value) = value.as_str() {
                        class_names.extend(value.split_whitespace().map(ToOwned::to_owned));
                    }
                } else {
                    collect_slot_class_names(value, class_names);
                }
            }
        }
        _ => {}
    }
}

fn is_mui_x_theme_class(class_name: &str) -> bool {
    class_name.starts_with("Mui") || class_name.starts_with("material-mui-x-")
}

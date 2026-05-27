use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const MUI_X_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_mui_x_style"
version = 1
display_name = "MUI Web MUI X Style"

[[stylesheets]]
id = "mui_web_mui_x"

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-withBorderColor.MuiDataGrid-root--densityCompact.MuiDataGrid-loading"
set = { self = { surface_variant = "data-grid-loading" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-emptyRows.MuiDataGrid-emptyColumns.MuiDataGrid-loading"
set = { self = { validation_level = "grid-loading-empty" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-hasRows.MuiDataGrid-hasColumns.MuiDataGrid-rowSelection"
set = { self = { text_tone = "grid-data-bound" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-columnHeaders.header-extra"
set = { self = { text_tone = "grid-header" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-row.MuiDataGrid-row--loading"
set = { self = { validation_level = "grid-row-loading" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-root--densityStandard"
set = { self = { density = "standard" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-root--densityComfortable"
set = { self = { density = "comfortable" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-checkboxSelection.MuiDataGrid-disableColumnMenu.MuiDataGrid-disableRowSelectionOnClick"
set = { self = { text_tone = "grid-feature-flags" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-sortingModeServer.MuiDataGrid-filterModeServer.MuiDataGrid-rowSelection"
set = { self = { validation_level = "grid-server-mode-selection" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-hasSortModel.MuiDataGrid-hasFilterModel.MuiDataGrid-hasPaginationModel.MuiDataGrid-hasQuickFilter"
set = { self = { validation_level = "grid-controlled-models" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-showToolbar.MuiDataGrid-hideFooterPagination.MuiDataGrid-hideFooterSelectedRowCount.MuiDataGrid-withVerticalBorder.MuiDataGrid-showCellVerticalBorder.MuiDataGrid-rowSpacingBorder.MuiDataGrid-hasScrollbarSize.MuiDataGrid-hasLabel"
set = { self = { validation_level = "grid-chrome-customized" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-columnHeader.MuiDataGrid-columnHeader--sortable.MuiDataGrid-columnHeader--sorted.MuiDataGrid-columnHeader--withRightBorder"
set = { self = { text_tone = "grid-column-header-state" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-cell.MuiDataGrid-cell--editable.MuiDataGrid-cell--editing.MuiDataGrid-cell--withRightBorder"
set = { self = { text_tone = "grid-cell-state" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-hasPage.MuiDataGrid-hasPageSize"
set = { self = { validation_level = "grid-paginated" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-hasRowCount.MuiDataGrid-hasRowHeight.MuiDataGrid-hasOverscan.MuiDataGrid-disableVirtualization"
set = { self = { validation_level = "grid-virtualization-config" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-hasViewportRange.MuiDataGrid-hasRequestedRange.MuiDataGrid-hasScrollOffset"
set = { self = { validation_level = "grid-visible-range" } }

[[stylesheets.rules]]
selector = ".MuiDataGrid-root.MuiDataGrid-editModeRow.MuiDataGrid-hasCellModes.MuiDataGrid-hasRowModes.MuiDataGrid-hasColumnVisibilityModel.MuiDataGrid-hasPinnedColumns"
set = { self = { validation_level = "grid-editing-column-state" } }

[[stylesheets.rules]]
selector = ".MuiTreeView-root.MuiTreeView-multiSelect"
set = { self = { surface_variant = "tree-view-multiselect" } }

[[stylesheets.rules]]
selector = ".MuiTreeView-root.MuiTreeView-checkboxSelection.MuiTreeView-disabledItemsFocusable.MuiTreeView-editable"
set = { self = { text_tone = "tree-view-feature-flags" } }

[[stylesheets.rules]]
selector = ".MuiTreeView-root.MuiTreeView-hasExpandedItems.MuiTreeView-hasSelectedItems.MuiTreeView-hasItemIndentation"
set = { self = { validation_level = "tree-view-data-bound" } }

[[stylesheets.rules]]
selector = ".MuiTreeItem-root.MuiTreeItem-expanded.MuiTreeItem-selected.MuiTreeItem-editable.MuiTreeItem-disabledItemsFocusable"
set = { self = { validation_level = "tree-item-state" } }

[[stylesheets.rules]]
selector = ".MuiTreeItem-content.MuiTreeItem-expanded.MuiTreeItem-selected"
set = { self = { text_tone = "tree-item-content" } }

[[stylesheets.rules]]
selector = ".MuiTreeItem-label.MuiTreeItem-labelInput"
set = { self = { text_tone = "tree-item-label-input" } }

[[stylesheets.rules]]
selector = ".MuiTreeItem-iconContainer"
set = { self = { surface_variant = "tree-item-icon" } }

[[stylesheets.rules]]
selector = ".MuiTreeItem-checkbox.MuiTreeItem-checkboxSelection"
set = { self = { surface_variant = "tree-item-checkbox" } }

[[stylesheets.rules]]
selector = ".MuiDateTimePicker-root.MuiPickersLayout-Desktop"
set = { self = { surface_variant = "date-time-picker" } }

[[stylesheets.rules]]
selector = ".MuiDatePicker-root.MuiPickersLayout-Mobile"
set = { self = { surface_variant = "date-picker-mobile" } }

[[stylesheets.rules]]
selector = ".MuiTimePicker-root.MuiPickersLayout-Static"
set = { self = { surface_variant = "time-picker-static" } }

[[stylesheets.rules]]
selector = ".MuiDateTimePicker-root.MuiPickers-readOnly.MuiPickers-ampm.MuiPickers-hasDateBounds.MuiPickers-hasViews"
set = { self = { validation_level = "picker-state-flags" } }

[[stylesheets.rules]]
selector = ".MuiDateTimePicker-root.MuiPickers-hasValue.MuiPickers-hasView.MuiPickers-hasFormat"
set = { self = { text_tone = "picker-value-bound" } }

[[stylesheets.rules]]
selector = ".MuiPickersField.MuiPickersField-readOnly.MuiPickersField-hasValue.MuiPickersField-hasFormat"
set = { self = { text_tone = "picker-field-state" } }

[[stylesheets.rules]]
selector = ".MuiPickersLayout.MuiPickersLayout-Desktop.MuiPickersLayout-hasViews"
set = { self = { surface_variant = "picker-layout-views" } }

[[stylesheets.rules]]
selector = ".MuiPickersToolbar.MuiPickersToolbar-ampm.MuiPickersToolbar-hasViews"
set = { self = { text_tone = "picker-toolbar-state" } }

[[stylesheets.rules]]
selector = ".MuiPickersPopper.MuiPickersPopper-open.MuiPickersPopper-hasDateBounds"
set = { self = { surface_variant = "picker-popper-open-bounds" } }

[[stylesheets.rules]]
selector = ".MuiLineChart-root.MuiChartsSurface-root.MuiCharts-loading"
set = { self = { surface_variant = "line-chart-loading" } }

[[stylesheets.rules]]
selector = ".MuiLineChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin"
set = { self = { text_tone = "chart-configured", validation_level = "chart-customized" } }

[[stylesheets.rules]]
selector = ".MuiBarChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin"
set = { self = { text_tone = "bar-chart-configured" } }

[[stylesheets.rules]]
selector = ".MuiPieChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin"
set = { self = { text_tone = "pie-chart-configured" } }

[[stylesheets.rules]]
selector = ".MuiSparkLineChart-root.MuiChartsSurface-root.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin"
set = { self = { text_tone = "sparkline-configured" } }

[[stylesheets.rules]]
selector = ".MuiGauge-root.MuiChartsSurface-root.MuiGauge-hasValue.MuiCharts-hasSeries.MuiCharts-hasAxes.MuiCharts-hasCustomColors.MuiCharts-hasMargin"
set = { self = { validation_level = "gauge-valued" } }

[[stylesheets.rules]]
selector = ".MuiChartsLegend.MuiChartsLegend-hasSeries.MuiChartsLegend-hasCustomColors"
set = { self = { text_tone = "chart-legend-state" } }

[[stylesheets.rules]]
selector = ".MuiChartsTooltip.MuiChartsTooltip-loading.MuiChartsTooltip-hasSeries.MuiChartsTooltip-hasAxes.MuiChartsTooltip-hasMargin.MuiChartsTooltip-interactionHover"
set = { self = { surface_variant = "chart-tooltip-state" } }

[[stylesheets.rules]]
selector = ".MuiChartsTooltip.MuiChartsTooltip-hasValue.MuiChartsTooltip-hasMargin"
set = { self = { validation_level = "gauge-tooltip-valued" } }

[[stylesheets.rules]]
selector = ".MuiAgentChat-root.MuiAgentChat-streaming.MuiAgentChat-error"
set = { self = { validation_level = "agent-chat-error" } }

[[stylesheets.rules]]
selector = ".MuiAgentChat-root.MuiAgentChat-hasMessages.MuiAgentChat-hasComposerText"
set = { self = { text_tone = "agent-chat-with-content" } }

[[stylesheets.rules]]
selector = ".MuiAgentChat-composer.composer-extra"
set = { self = { text_tone = "composer" } }

[[stylesheets.rules]]
selector = ".MuiAgentChat-messages.MuiAgentChat-messagesPopulated.MuiAgentChat-messagesStreaming.MuiAgentChat-messagesError"
set = { self = { surface_variant = "agent-chat-messages-state" } }

[[stylesheets.rules]]
selector = ".MuiAgentChat-composer.MuiAgentChat-composerStreaming.MuiAgentChat-composerError.MuiAgentChat-composerHasText.composer-extra"
set = { self = { text_tone = "agent-chat-composer-state" } }

[[stylesheets.rules]]
selector = ".MuiChatConversationList-root"
set = { self = { surface_variant = "chat-conversation-list" } }

[[stylesheets.rules]]
selector = ".MuiChatConversationList-root.MuiChatConversationList-populated"
set = { self = { surface_variant = "chat-conversation-list-populated" } }

[[stylesheets.rules]]
selector = ".MuiChatMessageList-root"
set = { self = { surface_variant = "chat-message-list" } }

[[stylesheets.rules]]
selector = ".MuiChatMessageList-root.MuiChatMessageList-populated"
set = { self = { surface_variant = "chat-message-list-populated" } }

[[stylesheets.rules]]
selector = ".MuiChatMessage-root.MuiChatMessage-populated"
set = { self = { text_tone = "chat-message-populated" } }

[[stylesheets.rules]]
selector = ".MuiChatComposer-root.MuiChatComposer-streaming"
set = { self = { text_tone = "chat-composer-streaming" } }

[[stylesheets.rules]]
selector = ".MuiChatComposer-root.MuiChatComposer-streaming.MuiChatComposer-hasText"
set = { self = { validation_level = "chat-composer-active" } }
"##;

const MUI_X_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_mui_x_style_layout"
version = 1
display_name = "MUI Web MUI X Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_mui_x_style.ui"]

[root]
node_id = "mui_x_root"
kind = "native"
type = "VerticalBox"
control_id = "MuiXRoot"

[[root.children]]
[root.children.node]
node_id = "data_grid"
kind = "native"
type = "DataGrid"
control_id = "DataGridRoot"
props = { density = "compact", loading = true, autoHeight = true, rows = [], columns = [], slotProps = { header = { className = "header-extra" } } }

[[root.children.node.children]]
mount = "header"
[root.children.node.children.node]
node_id = "data_grid_header"
kind = "native"
type = "Label"
control_id = "DataGridHeader"
props = { text = "Header" }

[[root.children.node.children]]
mount = "row"
[root.children.node.children.node]
node_id = "data_grid_row"
kind = "native"
type = "Label"
control_id = "DataGridRow"
props = { text = "Loading" }

[[root.children]]
[root.children.node]
node_id = "data_grid_standard"
kind = "native"
type = "DataGrid"
control_id = "DataGridStandardRoot"
props = { density = "standard", rows = [], columns = [] }

[[root.children]]
[root.children.node]
node_id = "data_grid_comfortable"
kind = "native"
type = "DataGrid"
control_id = "DataGridComfortableRoot"
props = { density = "comfortable", rows = [], columns = [] }

[[root.children]]
[root.children.node]
node_id = "data_grid_feature_flags"
kind = "native"
type = "DataGrid"
control_id = "DataGridFeatureFlagsRoot"
props = { checkboxSelection = true, disableColumnMenu = true, disableRowSelectionOnClick = true, rows = [], columns = [] }

[[root.children]]
[root.children.node]
node_id = "data_grid_server_modes"
kind = "native"
type = "DataGrid"
control_id = "DataGridServerModesRoot"
props = { sortingMode = "server", filterMode = "server", rowSelectionModel = ["Alpha"], rows = ["Alpha"], columns = ["Name"] }

[[root.children]]
[root.children.node]
node_id = "data_grid_controlled_models"
kind = "native"
type = "DataGrid"
control_id = "DataGridControlledModelsRoot"
props = { sortModel = ["Name:asc"], filterModel = { items = ["State=Ready"] }, paginationModel = { page = 1, pageSize = 25 }, quickFilterValues = ["Alpha"], rows = ["Alpha"], columns = ["Name", "State"] }

[[root.children]]
[root.children.node]
node_id = "data_grid_chrome_slots"
kind = "native"
type = "DataGrid"
control_id = "DataGridChromeSlotsRoot"
props = { showToolbar = true, hideFooterPagination = true, hideFooterSelectedRowCount = true, showCellVerticalBorder = true, showColumnVerticalBorder = true, rowSpacingType = "border", scrollbarSize = 12, label = "Material grid", sortModel = ["Name:asc"], editMode = "cell", cellModesModel = { Alpha = "edit" }, rows = ["Alpha"], columns = ["Name"] }

[[root.children.node.children]]
mount = "columnHeader"
[root.children.node.children.node]
node_id = "data_grid_column_header"
kind = "native"
type = "Label"
control_id = "DataGridColumnHeader"
props = { text = "Name" }

[[root.children.node.children]]
mount = "cell"
[root.children.node.children.node]
node_id = "data_grid_cell"
kind = "native"
type = "Label"
control_id = "DataGridCell"
props = { text = "Alpha" }

[[root.children]]
[root.children.node]
node_id = "data_grid_pagination"
kind = "native"
type = "DataGrid"
control_id = "DataGridPaginationRoot"
props = { page = 1, pageSize = 25, rows = ["Alpha"], columns = ["Name"] }

[[root.children]]
[root.children.node]
node_id = "data_grid_virtualization"
kind = "native"
type = "DataGrid"
control_id = "DataGridVirtualizationRoot"
props = { rowCount = 100, rowHeight = 32.0, overscanCount = 4, disableVirtualization = true, rows = ["Alpha"], columns = ["Name"] }

[[root.children]]
[root.children.node]
node_id = "data_grid_visible_range"
kind = "native"
type = "DataGrid"
control_id = "DataGridVisibleRangeRoot"
props = { viewport_start = 5, viewport_count = 20, requested_start = 4, requested_count = 24, scrollTop = 128.0, rows = ["Alpha"], columns = ["Name"] }

[[root.children]]
[root.children.node]
node_id = "data_grid_editing_columns"
kind = "native"
type = "DataGrid"
control_id = "DataGridEditingColumnsRoot"
props = { editMode = "row", cellModesModel = { Alpha = "edit" }, rowModesModel = { Beta = "view" }, columnVisibilityModel = { State = false }, pinnedColumns = { left = ["Name"], right = ["State"] }, rows = ["Alpha", "Beta"], columns = ["Name", "State"] }

[[root.children]]
[root.children.node]
node_id = "tree_view"
kind = "native"
type = "MaterialTreeView"
control_id = "TreeViewRoot"
props = { multiSelect = true, checkboxSelection = true, disabledItemsFocusable = true, editable = true, defaultExpandedItems = ["Assets"], selectedItems = ["Assets.Materials"], itemChildrenIndentation = 18.0 }

[[root.children.node.children]]
mount = "item"
[root.children.node.children.node]
node_id = "tree_view_item"
kind = "native"
type = "Label"
control_id = "TreeViewItem"
props = { text = "Assets" }

[[root.children.node.children]]
mount = "content"
[root.children.node.children.node]
node_id = "tree_view_content"
kind = "native"
type = "Label"
control_id = "TreeViewContent"
props = { text = "Materials" }

[[root.children.node.children]]
mount = "label"
[root.children.node.children.node]
node_id = "tree_view_label"
kind = "native"
type = "Label"
control_id = "TreeViewLabel"
props = { text = "Label" }

[[root.children.node.children]]
mount = "icon"
[root.children.node.children.node]
node_id = "tree_view_icon"
kind = "native"
type = "Label"
control_id = "TreeViewIcon"
props = { text = "Icon" }

[[root.children.node.children]]
mount = "checkbox"
[root.children.node.children.node]
node_id = "tree_view_checkbox"
kind = "native"
type = "Label"
control_id = "TreeViewCheckbox"
props = { text = "Check" }

[[root.children]]
[root.children.node]
node_id = "tree_view_feature_flags"
kind = "native"
type = "MaterialTreeView"
control_id = "TreeViewFeatureFlagsRoot"
props = { checkboxSelection = true, disabledItemsFocusable = true, editable = true }

[[root.children]]
[root.children.node]
node_id = "date_time_pickers"
kind = "native"
type = "DateTimePickers"
control_id = "DateTimePickersRoot"
props = { picker_mode = "date_time", variant = "desktop" }

[[root.children]]
[root.children.node]
node_id = "date_picker_mobile"
kind = "native"
type = "DateTimePickers"
control_id = "DatePickerMobileRoot"
props = { picker_mode = "date", variant = "mobile" }

[[root.children]]
[root.children.node]
node_id = "time_picker_static"
kind = "native"
type = "DateTimePickers"
control_id = "TimePickerStaticRoot"
props = { picker_mode = "time", variant = "static" }

[[root.children]]
[root.children.node]
node_id = "date_time_picker_state_flags"
kind = "native"
type = "DateTimePickers"
control_id = "DateTimePickerStateFlagsRoot"
props = { picker_mode = "date_time", variant = "desktop", readOnly = true, ampm = true, minDate = "2026-01-01", maxDate = "2026-12-31", views = ["year", "month", "day", "hours", "minutes"] }

[[root.children]]
[root.children.node]
node_id = "date_time_picker_value_state"
kind = "native"
type = "DateTimePickers"
control_id = "DateTimePickerValueStateRoot"
props = { picker_mode = "date_time", value = "2026-05-17 09:30", view = "day", format = "yyyy-MM-dd HH:mm" }

[[root.children]]
[root.children.node]
node_id = "date_time_picker_slot_state"
kind = "native"
type = "DateTimePickers"
control_id = "DateTimePickerSlotStateRoot"
props = { picker_mode = "date_time", variant = "desktop", value = "2026-05-17 09:30", view = "day", views = ["year", "month", "day"], format = "yyyy-MM-dd HH:mm", readOnly = true, ampm = true, minDate = "2026-01-01", maxDate = "2026-12-31", popup_open = true }

[[root.children.node.children]]
mount = "field"
[root.children.node.children.node]
node_id = "date_time_picker_field"
kind = "native"
type = "Label"
control_id = "DateTimePickerField"
props = { text = "2026-05-17 09:30" }

[[root.children.node.children]]
mount = "layout"
[root.children.node.children.node]
node_id = "date_time_picker_layout"
kind = "native"
type = "Label"
control_id = "DateTimePickerLayout"
props = { text = "Calendar" }

[[root.children.node.children]]
mount = "toolbar"
[root.children.node.children.node]
node_id = "date_time_picker_toolbar"
kind = "native"
type = "Label"
control_id = "DateTimePickerToolbar"
props = { text = "Toolbar" }

[[root.children.node.children]]
mount = "popper"
[root.children.node.children.node]
node_id = "date_time_picker_popper"
kind = "native"
type = "Label"
control_id = "DateTimePickerPopper"
props = { text = "Popup" }

[[root.children]]
[root.children.node]
node_id = "line_chart"
kind = "native"
type = "LineChart"
control_id = "LineChartRoot"
props = { loading = true, series = ["FPS", "GPU"], x_axis = ["1s", "2s", "3s"], y_axis = ["ms"], interaction = "hover", colors = ["#1976d2", "#9c27b0"], margin = { top = 8.0, right = 12.0, bottom = 12.0, left = 8.0 } }

[[root.children.node.children]]
mount = "legend"
[root.children.node.children.node]
node_id = "line_chart_legend"
kind = "native"
type = "Label"
control_id = "LineChartLegend"
props = { text = "Legend" }

[[root.children.node.children]]
mount = "tooltip"
[root.children.node.children.node]
node_id = "line_chart_tooltip"
kind = "native"
type = "Label"
control_id = "LineChartTooltip"
props = { text = "Tooltip" }

[[root.children]]
[root.children.node]
node_id = "bar_chart"
kind = "native"
type = "BarChart"
control_id = "BarChartRoot"
props = { series = ["Editor", "Runtime"], x_axis = ["Input", "Paint"], y_axis = ["ms"], colors = ["#1976d2", "#2e7d32"], margin = { top = 8.0, right = 12.0, bottom = 12.0, left = 8.0 } }

[[root.children]]
[root.children.node]
node_id = "pie_chart"
kind = "native"
type = "PieChart"
control_id = "PieChartRoot"
props = { series = ["Meshes", "Materials"], x_axis = ["Assets"], y_axis = ["count"], colors = ["#1976d2", "#ed6c02"], margin = { top = 8.0, right = 12.0, bottom = 12.0, left = 8.0 } }

[[root.children]]
[root.children.node]
node_id = "sparkline_chart"
kind = "native"
type = "SparkLineChart"
control_id = "SparkLineChartRoot"
props = { series = ["Frame budget"], x_axis = ["A", "B", "C"], y_axis = ["ms"], colors = ["#1976d2"], margin = { top = 8.0, right = 12.0, bottom = 12.0, left = 8.0 } }

[[root.children]]
[root.children.node]
node_id = "gauge"
kind = "native"
type = "Gauge"
control_id = "GaugeRoot"
props = { series = ["Utilization"], x_axis = ["Now"], y_axis = ["percent"], colors = ["#1976d2"], margin = { top = 8.0, right = 12.0, bottom = 12.0, left = 8.0 }, value = 0.68 }

[[root.children.node.children]]
mount = "tooltip"
[root.children.node.children.node]
node_id = "gauge_tooltip"
kind = "native"
type = "Label"
control_id = "GaugeTooltip"
props = { text = "Gauge tooltip" }

[[root.children]]
[root.children.node]
node_id = "agent_chat"
kind = "native"
type = "AgentChat"
control_id = "AgentChatRoot"
props = { messages = ["Designer: inspect MUI X variants"], composer_text = "Ask agent", streaming = true, error = true, slotProps = { composer = { className = "composer-extra" } } }

[[root.children.node.children]]
mount = "messages"
[root.children.node.children.node]
node_id = "agent_chat_messages"
kind = "native"
type = "Label"
control_id = "AgentChatMessages"
props = { text = "Messages" }

[[root.children.node.children]]
mount = "composer"
[root.children.node.children.node]
node_id = "agent_chat_composer"
kind = "native"
type = "ChatComposer"
control_id = "AgentChatComposer"
props = { composer_text = "Send" }

[[root.children]]
[root.children.node]
node_id = "chat_conversation_list"
kind = "native"
type = "ChatConversationList"
control_id = "ChatConversationListRoot"
props = { conversations = ["Design review", "Asset importer"] }

[[root.children]]
[root.children.node]
node_id = "chat_message_list"
kind = "native"
type = "ChatMessageList"
control_id = "ChatMessageListRoot"
props = { messages = ["Designer: inspect MUI X variants"] }

[[root.children.node.children]]
mount = "messages"
[root.children.node.children.node]
node_id = "chat_message_list_message"
kind = "native"
type = "Label"
control_id = "ChatMessageListMessage"
props = { text = "Message row" }

[[root.children]]
[root.children.node]
node_id = "chat_composer"
kind = "native"
type = "ChatComposer"
control_id = "ChatComposerRoot"
props = { composer_text = "Ask agent", streaming = true }
"##;

#[test]
fn mui_x_utility_classes_match_retained_x_targets() {
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

    let tree = find_node(root, "TreeViewRoot");
    assert_eq!(
        str_attr(tree, "surface_variant"),
        Some("tree-view-multiselect")
    );
    assert_eq!(
        str_attr(tree, "validation_level"),
        Some("tree-view-data-bound")
    );
    assert_classes(
        tree,
        &[
            "MuiMaterialTreeView-root",
            "MuiTreeView-root",
            "MuiTreeView-multiSelect",
            "MuiTreeView-checkboxSelection",
            "MuiTreeView-disabledItemsFocusable",
            "MuiTreeView-editable",
            "MuiTreeView-hasExpandedItems",
            "MuiTreeView-hasSelectedItems",
            "MuiTreeView-hasItemIndentation",
        ],
    );

    let tree_item = find_node(root, "TreeViewItem");
    assert_eq!(
        str_attr(tree_item, "validation_level"),
        Some("tree-item-state")
    );
    assert_classes(
        tree_item,
        &[
            "MuiTreeItem-root",
            "MuiTreeItem-expanded",
            "MuiTreeItem-selected",
            "MuiTreeItem-editable",
            "MuiTreeItem-disabledItemsFocusable",
        ],
    );

    let tree_content = find_node(root, "TreeViewContent");
    assert_eq!(
        str_attr(tree_content, "text_tone"),
        Some("tree-item-content")
    );
    assert_classes(
        tree_content,
        &[
            "MuiTreeItem-content",
            "MuiTreeItem-expanded",
            "MuiTreeItem-selected",
        ],
    );

    let tree_label = find_node(root, "TreeViewLabel");
    assert_eq!(
        str_attr(tree_label, "text_tone"),
        Some("tree-item-label-input")
    );
    assert_classes(tree_label, &["MuiTreeItem-label", "MuiTreeItem-labelInput"]);

    let tree_icon = find_node(root, "TreeViewIcon");
    assert_eq!(
        str_attr(tree_icon, "surface_variant"),
        Some("tree-item-icon")
    );
    assert_classes(tree_icon, &["MuiTreeItem-iconContainer"]);

    let tree_checkbox = find_node(root, "TreeViewCheckbox");
    assert_eq!(
        str_attr(tree_checkbox, "surface_variant"),
        Some("tree-item-checkbox")
    );
    assert_classes(
        tree_checkbox,
        &["MuiTreeItem-checkbox", "MuiTreeItem-checkboxSelection"],
    );

    let tree_features = find_node(root, "TreeViewFeatureFlagsRoot");
    assert_eq!(
        str_attr(tree_features, "text_tone"),
        Some("tree-view-feature-flags")
    );
    assert_classes(
        tree_features,
        &[
            "MuiMaterialTreeView-root",
            "MuiTreeView-root",
            "MuiTreeView-checkboxSelection",
            "MuiTreeView-disabledItemsFocusable",
            "MuiTreeView-editable",
        ],
    );

    let pickers = find_node(root, "DateTimePickersRoot");
    assert_eq!(
        str_attr(pickers, "surface_variant"),
        Some("date-time-picker")
    );
    assert_classes(
        pickers,
        &[
            "MuiDateTimePickers-root",
            "MuiDateTimePicker-root",
            "MuiPickersLayout-Desktop",
        ],
    );

    let mobile_date_picker = find_node(root, "DatePickerMobileRoot");
    assert_eq!(
        str_attr(mobile_date_picker, "surface_variant"),
        Some("date-picker-mobile")
    );
    assert_classes(
        mobile_date_picker,
        &["MuiDatePicker-root", "MuiPickersLayout-Mobile"],
    );

    let static_time_picker = find_node(root, "TimePickerStaticRoot");
    assert_eq!(
        str_attr(static_time_picker, "surface_variant"),
        Some("time-picker-static")
    );
    assert_classes(
        static_time_picker,
        &["MuiTimePicker-root", "MuiPickersLayout-Static"],
    );

    let picker_state_flags = find_node(root, "DateTimePickerStateFlagsRoot");
    assert_eq!(
        str_attr(picker_state_flags, "validation_level"),
        Some("picker-state-flags")
    );
    assert_classes(
        picker_state_flags,
        &[
            "MuiDateTimePicker-root",
            "MuiPickersLayout-Desktop",
            "MuiPickers-readOnly",
            "MuiPickers-ampm",
            "MuiPickers-hasDateBounds",
            "MuiPickers-hasViews",
        ],
    );

    let picker_value_state = find_node(root, "DateTimePickerValueStateRoot");
    assert_eq!(
        str_attr(picker_value_state, "text_tone"),
        Some("picker-value-bound")
    );
    assert_classes(
        picker_value_state,
        &[
            "MuiDateTimePicker-root",
            "MuiPickers-hasValue",
            "MuiPickers-hasView",
            "MuiPickers-hasFormat",
        ],
    );

    let picker_field = find_node(root, "DateTimePickerField");
    assert_eq!(
        str_attr(picker_field, "text_tone"),
        Some("picker-field-state")
    );
    assert_classes(
        picker_field,
        &[
            "MuiPickersField",
            "MuiPickersField-readOnly",
            "MuiPickersField-hasValue",
            "MuiPickersField-hasFormat",
        ],
    );

    let picker_layout = find_node(root, "DateTimePickerLayout");
    assert_eq!(
        str_attr(picker_layout, "surface_variant"),
        Some("picker-layout-views")
    );
    assert_classes(
        picker_layout,
        &[
            "MuiPickersLayout",
            "MuiPickersLayout-Desktop",
            "MuiPickersLayout-hasViews",
        ],
    );

    let picker_toolbar = find_node(root, "DateTimePickerToolbar");
    assert_eq!(
        str_attr(picker_toolbar, "text_tone"),
        Some("picker-toolbar-state")
    );
    assert_classes(
        picker_toolbar,
        &[
            "MuiPickersToolbar",
            "MuiPickersToolbar-ampm",
            "MuiPickersToolbar-hasViews",
        ],
    );

    let picker_popper = find_node(root, "DateTimePickerPopper");
    assert_eq!(
        str_attr(picker_popper, "surface_variant"),
        Some("picker-popper-open-bounds")
    );
    assert_classes(
        picker_popper,
        &[
            "MuiPickersPopper",
            "MuiPickersPopper-open",
            "MuiPickersPopper-hasDateBounds",
        ],
    );

    let chart = find_node(root, "LineChartRoot");
    assert_eq!(
        str_attr(chart, "surface_variant"),
        Some("line-chart-loading")
    );
    assert_eq!(str_attr(chart, "text_tone"), Some("chart-configured"));
    assert_eq!(
        str_attr(chart, "validation_level"),
        Some("chart-customized")
    );
    assert_classes(
        chart,
        &[
            "MuiLineChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-loading",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );
    let chart_legend = find_node(root, "LineChartLegend");
    assert_eq!(
        str_attr(chart_legend, "text_tone"),
        Some("chart-legend-state")
    );
    assert_classes(
        chart_legend,
        &[
            "MuiChartsLegend",
            "MuiChartsLegend-hasSeries",
            "MuiChartsLegend-hasCustomColors",
            "MuiChartsLegend-loading",
        ],
    );
    let chart_tooltip = find_node(root, "LineChartTooltip");
    assert_eq!(
        str_attr(chart_tooltip, "surface_variant"),
        Some("chart-tooltip-state")
    );
    assert_classes(
        chart_tooltip,
        &[
            "MuiChartsTooltip",
            "MuiChartsTooltip-loading",
            "MuiChartsTooltip-hasSeries",
            "MuiChartsTooltip-hasAxes",
            "MuiChartsTooltip-hasMargin",
            "MuiChartsTooltip-interactionHover",
        ],
    );

    let bar_chart = find_node(root, "BarChartRoot");
    assert_eq!(
        str_attr(bar_chart, "text_tone"),
        Some("bar-chart-configured")
    );
    assert_classes(
        bar_chart,
        &[
            "MuiBarChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );

    let pie_chart = find_node(root, "PieChartRoot");
    assert_eq!(
        str_attr(pie_chart, "text_tone"),
        Some("pie-chart-configured")
    );
    assert_classes(
        pie_chart,
        &[
            "MuiPieChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );

    let sparkline_chart = find_node(root, "SparkLineChartRoot");
    assert_eq!(
        str_attr(sparkline_chart, "text_tone"),
        Some("sparkline-configured")
    );
    assert_classes(
        sparkline_chart,
        &[
            "MuiSparkLineChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );

    let gauge = find_node(root, "GaugeRoot");
    assert_eq!(str_attr(gauge, "validation_level"), Some("gauge-valued"));
    assert_classes(
        gauge,
        &[
            "MuiGauge-root",
            "MuiChartsSurface-root",
            "MuiGauge-hasValue",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );
    let gauge_tooltip = find_node(root, "GaugeTooltip");
    assert_eq!(
        str_attr(gauge_tooltip, "validation_level"),
        Some("gauge-tooltip-valued")
    );
    assert_classes(
        gauge_tooltip,
        &[
            "MuiChartsTooltip",
            "MuiChartsTooltip-hasSeries",
            "MuiChartsTooltip-hasAxes",
            "MuiChartsTooltip-hasMargin",
            "MuiChartsTooltip-hasValue",
        ],
    );

    let chat = find_node(root, "AgentChatRoot");
    assert_eq!(str_attr(chat, "validation_level"), Some("agent-chat-error"));
    assert_eq!(str_attr(chat, "text_tone"), Some("agent-chat-with-content"));
    assert_classes(
        chat,
        &[
            "MuiAgentChat-root",
            "MuiAgentChat-streaming",
            "MuiAgentChat-error",
            "MuiAgentChat-hasMessages",
            "MuiAgentChat-hasComposerText",
        ],
    );

    let messages = find_node(root, "AgentChatMessages");
    assert_eq!(
        str_attr(messages, "surface_variant"),
        Some("agent-chat-messages-state")
    );
    assert_classes(
        messages,
        &[
            "MuiAgentChat-messages",
            "MuiAgentChat-messagesPopulated",
            "MuiAgentChat-messagesStreaming",
            "MuiAgentChat-messagesError",
        ],
    );

    let composer = find_node(root, "AgentChatComposer");
    assert_eq!(
        str_attr(composer, "text_tone"),
        Some("agent-chat-composer-state")
    );
    assert_classes(
        composer,
        &[
            "MuiAgentChat-composer",
            "MuiAgentChat-composerStreaming",
            "MuiAgentChat-composerError",
            "MuiAgentChat-composerHasText",
            "composer-extra",
        ],
    );

    let conversation_list = find_node(root, "ChatConversationListRoot");
    assert_eq!(
        str_attr(conversation_list, "surface_variant"),
        Some("chat-conversation-list-populated")
    );
    assert_classes(
        conversation_list,
        &[
            "MuiChatConversationList-root",
            "MuiChatConversationList-populated",
        ],
    );

    let message_list = find_node(root, "ChatMessageListRoot");
    assert_eq!(
        str_attr(message_list, "surface_variant"),
        Some("chat-message-list-populated")
    );
    assert_classes(
        message_list,
        &["MuiChatMessageList-root", "MuiChatMessageList-populated"],
    );

    let message = find_node(root, "ChatMessageListMessage");
    assert_eq!(
        str_attr(message, "text_tone"),
        Some("chat-message-populated")
    );
    assert_classes(
        message,
        &["MuiChatMessage-root", "MuiChatMessage-populated"],
    );

    let chat_composer = find_node(root, "ChatComposerRoot");
    assert_eq!(
        str_attr(chat_composer, "text_tone"),
        Some("chat-composer-streaming")
    );
    assert_eq!(
        str_attr(chat_composer, "validation_level"),
        Some("chat-composer-active")
    );
    assert_classes(
        chat_composer,
        &[
            "MuiChatComposer-root",
            "MuiChatComposer-streaming",
            "MuiChatComposer-hasText",
        ],
    );
}

fn find_node<'a>(node: &'a UiTemplateNode, control_id: &str) -> &'a UiTemplateNode {
    if node.control_id.as_deref() == Some(control_id) {
        return node;
    }
    node.children
        .iter()
        .find_map(|child| find_node_opt(child, control_id))
        .unwrap_or_else(|| panic!("missing node `{control_id}`"))
}

fn find_node_opt<'a>(node: &'a UiTemplateNode, control_id: &str) -> Option<&'a UiTemplateNode> {
    if node.control_id.as_deref() == Some(control_id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node_opt(child, control_id))
}

fn str_attr<'a>(node: &'a UiTemplateNode, name: &str) -> Option<&'a str> {
    node.attributes.get(name).and_then(Value::as_str)
}

fn assert_classes(node: &UiTemplateNode, expected: &[&str]) {
    for class_name in expected {
        assert!(
            node.classes.iter().any(|value| value == class_name),
            "missing {class_name} in {:?}",
            node.classes
        );
    }
}

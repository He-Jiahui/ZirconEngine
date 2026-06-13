import { readFileSync, readdirSync } from "node:fs";

const sources = {
  atoms: readLocal("./src/components/inputs/atoms.js"),
  inputUtils: readLocal("./src/components/inputs/input-utils.js"),
  buttons: readLocal("./src/components/inputs/buttons.js"),
  button: readLocal("./src/components/inputs/buttons/button.js"),
  iconButton: readLocal("./src/components/inputs/buttons/icon-button.js"),
  fields: readLocal("./src/components/inputs/fields.js"),
  fieldInput: readLocal("./src/components/inputs/fields/input.js"),
  fieldSearchInput: readLocal("./src/components/inputs/fields/search-input.js"),
  fieldNumber: readLocal("./src/components/inputs/fields/number-field.js"),
  inputSelectionControls: readLocal("./src/components/inputs/selection-controls.js"),
  inputCheckbox: readLocal("./src/components/inputs/selection-controls/checkbox.js"),
  inputRadio: readLocal("./src/components/inputs/selection-controls/radio.js"),
  inputToggle: readLocal("./src/components/inputs/selection-controls/toggle.js"),
  tabs: readLocal("./src/components/inputs/tabs.js"),
  dropdowns: readLocal("./src/components/inputs/dropdowns.js"),
  dropdownSelect: readLocal("./src/components/inputs/dropdowns/select.js"),
  sliders: readLocal("./src/components/inputs/sliders.js"),
  slider: readLocal("./src/components/inputs/sliders/slider.js"),
  rangeSlider: readLocal("./src/components/inputs/sliders/range-slider.js"),
  collections: readLocal("./src/components/data/collections.js"),
  listView: readLocal("./src/components/data/list-view.js"),
  listRow: readLocal("./src/components/data/list-view/row.js"),
  tableView: readLocal("./src/components/data/table-view.js"),
  tableHeader: readLocal("./src/components/data/table-view/header.js"),
  tableRow: readLocal("./src/components/data/table-view/row.js"),
  treeView: readLocal("./src/components/data/tree-view.js"),
  treeRow: readLocal("./src/components/data/tree-view/row.js"),
  alerts: readLocal("./src/components/feedback/alerts.js"),
  toast: readLocal("./src/components/feedback/toast.js"),
  tooltip: readLocal("./src/components/feedback/tooltip.js"),
  menu: readLocal("./src/components/overlays/menu.js"),
  menuRow: readLocal("./src/components/overlays/menu/row.js"),
  popupLayer: readLocal("./src/components/overlays/popup-layer.js"),
  surfaces: readLocal("./src/components/surfaces/surfaces.js"),
  surfaceWindow: readLocal("./src/components/surfaces/shell/window.js"),
  surfaceChrome: readLocal("./src/components/surfaces/shell/chrome.js"),
  surfaceDrawer: readLocal("./src/components/surfaces/panels/drawer-surface.js"),
  surfaceScenePanel: readLocal("./src/components/surfaces/panels/scene-panel.js"),
  surfaceInspectorPanel: readLocal("./src/components/surfaces/panels/inspector-panel.js"),
  surfaceShowcasePanel: readLocal("./src/components/surfaces/panels/showcase-panel.js"),
  surfaceViewport: readLocal("./src/components/surfaces/viewport/viewport-surface.js"),
  moduleComponents: readLocal("./src/modules/shared/module-components.js"),
  moduleActions: readLocal("./src/modules/shared/actions.js"),
  moduleBottomOutput: readLocal("./src/modules/shared/bottom-output.js"),
  modulePanels: readLocal("./src/modules/shared/panels.js"),
  moduleRegions: readLocal("./src/modules/shared/regions.js"),
  moduleRows: readLocal("./src/modules/shared/rows.js"),
  moduleUtils: readLocal("./src/modules/shared/utils.js"),
  moduleVisuals: readLocal("./src/modules/shared/visuals.js"),
  app: [
    "./app.js",
    "./src/app/controller.js",
    "./src/app/controller/activation.js",
    "./src/app/controller/command-application.js",
    "./src/app/controller/create-workbench-controller.js",
    "./src/app/controller/command-routing.js",
    "./src/app/controller/history.js",
    "./src/app/controller/location-state.js",
    "./src/app/controller/rendering.js",
    "./src/app/controller/state.js",
    "./src/app/controller/status.js",
    "./src/app/route-state.js",
    "./src/app/interactions/click.js",
    "./src/app/interactions/click/bind.js",
    "./src/app/interactions/click/dispatch.js",
    "./src/app/interactions/click/handlers.js",
    "./src/app/interactions/click/actions.js",
    "./src/app/interactions/click/actions/feedback.js",
    "./src/app/interactions/click/actions/group.js",
    "./src/app/interactions/click/actions/handle.js",
    "./src/app/interactions/click/actions/menu.js",
    "./src/app/interactions/click/actions/target.js",
    "./src/app/interactions/click/dropdowns.js",
    "./src/app/interactions/click/dropdowns/dismissal.js",
    "./src/app/interactions/click/dropdowns/feedback.js",
    "./src/app/interactions/click/dropdowns/placement.js",
    "./src/app/interactions/click/dropdowns/state.js",
    "./src/app/interactions/click/dropdowns/target.js",
    "./src/app/interactions/click/dropdowns/trigger.js",
    "./src/app/interactions/click/generic.js",
    "./src/app/interactions/click/generic/feedback.js",
    "./src/app/interactions/click/generic/handle.js",
    "./src/app/interactions/click/generic/target.js",
    "./src/app/interactions/click/navigation.js",
    "./src/app/interactions/click/navigation/activate.js",
    "./src/app/interactions/click/navigation/handle.js",
    "./src/app/interactions/click/navigation/target.js",
    "./src/app/interactions/click/rows.js",
    "./src/app/interactions/click/rows/data.js",
    "./src/app/interactions/click/rows/feedback.js",
    "./src/app/interactions/click/rows/selection.js",
    "./src/app/interactions/click/rows/tree.js",
    "./src/app/interactions/click/selection.js",
    "./src/app/interactions/click/selection/feedback.js",
    "./src/app/interactions/click/selection/radio.js",
    "./src/app/interactions/click/selection/state.js",
    "./src/app/interactions/click/selection/target.js",
    "./src/app/interactions/click/selection/toggle.js",
    "./src/app/interactions/click/tabs.js",
    "./src/app/interactions/click/tabs/feedback.js",
    "./src/app/interactions/click/tabs/handle.js",
    "./src/app/interactions/click/tabs/panel.js",
    "./src/app/interactions/click/tabs/state.js",
    "./src/app/interactions/click/tabs/target.js",
    "./src/app/interactions/click/toolbar.js",
    "./src/app/interactions/click/toolbar/feedback.js",
    "./src/app/interactions/click/toolbar/rail.js",
    "./src/app/interactions/click/toolbar/state.js",
    "./src/app/interactions/click/toolbar/target.js",
    "./src/app/interactions/click/toolbar/tool.js",
    "./src/app/interactions/click/utils.js",
    "./src/app/interactions/fields.js",
    "./src/app/interactions/fields/bind.js",
    "./src/app/interactions/fields/focus.js",
    "./src/app/interactions/fields/input.js",
    "./src/app/interactions/fields/target.js",
    "./src/app/interactions/keyboard.js",
    "./src/app/interactions/keyboard/activate.js",
    "./src/app/interactions/keyboard/bind.js",
    "./src/app/interactions/keyboard/filter.js",
    "./src/app/interactions/keyboard/target.js",
    "./src/app/interactions/history.js",
    "./src/app/interactions/history/bind.js",
    "./src/app/interactions/history/events.js",
  ].map(readLocal).join("\n"),
  componentDrawerZui: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui"),
  templateBuildSession: readRepo("../../../../zircon_editor/src/ui/template_runtime/runtime/build_session.rs"),
  zuiAssetGovernance: readRepo("../../../../zircon_editor/src/tests/ui/boundary/zui_asset_governance.rs"),
  zuiNodeComponentGovernance: readRepo("../../../../zircon_editor/src/tests/ui/boundary/zui_asset_governance/node_component.rs"),
  zuiWorkbenchPrimitiveGovernance: readRepo("../../../../zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_primitives.rs"),
  templateNodes: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs"),
  buttonStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_button.rs"),
  chromeStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_chrome.rs"),
  iconButtonStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_icon_button.rs"),
  textFieldStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_text_field.rs"),
  selectionControlStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_selection_control.rs"),
  dropdownStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_dropdown.rs"),
  sliderStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_slider.rs"),
  popupRowStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_popup_row.rs"),
  listRowStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_list_row.rs"),
  treeRowStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_tree_row.rs"),
  tableRowStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_table_row.rs"),
  segmentedStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_segmented_control.rs"),
  tooltipStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_tooltip.rs"),
  toastStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_toast.rs"),
  selectionControls: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/template_selection_controls.rs"),
  componentFamily: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs"),
  inputSemantics: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs"),
  activationSemantics: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs"),
  keyboardSemantics: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs"),
  pointerSemantics: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs"),
  popupDismiss: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs"),
  templateGeometry: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/template_geometry.rs"),
  popupLayout: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs"),
  virtualRows: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs"),
  popupPrimitives: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs"),
  runtimeButtons: readRepo("../../../../zircon_runtime/src/ui/surface/render/buttons.rs"),
  runtimeChrome: readRepo("../../../../zircon_runtime/src/ui/surface/render/chrome.rs"),
  runtimeCollectionRows: readRepo("../../../../zircon_runtime/src/ui/surface/render/collection_rows/mod.rs"),
  runtimeCollectionRowsShared: readRepo("../../../../zircon_runtime/src/ui/surface/render/collection_rows/shared.rs"),
  runtimeCollectionRowsList: readRepo("../../../../zircon_runtime/src/ui/surface/render/collection_rows/list.rs"),
  runtimeCollectionRowsTree: readRepo("../../../../zircon_runtime/src/ui/surface/render/collection_rows/tree.rs"),
  runtimeCollectionRowsTable: readRepo("../../../../zircon_runtime/src/ui/surface/render/collection_rows/table.rs"),
  runtimeSelectionControls: readRepo("../../../../zircon_runtime/src/ui/surface/render/selection_controls.rs"),
  runtimeSegmentedControls: readRepo("../../../../zircon_runtime/src/ui/surface/render/segmented_controls.rs"),
  runtimeSliders: readRepo("../../../../zircon_runtime/src/ui/surface/render/sliders.rs"),
  runtimeDropdowns: readRepo("../../../../zircon_runtime/src/ui/surface/render/dropdowns.rs"),
  runtimeFeedback: readRepo("../../../../zircon_runtime/src/ui/surface/render/feedback.rs"),
  runtimeFeedbackState: readRepo("../../../../zircon_runtime/src/ui/surface/render/feedback/state.rs"),
  runtimeTextFields: readRepo("../../../../zircon_runtime/src/ui/surface/render/text_fields.rs"),
  runtimeExtract: readRepo("../../../../zircon_runtime/src/ui/surface/render/extract.rs"),
  runtimePopupRows: readRepo("../../../../zircon_runtime/src/ui/surface/render/popup_rows.rs"),
  runtimePopupMenu: readRepo("../../../../zircon_runtime/src/ui/surface/render/popup_menu.rs"),
  runtimePopupOptions: readRepo("../../../../zircon_runtime/src/ui/surface/render/popup_options.rs"),
  runtimePrototypeStore: readRepo("../../../../zircon_runtime/src/ui/template/asset/prototype_store.rs"),
  runtimeComponentInstancer: readRepo("../../../../zircon_runtime/src/ui/v2/component_instancer.rs"),
  runtimeTestsMod: readRepo("../../../../zircon_runtime/src/ui/tests/mod.rs"),
  runtimeButtonTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_buttons.rs"),
  runtimeChromeTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_chrome.rs"),
  runtimeCollectionRowTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_collection_rows.rs"),
  runtimeSelectionTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_selection_controls.rs"),
  runtimeSegmentedControlTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_segmented_controls.rs"),
  runtimeSliderTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_sliders.rs"),
  runtimeDropdownTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_dropdowns.rs"),
  runtimeFeedbackTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_feedback.rs"),
  runtimeTextFieldTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_text_fields.rs"),
  runtimePopupOptionsTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_popup_options.rs")
};
const componentAssetFiles = componentAssetPaths(new URL("../../../../zircon_editor/assets/ui/editor/components/", import.meta.url));
const extensionModuleDomainFolders = [
  "workbench/modules/extensions/animation/",
  "workbench/modules/extensions/data/",
  "workbench/modules/extensions/diagnostics/",
  "workbench/modules/extensions/gameplay/",
  "workbench/modules/extensions/index/",
  "workbench/modules/extensions/multiplayer/",
  "workbench/modules/extensions/production/",
  "workbench/modules/extensions/rendering/",
  "workbench/modules/extensions/simulation/",
  "workbench/modules/extensions/ui/",
  "workbench/modules/extensions/world/"
];
const coreModuleDomainFolders = [
  "workbench/modules/core/ai/",
  "workbench/modules/core/assets/",
  "workbench/modules/core/gameplay/",
  "workbench/modules/core/index/",
  "workbench/modules/core/rendering/",
  "workbench/modules/core/ui/"
];
const allowedComponentAssetFolders = [
  "showcase/",
  "workbench/primitives/inputs/",
  "workbench/primitives/data/",
  "workbench/primitives/feedback/",
  "workbench/primitives/chrome/",
  "workbench/shell/",
  "workbench/modules/generated/",
  ...coreModuleDomainFolders,
  ...extensionModuleDomainFolders
];
const flatComponentRootZuiFiles = componentAssetFiles.filter((file) => !file.includes("/"));
const flatCoreModuleZuiFiles = componentAssetFiles.filter((file) =>
  file.startsWith("workbench/modules/core/")
    && file.endsWith(".zui")
    && file.slice("workbench/modules/core/".length).split("/").length === 1
);
const flatExtensionModuleZuiFiles = componentAssetFiles.filter((file) =>
  file.startsWith("workbench/modules/extensions/")
    && file.endsWith(".zui")
    && file.slice("workbench/modules/extensions/".length).split("/").length === 1
);
const miscategorizedComponentAssets = componentAssetFiles.filter((file) =>
  !allowedComponentAssetFolders.some((folder) => file.startsWith(folder))
);

const nativeComponentFamilies = [
  {
    name: "button atom",
    web: ["export function button", "zr-button"],
    painter: ["template_buttons::push_button_commands", "push_button_commands("],
    files: ["template_buttons.rs"]
  },
  {
    name: "icon button atom",
    web: ["export function iconButton", "zr-icon-button"],
    painter: ["template_icon_buttons::push_icon_button_commands", "push_icon_button_commands("],
    files: ["template_icon_buttons.rs"]
  },
  {
    name: "text input atom",
    web: ["export function input", "zr-input"],
    painter: ["template_fields::push_field_commands", "push_field_commands("],
    files: ["template_fields.rs"]
  },
  {
    name: "checkbox radio toggle atom",
    web: ["export function checkbox", "export function radio", "export function toggle"],
    painter: ["template_selection_controls::push_selection_control_commands", "push_selection_control_commands("],
    files: ["template_selection_controls.rs"]
  },
  {
    name: "tabs segmented controls",
    web: ["export function tabs", "zr-segment"],
    painter: ["template_segmented_controls::push_segmented_control_commands", "push_segmented_control_commands("],
    files: ["template_segmented_controls.rs"]
  },
  {
    name: "dropdown select atom",
    web: ["export function select", "data-dropdown"],
    painter: ["template_dropdowns::{dropdown_paint_rect, push_dropdown_commands}", "push_dropdown_commands("],
    files: ["template_dropdowns.rs", "template_popup_rows.rs"]
  },
  {
    name: "slider atom",
    web: ["export function slider", "export function rangeSlider"],
    painter: ["template_sliders::push_slider_commands", "push_slider_commands("],
    files: ["template_sliders.rs"]
  },
  {
    name: "list collection",
    web: ["export function listView", "zr-list-item"],
    painter: ["template_list_rows::push_list_row_commands", "push_list_row_commands("],
    files: ["template_list_rows.rs"]
  },
  {
    name: "tree collection",
    web: ["export function treeView", "zr-tree-row"],
    painter: ["template_tree_rows::push_tree_row_commands", "push_tree_row_commands("],
    files: ["template_tree_rows.rs"]
  },
  {
    name: "table collection",
    web: ["export function tableView", "zr-table-row"],
    painter: ["template_table_rows::{push_table_row_commands, push_table_row_text_commands}", "push_table_row_commands("],
    files: ["template_table_rows.rs"]
  },
  {
    name: "popup menu collection",
    web: ["export function menu", "data-menu-item"],
    painter: ["template_popup_rows::push_template_popup_row_commands", "push_template_popup_row_commands("],
    files: ["template_popup_rows.rs"]
  },
  {
    name: "alert feedback collection",
    web: ["export function alerts", "zr-alert"],
    painter: ["template_alerts::push_alert_commands", "push_alert_commands("],
    files: ["template_alerts.rs"]
  },
  {
    name: "tooltip feedback collection",
    web: ["export function tooltip", "zr-tooltip"],
    painter: ["template_tooltips::push_tooltip_commands", "push_tooltip_commands("],
    files: ["template_tooltips.rs"]
  },
  {
    name: "drawer window panel surfaces",
    web: ["workbenchWindow", "data-surface=\"drawer\"", "data-surface=\"window\"", "data-surface=\"panel-view\""],
    painter: ["template_shell_panels::push_shell_panel_commands", "push_shell_panel_commands("],
    files: ["template_shell_panels.rs"]
  },
  {
    name: "module row primitives",
    web: ["moduleTable", "moduleTree", "listRows"],
    painter: ["template_table_rows", "template_tree_rows", "template_list_rows"],
    files: ["template_table_rows.rs", "template_tree_rows.rs", "template_list_rows.rs"]
  }
];

const nativeZuiComponentAssets = [
  {
    name: "button atom",
    file: "workbench/primitives/inputs/workbench_button.zui",
    componentName: "WorkbenchButton",
    rootComponent: "Button",
    interactive: true,
    needles: ["button_variant", "button_interaction_state"]
  },
  {
    name: "icon button atom",
    file: "workbench/primitives/inputs/workbench_icon_button.zui",
    componentName: "WorkbenchIconButton",
    rootComponent: "IconButton",
    interactive: true,
    needles: ["icon_placement", "layout_icon_size"]
  },
  {
    name: "rail button shell leaf",
    file: "workbench/primitives/chrome/workbench_rail_button.zui",
    componentName: "WorkbenchRailButton",
    rootComponent: "IconButton",
    interactive: true,
    needles: ['button_size = "large"', "layout_icon_size = 22.0"]
  },
  {
    name: "text input atom",
    file: "workbench/primitives/inputs/workbench_field.zui",
    componentName: "WorkbenchField",
    rootComponent: "InputField",
    interactive: true,
    needles: ["editable_text = true", "placeholder"]
  },
  {
    name: "search input atom",
    file: "workbench/primitives/inputs/workbench_search_input.zui",
    componentName: "WorkbenchSearchInput",
    rootComponent: "SearchField",
    interactive: true,
    needles: ["query = \"\"", "search_icon", "value_property = \"query\""]
  },
  {
    name: "checkbox atom",
    file: "workbench/primitives/inputs/workbench_checkbox.zui",
    componentName: "WorkbenchCheckbox",
    rootComponent: "Checkbox",
    interactive: true,
    needles: ["checked = false", "label_color"]
  },
  {
    name: "radio atom",
    file: "workbench/primitives/inputs/workbench_radio.zui",
    componentName: "WorkbenchRadio",
    rootComponent: "Radio",
    interactive: true,
    needles: ["checked = false", "dot_size"]
  },
  {
    name: "toggle atom",
    file: "workbench/primitives/inputs/workbench_toggle.zui",
    componentName: "WorkbenchToggle",
    rootComponent: "Toggle",
    interactive: true,
    needles: ["track_width", "thumb_size"]
  },
  {
    name: "tab atom",
    file: "workbench/primitives/inputs/workbench_tab.zui",
    componentName: "WorkbenchTab",
    rootComponent: "ToggleButton",
    interactive: true,
    needles: ["checked = false", "selected = false"]
  },
  {
    name: "segmented control atom",
    file: "workbench/primitives/inputs/workbench_segmented_control.zui",
    componentName: "WorkbenchSegmentedControl",
    rootComponent: "SegmentedControl",
    interactive: true,
    needles: ['options = ["left", "center", "right"]', 'selection_state = "single"']
  },
  {
    name: "dropdown atom",
    file: "workbench/primitives/inputs/workbench_dropdown.zui",
    componentName: "WorkbenchDropdown",
    rootComponent: "Dropdown",
    interactive: true,
    needles: ["popup_open = false", 'selection_state = "single"']
  },
  {
    name: "slider atom",
    file: "workbench/primitives/inputs/workbench_slider.zui",
    componentName: "WorkbenchSlider",
    rootComponent: "RangeField",
    interactive: true,
    needles: ["min = 0.0", "max = 100.0", "step = 1.0"]
  },
  {
    name: "range slider atom",
    file: "workbench/primitives/inputs/workbench_range_slider.zui",
    componentName: "WorkbenchRangeSlider",
    rootComponent: "RangeSlider",
    interactive: true,
    needles: ["range_min = 20.0", "value = 80.0", "range_min_percent = 0.2", "value_percent = 0.8"]
  },
  {
    name: "number field atom",
    file: "workbench/primitives/inputs/workbench_number_field.zui",
    componentName: "WorkbenchNumberField",
    rootComponent: "NumberField",
    interactive: true,
    needles: ["value = 42.0", "large_step = 10.0", "layout_stepper = true"]
  },
  {
    name: "tab strip atom",
    file: "workbench/primitives/inputs/workbench_tab_strip.zui",
    componentName: "WorkbenchTabStrip",
    rootComponent: "Tabs",
    interactive: true,
    needles: ['options = ["overview", "details", "stats"]', "selection_follows_focus = true"]
  },
  {
    name: "label atom",
    file: "workbench/primitives/data/workbench_label.zui",
    componentName: "WorkbenchLabel",
    rootComponent: "Label",
    interactive: false,
    needles: ['text = "Label"', "font_weight = 500"]
  },
  {
    name: "icon atom",
    file: "workbench/primitives/data/workbench_icon.zui",
    componentName: "WorkbenchIcon",
    rootComponent: "Icon",
    interactive: false,
    needles: ['icon = "zircon_editor_shell/controls/add.svg"', "layout_icon_size = 18.0"]
  },
  {
    name: "list row collection",
    file: "workbench/primitives/data/workbench_list_row.zui",
    componentName: "WorkbenchListRow",
    rootComponent: "ListRow",
    interactive: true,
    needles: ["selected = false", "layout_spacing"]
  },
  {
    name: "tree row collection",
    file: "workbench/primitives/data/workbench_tree_row.zui",
    componentName: "WorkbenchTreeRow",
    rootComponent: "TreeRow",
    interactive: true,
    needles: ["expanded = true", "tree_depth", "tree_indent_px"]
  },
  {
    name: "table row collection",
    file: "workbench/primitives/data/workbench_table_row.zui",
    componentName: "WorkbenchTableRow",
    rootComponent: "Table",
    interactive: true,
    needles: ["options = [", "layout_first_cell_offset_x"]
  },
  {
    name: "divider atom",
    file: "workbench/primitives/data/workbench_divider.zui",
    componentName: "WorkbenchDivider",
    rootComponent: "Divider",
    interactive: false,
    needles: ['orientation = "horizontal"', "thickness = 1.0"]
  },
  {
    name: "popup menu collection",
    file: "workbench/primitives/feedback/workbench_popup_menu.zui",
    componentName: "WorkbenchPopupMenu",
    rootComponent: "ContextActionMenu",
    interactive: true,
    needles: ["menu_items = [", "popup_anchor_x", "popup_anchor_y"]
  },
  {
    name: "progress bar feedback",
    file: "workbench/primitives/feedback/workbench_progress_bar.zui",
    componentName: "WorkbenchProgressBar",
    rootComponent: "Progress",
    interactive: false,
    needles: ["value_percent = 0.64", 'variant = "linear"', "track_fill_color"]
  },
  {
    name: "skeleton feedback",
    file: "workbench/primitives/feedback/workbench_skeleton.zui",
    componentName: "WorkbenchSkeleton",
    rootComponent: "Skeleton",
    interactive: false,
    needles: ['animation = "pulse"', "loading = true", "highlight_color"]
  },
  {
    name: "tooltip feedback",
    file: "workbench/primitives/feedback/workbench_tooltip.zui",
    componentName: "WorkbenchTooltip",
    rootComponent: "Tooltip",
    interactive: false,
    needles: ["arrow_size", "surface_variant"]
  },
  {
    name: "toast feedback",
    file: "workbench/primitives/feedback/workbench_toast.zui",
    componentName: "WorkbenchToast",
    rootComponent: "Alert",
    interactive: false,
    needles: ["severity", "closeText", "status_mark_size"]
  },
  {
    name: "property row primitive",
    file: "workbench/primitives/data/workbench_property_row.zui",
    componentName: "WorkbenchPropertyRow",
    rootComponent: "PropertyRow",
    interactive: false,
    needles: ['text = "Property"', 'value = "Value"']
  },
  {
    name: "editable property row primitive",
    file: "workbench/primitives/data/workbench_component_property_row.zui",
    componentName: "WorkbenchComponentPropertyRow",
    rootComponent: "InputField",
    interactive: true,
    needles: ["editable_text = true", "layout_label_width"]
  },
  {
    name: "chip shell leaf",
    file: "workbench/primitives/chrome/workbench_chip.zui",
    componentName: "WorkbenchChip",
    rootComponent: "Label",
    interactive: false,
    needles: ['text = "Chip"', "font_size = 12.0"]
  },
  {
    name: "axis value field shell leaf",
    file: "workbench/primitives/chrome/workbench_axis_value_field.zui",
    componentName: "WorkbenchAxisValueField",
    rootComponent: "InputField",
    interactive: true,
    needles: ["editable_text = true", "layout_min_height = 24.0"]
  },
  {
    name: "section title shell leaf",
    file: "workbench/primitives/chrome/workbench_section_title.zui",
    componentName: "WorkbenchSectionTitle",
    rootComponent: "Label",
    interactive: false,
    needles: ["font_weight = 700", 'text_tone = "primary"']
  },
  {
    name: "status item shell leaf",
    file: "workbench/primitives/feedback/workbench_status_item.zui",
    componentName: "WorkbenchStatusItem",
    rootComponent: "Label",
    interactive: false,
    needles: ['text = "Status"', "font_size = 12.0"]
  }
];

const componentDrawerImports = [
  ["workbench/primitives/inputs/workbench_button.zui", "WorkbenchButton"],
  ["workbench/primitives/inputs/workbench_checkbox.zui", "WorkbenchCheckbox"],
  ["workbench/primitives/inputs/workbench_dropdown.zui", "WorkbenchDropdown"],
  ["workbench/primitives/inputs/workbench_field.zui", "WorkbenchField"],
  ["workbench/primitives/inputs/workbench_search_input.zui", "WorkbenchSearchInput"],
  ["workbench/primitives/inputs/workbench_icon_button.zui", "WorkbenchIconButton"],
  ["workbench/primitives/data/workbench_label.zui", "WorkbenchLabel"],
  ["workbench/primitives/data/workbench_icon.zui", "WorkbenchIcon"],
  ["workbench/primitives/data/workbench_divider.zui", "WorkbenchDivider"],
  ["workbench/primitives/data/workbench_list_row.zui", "WorkbenchListRow"],
  ["workbench/primitives/feedback/workbench_popup_menu.zui", "WorkbenchPopupMenu"],
  ["workbench/primitives/feedback/workbench_progress_bar.zui", "WorkbenchProgressBar"],
  ["workbench/primitives/inputs/workbench_radio.zui", "WorkbenchRadio"],
  ["workbench/primitives/inputs/workbench_segmented_control.zui", "WorkbenchSegmentedControl"],
  ["workbench/primitives/inputs/workbench_slider.zui", "WorkbenchSlider"],
  ["workbench/primitives/inputs/workbench_range_slider.zui", "WorkbenchRangeSlider"],
  ["workbench/primitives/feedback/workbench_skeleton.zui", "WorkbenchSkeleton"],
  ["workbench/primitives/inputs/workbench_tab.zui", "WorkbenchTab"],
  ["workbench/primitives/inputs/workbench_tab_strip.zui", "WorkbenchTabStrip"],
  ["workbench/primitives/data/workbench_table_row.zui", "WorkbenchTableRow"],
  ["workbench/primitives/feedback/workbench_toast.zui", "WorkbenchToast"],
  ["workbench/primitives/feedback/workbench_tooltip.zui", "WorkbenchTooltip"],
  ["workbench/primitives/inputs/workbench_toggle.zui", "WorkbenchToggle"],
  ["workbench/primitives/inputs/workbench_number_field.zui", "WorkbenchNumberField"]
];

const nativeZuiShellSurfaceAssets = [
  {
    name: "activity rail shell surface",
    file: "workbench/shell/workbench_activity_rail.zui",
    componentName: "WorkbenchActivityRail",
    rootNode: "activity_rail",
    rootComponent: "VerticalGroup",
    rootControlId: "WorkbenchWindowActivityRail",
    classes: ["workbench-rail"],
    imports: ["workbench/primitives/chrome/workbench_rail_button.zui#WorkbenchRailButton"],
    mountedComponents: ["WorkbenchRailButton"],
    controlIds: ["WorkbenchRailScene", "WorkbenchRailCube", "WorkbenchRailCode"]
  },
  {
    name: "top toolbar shell surface",
    file: "workbench/shell/workbench_top_toolbar.zui",
    componentName: "WorkbenchTopToolbar",
    rootNode: "top_toolbar",
    rootComponent: "HorizontalGroup",
    rootControlId: "WorkbenchWindowTopToolbar",
    classes: ["workbench-topbar"],
    imports: ["workbench/primitives/inputs/workbench_button.zui#WorkbenchButton", "workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton", "workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab"],
    mountedComponents: ["WorkbenchButton", "WorkbenchIconButton", "WorkbenchTab"],
    controlIds: ["WorkbenchToolbarFileGroup", "WorkbenchModuleTabs", "WorkbenchModuleCommands", "WorkbenchToolbarRunGroup"]
  },
  {
    name: "scene tree panel shell surface",
    file: "workbench/shell/workbench_scene_tree_panel.zui",
    componentName: "WorkbenchSceneTreePanel",
    rootNode: "scene_tree_panel",
    rootComponent: "VerticalGroup",
    rootControlId: "WorkbenchSceneTreePanel",
    classes: ["workbench-panel", "workbench-left-panel"],
    imports: ["workbench/primitives/inputs/workbench_search_input.zui#WorkbenchSearchInput", "workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton", "workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab", "workbench/primitives/data/workbench_tree_row.zui#WorkbenchTreeRow"],
    mountedComponents: ["WorkbenchSearchInput", "WorkbenchIconButton", "WorkbenchTab", "WorkbenchTreeRow"],
    controlIds: ["WorkbenchSceneTabs", "WorkbenchSceneSearchField", "WorkbenchSceneTree", "WorkbenchSceneRootItem"]
  },
  {
    name: "viewport panel shell surface",
    file: "workbench/shell/workbench_viewport_panel.zui",
    componentName: "WorkbenchViewportPanel",
    rootNode: "viewport_panel",
    rootComponent: "VerticalGroup",
    rootControlId: "WorkbenchViewportPanel",
    classes: ["workbench-viewport-panel"],
    imports: ["workbench/primitives/chrome/workbench_chip.zui#WorkbenchChip"],
    mountedComponents: ["WorkbenchChip"],
    controlIds: ["WorkbenchViewportToolbar", "WorkbenchViewportSurface", "WorkbenchViewportGizmoPanel"]
  },
  {
    name: "inspector panel shell surface",
    file: "workbench/shell/workbench_inspector_panel.zui",
    componentName: "WorkbenchInspectorPanel",
    rootNode: "inspector_panel",
    rootComponent: "VerticalGroup",
    rootControlId: "WorkbenchInspectorPanel",
    classes: ["workbench-panel", "workbench-right-panel"],
    imports: [
      "workbench/primitives/chrome/workbench_axis_value_field.zui#WorkbenchAxisValueField",
      "workbench/primitives/inputs/workbench_button.zui#WorkbenchButton",
      "workbench/primitives/data/workbench_component_property_row.zui#WorkbenchComponentPropertyRow",
      "workbench/primitives/inputs/workbench_dropdown.zui#WorkbenchDropdown",
      "workbench/primitives/chrome/workbench_section_title.zui#WorkbenchSectionTitle",
      "workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab"
    ],
    mountedComponents: [
      "WorkbenchAxisValueField",
      "WorkbenchButton",
      "WorkbenchComponentPropertyRow",
      "WorkbenchDropdown",
      "WorkbenchSectionTitle",
      "WorkbenchTab"
    ],
    controlIds: ["WorkbenchInspectorTabs", "WorkbenchInspectorTransform", "WorkbenchInspectorMesh", "WorkbenchAddComponent"]
  },
  {
    name: "status bar shell surface",
    file: "workbench/shell/workbench_status_bar.zui",
    componentName: "WorkbenchStatusBar",
    rootNode: "status_bar",
    rootComponent: "HorizontalGroup",
    rootControlId: "WorkbenchWindowStatusBar",
    classes: ["workbench-status"],
    imports: ["workbench/primitives/chrome/workbench_chip.zui#WorkbenchChip", "workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton", "workbench/primitives/feedback/workbench_status_item.zui#WorkbenchStatusItem"],
    mountedComponents: ["WorkbenchChip", "WorkbenchIconButton", "WorkbenchStatusItem"],
    controlIds: ["WorkbenchStatusReady", "WorkbenchStatusWarnings", "WorkbenchStatusGrid", "WorkbenchStatusZoom"]
  },
  {
    name: "main band shell surface",
    file: "workbench/shell/workbench_main_band.zui",
    componentName: "WorkbenchMainBand",
    rootNode: "main_band",
    rootComponent: "Overlay",
    rootControlId: "WorkbenchMainBand",
    classes: ["workbench-main-band"],
    imports: [
      "workbench/shell/workbench_activity_rail.zui#WorkbenchActivityRail",
      "workbench/shell/workbench_inspector_panel.zui#WorkbenchInspectorPanel",
      "workbench/modules/core/index/workbench_module_workspace.zui#WorkbenchModuleWorkspace",
      "workbench/shell/workbench_scene_tree_panel.zui#WorkbenchSceneTreePanel",
      "workbench/shell/workbench_viewport_panel.zui#WorkbenchViewportPanel"
    ],
    mountedComponents: [
      "WorkbenchActivityRail",
      "WorkbenchInspectorPanel",
      "WorkbenchModuleWorkspace",
      "WorkbenchSceneTreePanel",
      "WorkbenchViewportPanel"
    ],
    controlIds: [
      "WorkbenchSceneWorkspace",
      "WorkbenchMainBandActivityRail",
      "WorkbenchMainBandViewportPanel",
      "WorkbenchMainBandInspectorPanel",
      "WorkbenchMainBandModuleWorkspace"
    ]
  }
];

const nativeInteractionContracts = [
  {
    name: "web delegated click and route responses",
    source: sources.app,
    needles: [
      'document.addEventListener("click"',
      "applyCommandRoute(action)",
      "recordActionFallbackFeedback(controller, action)",
      'actionPath("workbench.action", label)',
      'event.target.closest("[data-module]")',
      'event.target.closest("[data-action]")',
      'selectionControlTarget(event, "[data-toggle]")',
      'selectionControlTarget(event, "[data-radio]")',
      'event.target.closest(".zr-tab, .zr-segment-item, .zr-panel-tab")',
      'event.target.closest("[data-tree-row]")',
      'event.target.closest("[data-dropdown]")'
    ]
  },
  {
    name: "web focus keyboard and input responses",
    source: sources.app,
    needles: [
      'document.addEventListener("focusin"',
      'document.addEventListener("keydown"',
      'document.addEventListener("input"',
      "target.click()",
      'actionPath("workbench.field.focus"',
      'actionPath("workbench.field.edit"'
    ]
  },
  {
    name: "native component family role taxonomy",
    source: sources.componentFamily,
    needles: [
      "enum TemplateComponentFamily",
      "Self::Button =>",
      "Self::IconButton =>",
      "Self::TextInput =>",
      "Self::Slider =>",
      "Self::Checkbox =>",
      "Self::Radio =>",
      "Self::Toggle =>",
      "Self::Dropdown =>",
      "Self::Tab =>",
      "Self::SegmentedControl =>",
      "Self::ListRow =>",
      "Self::TreeRow =>",
      "Self::TableRow =>",
      "Self::Popup =>",
      "Self::Tooltip =>",
      "Self::Alert =>",
      "Self::Drawer =>",
      "Self::Window =>"
    ]
  },
  {
    name: "native text input focus semantics",
    source: sources.inputSemantics,
    needles: [
      "hit_is_text_input",
      "TemplateComponentFamily::TextInput",
      '"input-field" | "number-field"',
      "text_input_edit_target_id",
      "hit_uses_component_text_input_semantics"
    ]
  },
  {
    name: "native activation routes separate focus option menu and bindings",
    source: sources.activationSemantics,
    needles: [
      "TemplatePrimaryActivationRoute::TextInputFocusOnly",
      "TemplatePrimaryActivationRoute::WorkbenchOption",
      "TemplatePrimaryActivationRoute::WorkbenchMenuItem",
      "TemplatePrimaryActivationRoute::SurfaceBinding",
      "TemplatePrimaryActivationRoute::SurfaceAction",
      "invoke_component_showcase_option_selected",
      "invoke_surface_control_clicked"
    ]
  },
  {
    name: "native pointer routing covers workbench and pane controls",
    source: sources.pointerSemantics,
    needles: [
      "route_pointer_to_workbench_window",
      "dispatch_template_node_primary_press",
      "focus_template_node_text_input",
      "set_hovered_workbench_template_hit",
      "PanePointerTarget::TemplateNode",
      "PanePointerTarget::ViewportToolbar",
      "route_activity_rail",
      "route_drawer_header",
      "route_document_tabs"
    ]
  },
  {
    name: "native popup keyboard navigation",
    source: sources.keyboardSemantics,
    needles: [
      "WorkbenchPopupKeyboardCommand::Next",
      "WorkbenchPopupKeyboardCommand::Previous",
      "WorkbenchPopupKeyboardCommand::Accept",
      "WorkbenchPopupKeyboardCommand::Cancel",
      "Key::Named(NamedKey::ArrowDown)",
      "Key::Named(NamedKey::ArrowUp)",
      "Key::Named(NamedKey::Enter)",
      "Key::Named(NamedKey::Escape)",
      "dropdown_option_row_frame_within",
      "menu_item_row_frame"
    ]
  },
  {
    name: "native popup dismiss and bounded layout",
    source: [sources.popupDismiss, sources.templateGeometry, sources.popupLayout].join("\n"),
    needles: [
      "dispatch_workbench_popup_outside_primary_press",
      "PopupDismissTarget",
      "template_popup_bounds",
      "template_nodes_bounds",
      "dropdown_option_popup_frame_within",
      "dropdown_option_row_frame_within",
      "menu_item_row_frame",
      "popup.x = popup.x.clamp(bounds.x, max_x)"
    ]
  },
  {
    name: "native virtual row layout bridge",
    source: sources.virtualRows,
    needles: [
      "TemplateBridgeVirtualRowSequence",
      "UI_V2_REPEAT_KIND_VIRTUAL_ROWS",
      "reconcile",
      "insert_or_reuse_pooled_child",
      "detach_subtree_to_pool",
      "structure_dirty_flags",
      "UiSlotKind::Linear"
    ]
  },
  {
    name: "native popup primitive row state parsing",
    source: sources.popupPrimitives,
    needles: [
      "TemplatePopupMenuItemState",
      "template_popup_menu_item_state",
      "menu_item_without_transient_flags",
      "disabled",
      "separator",
      "matches_transient_menu_item_flag"
    ]
  },
  {
    name: "native builtin import graph resolves zui component assets",
    source: [
      sources.templateBuildSession,
      sources.runtimePrototypeStore,
      sources.runtimeComponentInstancer,
      sources.zuiAssetGovernance,
      sources.zuiNodeComponentGovernance,
      sources.zuiWorkbenchPrimitiveGovernance
    ].join("\n"),
    needles: [
      "compile_template_document_with_builtin_imports",
      "collect_builtin_template_imports",
      "register_document_imports",
      "resolve_builtin_import",
      "root_component_aliases",
      "UiPrototypeStoreBuilder",
      "component_prototype(",
      "resolve_component(",
      "production_v2_zui_widget_imports_resolve_to_named_components",
      "production_widget_import_zui_locators",
      "production_zui_node_components_resolve_to_known_descriptors_or_imported_components",
      "mod workbench_primitives;",
      "WORKBENCH_PRIMITIVE_CONTRACTS",
      "WORKBENCH_SHELL_SURFACE_CONTRACTS",
      "workbench_primitive_component_assets_keep_native_component_contract",
      "workbench_shell_surface_component_assets_keep_bottom_up_composition_contract",
      "workbench_component_drawer_composes_workbench_primitive_assets"
    ]
  }
];

const checks = [];

checks.push([
  "zui component assets are grouped by functional folder",
  componentAssetFiles.length >= 98
    && flatComponentRootZuiFiles.length === 0
    && flatCoreModuleZuiFiles.length === 0
    && flatExtensionModuleZuiFiles.length === 0
    && miscategorizedComponentAssets.length === 0
    && allowedComponentAssetFolders.every((folder) => componentAssetFiles.some((file) => file.startsWith(folder)))
]);

checks.push([
  "rust zui governance rejects flat workbench component assets",
  [
    "editor_workbench_zui_assets_are_grouped_by_functional_component_folder",
    "res://ui/editor/components/showcase/",
    "res://ui/editor/components/workbench/primitives/inputs/",
    "res://ui/editor/components/workbench/primitives/data/",
    "res://ui/editor/components/workbench/primitives/feedback/",
    "res://ui/editor/components/workbench/primitives/chrome/",
    "res://ui/editor/components/workbench/shell/",
    "res://ui/editor/components/workbench/modules/core/ai/",
    "res://ui/editor/components/workbench/modules/core/assets/",
    "res://ui/editor/components/workbench/modules/core/gameplay/",
    "res://ui/editor/components/workbench/modules/core/index/",
    "res://ui/editor/components/workbench/modules/core/rendering/",
    "res://ui/editor/components/workbench/modules/core/ui/",
    "res://ui/editor/components/workbench/modules/extensions/animation/",
    "res://ui/editor/components/workbench/modules/extensions/data/",
    "res://ui/editor/components/workbench/modules/extensions/diagnostics/",
    "res://ui/editor/components/workbench/modules/extensions/gameplay/",
    "res://ui/editor/components/workbench/modules/extensions/index/",
    "res://ui/editor/components/workbench/modules/extensions/multiplayer/",
    "res://ui/editor/components/workbench/modules/extensions/production/",
    "res://ui/editor/components/workbench/modules/extensions/rendering/",
    "res://ui/editor/components/workbench/modules/extensions/simulation/",
    "res://ui/editor/components/workbench/modules/extensions/ui/",
    "res://ui/editor/components/workbench/modules/extensions/world/",
    "res://ui/editor/components/workbench/modules/generated/",
    "checked_assets >= 98"
  ].every((needle) => sources.zuiAssetGovernance.includes(needle))
    && !sources.zuiAssetGovernance.includes('"res://ui/editor/components/workbench/modules/core/",')
    && !sources.zuiAssetGovernance.includes('"res://ui/editor/components/workbench/modules/extensions/",')
]);

for (const family of nativeComponentFamilies) {
  checks.push([
    `${family.name} web surface`,
    family.web.every((needle) => webSources().includes(needle))
  ]);
  checks.push([
    `${family.name} painter dispatch`,
    family.painter.every((needle) => sources.templateNodes.includes(needle))
  ]);
  checks.push([
    `${family.name} native file present`,
    family.files.every((file) => fileSource(file).length > 0)
  ]);
}

for (const asset of nativeZuiComponentAssets) {
  const assetSource = componentAssetSource(asset.file);
  checks.push([
    `${asset.name} declarative zui asset`,
    [
      "[asset]",
      'kind = "component"',
      `id = "res://ui/editor/components/${asset.file}"`,
      `[components.${asset.componentName}]`,
      'root = "root"',
      '"workbench-primitive"',
      "[nodes.root]",
      `component = "${asset.rootComponent}"`,
      `control_id = "${asset.componentName}Root"`,
      "layout = {",
      "width = {",
      "height = {",
      ...asset.needles
    ].every((needle) => assetSource.includes(needle))
  ]);
  if (asset.interactive) {
    checks.push([
      `${asset.name} declarative zui input semantics`,
      [
        "input_interactive = true",
        "input_clickable = true",
        "input_hoverable = true",
        "input_focusable = true"
      ].every((needle) => assetSource.includes(needle))
    ]);
  }
}

checks.push([
  "component drawer composes low-level zui assets",
  componentDrawerImports.every(([file, componentName]) =>
    sources.componentDrawerZui.includes(`res://ui/editor/components/${file}#${componentName}`)
  )
    && sources.componentDrawerZui.includes("[components.WorkbenchComponentDrawer]")
    && sources.componentDrawerZui.includes('component = "WorkbenchButton"')
    && sources.componentDrawerZui.includes('component = "WorkbenchIconButton"')
    && sources.componentDrawerZui.includes('component = "WorkbenchDropdown"')
    && sources.componentDrawerZui.includes('component = "WorkbenchRangeSlider"')
    && sources.componentDrawerZui.includes('component = "WorkbenchPopupMenu"')
    && sources.componentDrawerZui.includes('component = "WorkbenchToast"')
]);

for (const surface of nativeZuiShellSurfaceAssets) {
  const surfaceSource = componentAssetSource(surface.file);
  checks.push([
    `${surface.name} declarative shell composition`,
    [
      "[asset]",
      'kind = "component"',
      `id = "res://ui/editor/components/${surface.file}"`,
      `[components.${surface.componentName}]`,
      `root = "${surface.rootNode}"`,
      `[nodes.${surface.rootNode}]`,
      `component = "${surface.rootComponent}"`,
      `control_id = "${surface.rootControlId}"`,
      "layout = {",
      "width = {",
      "height = {",
      ...surface.classes.map((className) => `"${className}"`),
      ...surface.imports.map((importRef) => `res://ui/editor/components/${importRef}`),
      ...surface.mountedComponents.map((componentName) => `component = "${componentName}"`),
      ...surface.controlIds.map((controlId) => `control_id = "${controlId}"`)
    ].every((needle) => surfaceSource.includes(needle))
  ]);
}

for (const contract of nativeInteractionContracts) {
  checks.push([
    contract.name,
    contract.needles.every((needle) => contract.source.includes(needle))
  ]);
}

checks.push([
  "native button selector state",
  sources.buttonStyle.includes("select_workbench_button_style")
    && sources.buttonStyle.includes("WorkbenchButtonStyle")
    && sources.buttonStyle.includes("button_interaction_state()")
    && sources.buttonStyle.includes("ButtonInteractionState::Pressed")
    && sources.buttonStyle.includes("ButtonInteractionState::Disabled")
]);
checks.push([
  "native chrome selector state",
  sources.chromeStyle.includes("select_workbench_chrome_style")
    && sources.chromeStyle.includes("WorkbenchChromeKind")
    && sources.chromeStyle.includes("WorkbenchChromeStyle")
    && sources.chromeStyle.includes("UiPainterFamily::Chrome")
    && sources.chromeStyle.includes("UiPainterResolvedState::Loading")
    && sources.chromeStyle.includes("UiPainterResolvedState::Focused")
    && sources.chromeStyle.includes("UiPainterResolvedState::DropHovered")
]);
checks.push([
  "native icon button selector state",
  sources.iconButtonStyle.includes("select_workbench_icon_button_style")
    && sources.iconButtonStyle.includes("WorkbenchIconButtonStyle")
    && sources.iconButtonStyle.includes("UiPainterFamily::IconButton")
    && sources.iconButtonStyle.includes("UiPainterResolvedState::Pressed")
    && sources.iconButtonStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native text field selector state",
  sources.textFieldStyle.includes("select_workbench_text_field_style")
    && sources.textFieldStyle.includes("WorkbenchTextFieldStyle")
    && sources.textFieldStyle.includes("UiPainterFamily::TextField")
    && sources.textFieldStyle.includes("UiPainterResolvedState::Pressed")
    && sources.textFieldStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native selection control selector state",
  sources.selectionControlStyle.includes("select_workbench_selection_control_style")
    && sources.selectionControlStyle.includes("WorkbenchSelectionControlKind")
    && sources.selectionControlStyle.includes("UiPainterFamily::Checkbox")
    && sources.selectionControlStyle.includes("UiPainterFamily::Radio")
    && sources.selectionControlStyle.includes("UiPainterFamily::Toggle")
    && sources.selectionControlStyle.includes("UiPainterResolvedState::Pressed")
    && sources.selectionControlStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native dropdown selector state",
  sources.dropdownStyle.includes("select_workbench_dropdown_style")
    && sources.dropdownStyle.includes("WorkbenchDropdownStyle")
    && sources.dropdownStyle.includes("UiPainterFamily::Dropdown")
    && sources.dropdownStyle.includes("UiPainterResolvedState::Pressed")
    && sources.dropdownStyle.includes("UiPainterResolvedState::Open")
    && sources.dropdownStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native slider selector state",
  sources.sliderStyle.includes("select_workbench_slider_style")
    && sources.sliderStyle.includes("WorkbenchSliderStyle")
    && sources.sliderStyle.includes("slider_resolved_state()")
    && sources.sliderStyle.includes("UiPainterResolvedState::Pressed")
    && sources.sliderStyle.includes("UiPainterResolvedState::Focused")
    && sources.sliderStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native popup row selector state",
  sources.popupRowStyle.includes("select_workbench_popup_row_style")
    && sources.popupRowStyle.includes("WorkbenchPopupRowStyle")
    && sources.popupRowStyle.includes("UiPainterFamily::PopupRow")
    && sources.popupRowStyle.includes("UiPainterResolvedState::Pressed")
    && sources.popupRowStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native list row selector state",
  sources.listRowStyle.includes("select_workbench_list_row_style")
    && sources.listRowStyle.includes("WorkbenchListRowStyle")
    && sources.listRowStyle.includes("UiPainterFamily::ListRow")
    && sources.listRowStyle.includes("UiPainterResolvedState::Pressed")
    && sources.listRowStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native tree row selector state",
  sources.treeRowStyle.includes("select_workbench_tree_row_style")
    && sources.treeRowStyle.includes("WorkbenchTreeRowStyle")
    && sources.treeRowStyle.includes("UiPainterFamily::TreeRow")
    && sources.treeRowStyle.includes("UiPainterResolvedState::Pressed")
    && sources.treeRowStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native table row selector state",
  sources.tableRowStyle.includes("select_workbench_table_row_style")
    && sources.tableRowStyle.includes("WorkbenchTableRowStyle")
    && sources.tableRowStyle.includes("UiPainterFamily::TableRow")
    && sources.tableRowStyle.includes("UiPainterResolvedState::Pressed")
    && sources.tableRowStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native tabs segmented selector state",
  sources.segmentedStyle.includes("select_workbench_segmented_control_style")
    && sources.segmentedStyle.includes("WorkbenchSegmentedControlKind")
    && sources.segmentedStyle.includes("UiPainterFamily::Tab")
    && sources.segmentedStyle.includes("UiPainterResolvedState::Pressed")
    && sources.segmentedStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native tooltip selector state",
  sources.tooltipStyle.includes("select_workbench_tooltip_style")
    && sources.tooltipStyle.includes("WorkbenchTooltipStyle")
    && sources.tooltipStyle.includes("UiPainterFamily::Tooltip")
    && sources.tooltipStyle.includes("UiPainterResolvedState::Pressed")
    && sources.tooltipStyle.includes("UiPainterResolvedState::Focused")
    && sources.tooltipStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "native toast selector state",
  sources.toastStyle.includes("select_workbench_toast_style")
    && sources.toastStyle.includes("WorkbenchToastStyle")
    && sources.toastStyle.includes("UiPainterFamily::Toast")
    && sources.toastStyle.includes("UiPainterResolvedState::Pressed")
    && sources.toastStyle.includes("UiPainterResolvedState::Focused")
    && sources.toastStyle.includes("UiPainterResolvedState::Disabled")
]);
checks.push([
  "button icon button runtime extract",
  sources.runtimeButtons.includes('"Button" | "ToggleButton" | "IconButton"')
    && sources.runtimeButtons.includes("button_render_commands")
    && sources.runtimeButtons.includes("button_suppresses_owner_text")
    && sources.runtimeButtons.includes("button_suppresses_owner_image")
    && sources.runtimeButtons.includes("UiPainterFamily::Button")
    && sources.runtimeButtons.includes("UiPainterFamily::IconButton")
    && sources.runtimeButtons.includes("UiVisualAssetRef::Icon")
    && sources.runtimeExtract.includes("button_render_commands")
    && sources.runtimeExtract.includes("button_suppresses_owner_text")
    && sources.runtimeExtract.includes("button_suppresses_owner_image")
]);
checks.push([
  "chrome runtime extract",
  sources.runtimeChrome.includes('"WorkbenchShell" | "Shell" | "WorkbenchWindow"')
    && sources.runtimeChrome.includes('"ActivityRail" | "ActivityRailPanel"')
    && sources.runtimeChrome.includes('"TopToolbar" | "Toolbar" | "MenuBar" | "WorkbenchMenuBar"')
    && sources.runtimeChrome.includes('"StatusBar" | "BottomStatusBar"')
    && sources.runtimeChrome.includes('"ViewportPanel" | "Viewport" | "SceneViewport" | "DocumentViewport"')
    && sources.runtimeChrome.includes("chrome_render_commands")
    && sources.runtimeChrome.includes("chrome_suppresses_owner_surface")
    && sources.runtimeChrome.includes("chrome_suppresses_owner_text")
    && sources.runtimeChrome.includes("chrome_suppresses_owner_image")
    && sources.runtimeChrome.includes("UiPainterFamily::Chrome")
    && sources.runtimeChrome.includes("UiPainterResolvedState::Loading")
    && sources.runtimeExtract.includes("chrome_render_commands")
    && sources.runtimeExtract.includes("chrome_suppresses_owner_surface")
    && sources.runtimeExtract.includes("chrome_suppresses_owner_text")
    && sources.runtimeExtract.includes("chrome_suppresses_owner_image")
]);
checks.push([
  "collection row runtime extract",
  sources.runtimeCollectionRows.includes("collection_row_render_commands")
    && sources.runtimeCollectionRows.includes("collection_row_suppresses_owner_text")
    && sources.runtimeCollectionRows.includes("collection_row_suppresses_owner_image")
    && sources.runtimeCollectionRowsShared.includes('"ListRow"')
    && sources.runtimeCollectionRowsShared.includes('"TreeRow"')
    && sources.runtimeCollectionRowsShared.includes('"Table" | "TableRow"')
    && sources.runtimeCollectionRowsShared.includes("UiPainterFamily::ListRow")
    && sources.runtimeCollectionRowsShared.includes("UiPainterFamily::TreeRow")
    && sources.runtimeCollectionRowsShared.includes("UiPainterFamily::TableRow")
    && sources.runtimeCollectionRowsList.includes("list_row_commands")
    && sources.runtimeCollectionRowsTree.includes("tree_row_commands")
    && sources.runtimeCollectionRowsTable.includes("table_row_commands")
    && sources.runtimeExtract.includes("collection_row_render_commands")
    && sources.runtimeExtract.includes("collection_row_suppresses_owner_text")
    && sources.runtimeExtract.includes("collection_row_suppresses_owner_image")
]);
checks.push([
  "selection controls runtime extract",
  sources.runtimeSelectionControls.includes('SelectionControlKind::Checkbox')
    && sources.runtimeSelectionControls.includes('SelectionControlKind::Radio')
    && sources.runtimeSelectionControls.includes('SelectionControlKind::Toggle')
    && sources.runtimeExtract.includes("selection_control_render_commands")
    && sources.runtimeExtract.includes("selection_control_suppresses_owner_text")
]);
checks.push([
  "tabs segmented controls runtime extract",
  sources.runtimeSegmentedControls.includes('"SegmentedControl" | "Segmented"')
    && sources.runtimeSegmentedControls.includes('"Tab" | "PanelTab"')
    && sources.runtimeSegmentedControls.includes("segmented_control_render_commands")
    && sources.runtimeSegmentedControls.includes("segmented_control_suppresses_owner_text")
    && sources.runtimeSegmentedControls.includes("push_selected_segment")
    && sources.runtimeSegmentedControls.includes("UiPainterFamily::Tab")
    && sources.runtimeExtract.includes("segmented_control_render_commands")
    && sources.runtimeExtract.includes("segmented_control_suppresses_owner_text")
]);
checks.push([
  "slider runtime extract",
  sources.runtimeSliders.includes('metadata.component.as_str()')
    && sources.runtimeSliders.includes('"RangeField" | "Slider" | "RangeSlider"')
    && sources.runtimeSliders.includes("push_track_commands")
    && sources.runtimeSliders.includes("push_thumb_command")
    && sources.runtimeSliders.includes("push_value_box")
    && sources.runtimeExtract.includes("slider_render_commands")
    && sources.runtimeExtract.includes("slider_suppresses_owner_text")
]);
checks.push([
  "dropdown trigger runtime extract",
  sources.runtimeDropdowns.includes('"ComboBox" | "Dropdown" | "Select"')
    && sources.runtimeDropdowns.includes("dropdown_render_commands")
    && sources.runtimeDropdowns.includes("dropdown_suppresses_owner_text")
    && sources.runtimeDropdowns.includes("chevron-up")
    && sources.runtimeDropdowns.includes("option_label_for_value")
    && sources.runtimeExtract.includes("dropdown_render_commands")
    && sources.runtimeExtract.includes("dropdown_suppresses_owner_text")
]);
checks.push([
  "feedback runtime extract",
  sources.runtimeFeedback.includes('"Alert" => Some(FeedbackKind::Alert)')
    && sources.runtimeFeedback.includes('"Tooltip" => Some(FeedbackKind::Tooltip)')
    && sources.runtimeFeedback.includes('"Toast" | "Snackbar" | "SnackbarContent"')
    && sources.runtimeFeedback.includes("feedback_render_commands")
    && sources.runtimeFeedback.includes("feedback_suppresses_owner_text")
    && sources.runtimeFeedback.includes("feedback_suppresses_owner_image")
    && sources.runtimeFeedbackState.includes("UiPainterFamily::Alert")
    && sources.runtimeFeedbackState.includes("UiPainterFamily::Tooltip")
    && sources.runtimeFeedbackState.includes("UiPainterFamily::Toast")
    && sources.runtimeExtract.includes("feedback_render_commands")
    && sources.runtimeExtract.includes("feedback_suppresses_owner_text")
    && sources.runtimeExtract.includes("feedback_suppresses_owner_image")
]);
checks.push([
  "text field runtime extract",
  sources.runtimeTextFields.includes('"InputField" | "TextField" | "LineEdit" | "TextEdit" | "NumberField"')
    && sources.runtimeTextFields.includes("text_field_render_commands")
    && sources.runtimeTextFields.includes("text_field_suppresses_owner_text")
    && sources.runtimeTextFields.includes("UiEditableTextState")
    && sources.runtimeTextFields.includes("layout.editable = editable.cloned()")
    && sources.runtimeTextFields.includes("UiPainterFamily::TextField")
    && sources.runtimeExtract.includes("text_field_render_commands")
    && sources.runtimeExtract.includes("text_field_suppresses_owner_text")
]);
checks.push([
  "popup rows runtime extract",
  sources.runtimePopupRows.includes("push_popup_background")
    && sources.runtimePopupRows.includes("push_popup_row_surface")
    && sources.runtimePopupMenu.includes("popup_menu_render_commands")
    && sources.runtimePopupOptions.includes("popup_option_render_commands")
    && sources.runtimeExtract.includes("popup_menu_render_commands")
    && sources.runtimeExtract.includes("popup_option_render_commands")
]);
checks.push([
  "runtime component render tests registered",
  sources.runtimeTestsMod.includes("mod render_buttons;")
    && sources.runtimeTestsMod.includes("mod render_chrome;")
    && sources.runtimeTestsMod.includes("mod render_collection_rows;")
    && sources.runtimeTestsMod.includes("mod render_selection_controls;")
    && sources.runtimeTestsMod.includes("mod render_segmented_controls;")
    && sources.runtimeTestsMod.includes("mod render_sliders;")
    && sources.runtimeTestsMod.includes("mod render_dropdowns;")
    && sources.runtimeTestsMod.includes("mod render_feedback;")
    && sources.runtimeTestsMod.includes("mod render_text_fields;")
    && sources.runtimeTestsMod.includes("mod render_popup_options;")
    && sources.runtimeButtonTest.includes("render_extract_expands_button_primitives")
    && sources.runtimeButtonTest.includes("render_extract_expands_icon_button_state_surface")
    && sources.runtimeCollectionRowTest.includes("render_extract_expands_collection_row_primitives")
    && sources.runtimeSelectionTest.includes("render_extract_expands_selection_control_indicators")
    && sources.runtimeSegmentedControlTest.includes("render_extract_expands_tabs_and_segmented_control_primitives")
    && sources.runtimeSliderTest.includes("render_extract_expands_slider_primitives")
    && sources.runtimeSliderTest.includes("render_extract_expands_range_slider_dual_thumb_primitives")
    && sources.runtimeDropdownTest.includes("render_extract_expands_dropdown_trigger_primitives")
    && sources.runtimeFeedbackTest.includes("render_extract_expands_feedback_primitives")
    && sources.runtimeTextFieldTest.includes("render_extract_expands_text_field_primitives")
    && sources.runtimePopupOptionsTest.includes("render_extract_expands_open_dropdown_options")
    && sources.runtimeChromeTest.includes("render_extract_expands_workbench_chrome_surfaces")
    && sources.runtimeChromeTest.includes("render_extract_chrome_uses_shared_unavailable_and_active_state_priority")
]);
checks.push([
  "native component contract spans web and runtime",
  nativeComponentFamilies.length >= 15
    && nativeZuiComponentAssets.length >= 32
    && nativeZuiShellSurfaceAssets.length >= 7
    && nativeInteractionContracts.length >= 10
    && sources.templateNodes.includes("push_template_node_commands")
    && sources.runtimeExtract.includes("extract_ui_render_tree_from_arranged")
]);

const failed = checks.filter(([, passed]) => !passed);
for (const [name, passed] of checks) {
  console.log(`${passed ? "ok" : "fail"} ${name}`);
}

if (failed.length > 0) {
  console.error(`Native component contract failed: ${failed.map(([name]) => name).join(", ")}`);
  process.exit(1);
}

console.log(
  `native component contract: families=${nativeComponentFamilies.length} zuiAssets=${nativeZuiComponentAssets.length} shellSurfaces=${nativeZuiShellSurfaceAssets.length} interactions=${nativeInteractionContracts.length} runtimeExtract=button,chrome,collection-row,selection,segmented,slider,dropdown,feedback,text-field,popup`,
);
console.log("ok native component family contract");

function readLocal(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function readRepo(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function webSources() {
  return [
    sources.atoms,
    sources.inputUtils,
    sources.buttons,
    sources.button,
    sources.iconButton,
    sources.fields,
    sources.fieldInput,
    sources.fieldSearchInput,
    sources.fieldNumber,
    sources.inputSelectionControls,
    sources.inputCheckbox,
    sources.inputRadio,
    sources.inputToggle,
    sources.tabs,
    sources.dropdowns,
    sources.dropdownSelect,
    sources.sliders,
    sources.slider,
    sources.rangeSlider,
    sources.collections,
    sources.listView,
    sources.listRow,
    sources.tableView,
    sources.tableHeader,
    sources.tableRow,
    sources.treeView,
    sources.treeRow,
    sources.alerts,
    sources.toast,
    sources.tooltip,
    sources.menu,
    sources.menuRow,
    sources.popupLayer,
    sources.surfaces,
    sources.surfaceWindow,
    sources.surfaceChrome,
    sources.surfaceDrawer,
    sources.surfaceScenePanel,
    sources.surfaceInspectorPanel,
    sources.surfaceShowcasePanel,
    sources.surfaceViewport,
    sources.moduleComponents,
    sources.moduleActions,
    sources.moduleBottomOutput,
    sources.modulePanels,
    sources.moduleRegions,
    sources.moduleRows,
    sources.moduleUtils,
    sources.moduleVisuals
  ].join("\n");
}

function fileSource(fileName) {
  return readRepo(`../../../../zircon_editor/src/ui/retained_host/host_contract/painter/${fileName}`);
}

function componentAssetSource(fileName) {
  return readRepo(`../../../../zircon_editor/assets/ui/editor/components/${fileName}`);
}

function componentAssetPaths(rootUrl, prefix = "") {
  return readdirSync(rootUrl, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isDirectory()) {
      return componentAssetPaths(new URL(`${entry.name}/`, rootUrl), `${prefix}${entry.name}/`);
    }
    if (entry.isFile() && entry.name.endsWith(".zui")) {
      return [`${prefix}${entry.name}`];
    }
    return [];
  });
}

import { readFileSync } from "node:fs";

const sources = {
  atoms: readLocal("./atoms.js"),
  collections: readLocal("./collections.js"),
  surfaces: readLocal("./surfaces.js"),
  moduleComponents: readLocal("./module-components.js"),
  app: readLocal("./app.js"),
  templateNodes: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs"),
  listRowStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_list_row.rs"),
  segmentedStyle: readRepo("../../../../zircon_editor/src/ui/retained_host/host_contract/painter/style_selector/workbench_segmented_control.rs"),
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
  runtimeSelectionControls: readRepo("../../../../zircon_runtime/src/ui/surface/render/selection_controls.rs"),
  runtimeSliders: readRepo("../../../../zircon_runtime/src/ui/surface/render/sliders.rs"),
  runtimeDropdowns: readRepo("../../../../zircon_runtime/src/ui/surface/render/dropdowns.rs"),
  runtimeTextFields: readRepo("../../../../zircon_runtime/src/ui/surface/render/text_fields.rs"),
  runtimeExtract: readRepo("../../../../zircon_runtime/src/ui/surface/render/extract.rs"),
  runtimePopupRows: readRepo("../../../../zircon_runtime/src/ui/surface/render/popup_rows.rs"),
  runtimePopupMenu: readRepo("../../../../zircon_runtime/src/ui/surface/render/popup_menu.rs"),
  runtimePopupOptions: readRepo("../../../../zircon_runtime/src/ui/surface/render/popup_options.rs"),
  runtimeTestsMod: readRepo("../../../../zircon_runtime/src/ui/tests/mod.rs"),
  runtimeSelectionTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_selection_controls.rs"),
  runtimeSliderTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_sliders.rs"),
  runtimeDropdownTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_dropdowns.rs"),
  runtimeTextFieldTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_text_fields.rs"),
  runtimePopupOptionsTest: readRepo("../../../../zircon_runtime/src/ui/tests/render_popup_options.rs")
};

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

const nativeInteractionContracts = [
  {
    name: "web delegated click and route responses",
    source: sources.app,
    needles: [
      'document.addEventListener("click"',
      "applyCommandRoute(action)",
      "recordCommand(action.dataset.action || commandLabel(action))",
      'event.target.closest("[data-module]")',
      'event.target.closest("[data-action]")',
      'event.target.closest("[data-toggle]")',
      'event.target.closest("[data-radio]")',
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
      "recordCommand(`focus-",
      "recordCommand(`edit-"
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
  }
];

const checks = [];

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

for (const contract of nativeInteractionContracts) {
  checks.push([
    contract.name,
    contract.needles.every((needle) => contract.source.includes(needle))
  ]);
}

checks.push([
  "native list row selector state",
  sources.listRowStyle.includes("select_workbench_list_row_style")
    && sources.listRowStyle.includes("WorkbenchListRowStyle")
    && sources.listRowStyle.includes("UiPainterFamily::ListRow")
    && sources.listRowStyle.includes("UiPainterResolvedState::Pressed")
    && sources.listRowStyle.includes("UiPainterResolvedState::Disabled")
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
  "selection controls runtime extract",
  sources.runtimeSelectionControls.includes('SelectionControlKind::Checkbox')
    && sources.runtimeSelectionControls.includes('SelectionControlKind::Radio')
    && sources.runtimeSelectionControls.includes('SelectionControlKind::Toggle')
    && sources.runtimeExtract.includes("selection_control_render_commands")
    && sources.runtimeExtract.includes("selection_control_suppresses_owner_text")
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
  sources.runtimeTestsMod.includes("mod render_selection_controls;")
    && sources.runtimeTestsMod.includes("mod render_sliders;")
    && sources.runtimeTestsMod.includes("mod render_dropdowns;")
    && sources.runtimeTestsMod.includes("mod render_text_fields;")
    && sources.runtimeTestsMod.includes("mod render_popup_options;")
    && sources.runtimeSelectionTest.includes("render_extract_expands_selection_control_indicators")
    && sources.runtimeSliderTest.includes("render_extract_expands_slider_primitives")
    && sources.runtimeDropdownTest.includes("render_extract_expands_dropdown_trigger_primitives")
    && sources.runtimeTextFieldTest.includes("render_extract_expands_text_field_primitives")
    && sources.runtimePopupOptionsTest.includes("render_extract_expands_open_dropdown_options")
]);
checks.push([
  "native component contract spans web and runtime",
  nativeComponentFamilies.length >= 15
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
  `native component contract: families=${nativeComponentFamilies.length} interactions=${nativeInteractionContracts.length} runtimeExtract=selection,slider,dropdown,text-field,popup`,
);
console.log("ok native component family contract");

function readLocal(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function readRepo(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function webSources() {
  return [sources.atoms, sources.collections, sources.surfaces, sources.moduleComponents].join("\n");
}

function fileSource(fileName) {
  return readRepo(`../../../../zircon_editor/src/ui/retained_host/host_contract/painter/${fileName}`);
}

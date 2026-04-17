use std::fs;
use std::path::PathBuf;

fn shell_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench.slint");
    fs::read_to_string(path).expect("workbench.slint should be readable")
}

fn panes_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/panes.slint");
    fs::read_to_string(path).expect("panes.slint should be readable")
}

fn block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker `{marker}` in workbench.slint"));
    &source[start..]
}

#[test]
fn shell_regions_bind_absolute_anchors_from_solver_frames() {
    let source = shell_source();

    let center_band = block_after(&source, "main_content_zone := Rectangle {");
    assert!(center_band.contains("x: root.center_band_frame.x * 1px;"));
    assert!(center_band.contains("y: root.center_band_frame.y * 1px;"));
    assert!(center_band.contains("width: root.center_band_frame.width * 1px;"));
    assert!(center_band.contains("height: root.center_band_frame.height * 1px;"));

    let right_region = block_after(&source, "right_stack_zone := Rectangle {");
    assert!(right_region.contains("x: root.right_region_frame.x * 1px;"));
    assert!(right_region.contains("y: root.right_region_frame.y * 1px;"));
    assert!(right_region.contains("width: root.right_region_frame.width * 1px;"));
    assert!(right_region.contains("height: root.right_region_frame.height * 1px;"));

    let bottom_region = block_after(&source, "bottom_panel_zone := Rectangle {");
    assert!(bottom_region.contains("x: root.bottom_region_frame.x * 1px;"));
    assert!(bottom_region.contains("y: root.bottom_region_frame.y * 1px;"));
    assert!(bottom_region.contains("width: root.bottom_region_frame.width * 1px;"));
    assert!(bottom_region.contains("height: root.bottom_region_frame.height * 1px;"));

    let status_bar = block_after(&source, "status_bar_zone := Rectangle {");
    assert!(status_bar.contains("x: root.status_bar_frame.x * 1px;"));
    assert!(status_bar.contains("y: root.status_bar_frame.y * 1px;"));
    assert!(status_bar.contains("width: root.status_bar_frame.width * 1px;"));
    assert!(status_bar.contains("height: root.status_bar_frame.height * 1px;"));
}

#[test]
fn workbench_shell_declares_native_resize_and_maximize_bounds() {
    let source = shell_source();
    let shell_block = block_after(&source, "export component WorkbenchShell inherits Window {");

    assert!(shell_block.contains("no-frame: false;"));
    assert!(shell_block.contains("resize-border-width: 8px;"));
    assert!(shell_block.contains("max-width:"));
    assert!(shell_block.contains("max-height:"));
}

#[test]
fn shell_drag_targets_allow_empty_tool_regions() {
    let source = shell_source();

    assert!(
        source.contains("callback workbench_drag_pointer_event(kind: int, x: float, y: float);")
    );
    assert!(
        source.contains("callback workbench_resize_pointer_event(kind: int, x: float, y: float);")
    );
    assert!(!source.contains("callback drop_tab(tab_id: string, target_group: string, pointer_x: float, pointer_y: float);"));
    assert!(!source.contains("callback update_drag_target(x: float, y: float);"));
    assert!(!source.contains("callback begin_drawer_resize(x: float, y: float);"));
    assert!(!source.contains("callback update_drawer_resize(x: float, y: float);"));
    assert!(!source.contains("callback finish_drawer_resize(x: float, y: float);"));

    assert!(source.contains("in-out property <string> active_drag_target_group: \"\";"));
    assert!(!source.contains("property <string> drag_target_group:"));

    let drag_overlay = block_after(&source, "if root.drag_active: Rectangle {");
    assert!(drag_overlay.contains("if root.left_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains("if root.right_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains("x: parent.width - root.right_drop_width + 8px;"));
    assert!(drag_overlay.contains("if root.bottom_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains(
        "root.workbench_drag_pointer_event(1, root.drag_pointer_x, root.drag_pointer_y);"
    ));
    assert!(drag_overlay.contains(
        "root.workbench_drag_pointer_event(2, root.drag_pointer_x, root.drag_pointer_y);"
    ));
    assert!(!drag_overlay
        .contains("root.update_drag_target(root.drag_pointer_x, root.drag_pointer_y);"));
    assert!(!drag_overlay.contains("root.drop_tab("));
    assert!(drag_overlay.contains("root.active_drag_target_group = \"\";"));

    let resize_overlay = block_after(&source, "if root.resize_active: TouchArea {");
    assert!(resize_overlay.contains(
        "root.workbench_resize_pointer_event(1, self.mouse-x / 1px, self.mouse-y / 1px);"
    ));
    assert!(resize_overlay.contains(
        "root.workbench_resize_pointer_event(2, self.mouse-x / 1px, self.mouse-y / 1px);"
    ));
    assert!(!resize_overlay.contains("root.update_drawer_resize("));
    assert!(!resize_overlay.contains("root.finish_drawer_resize("));
}

#[test]
fn drag_overlay_uses_pointer_following_ghost_preview_instead_of_centered_banner() {
    let source = shell_source();

    assert!(source.contains("drag_preview := Rectangle {"));
    assert!(source.contains("x: clamp(root.drag_pointer_x * 1px"));
    assert!(source.contains("y: clamp("));
    assert!(source.contains("root.drag_pointer_y * 1px - self.height - 14px"));
    assert!(!source.contains("animate x { duration: 42ms; }"));
    assert!(!source.contains("animate y { duration: 42ms; }"));
    assert!(!source.contains("x: (parent.width - 220px) / 2;"));
}

#[test]
fn menu_popups_anchor_to_actual_menu_buttons_instead_of_hardcoded_offsets() {
    let source = shell_source();
    assert!(source.contains("menu_button_row := HorizontalLayout {"));
    assert!(source.contains("file_menu_button := MenuBarButton"));
    assert!(source.contains("edit_menu_button := MenuBarButton"));
    assert!(source.contains("selection_menu_button := MenuBarButton"));
    assert!(source.contains("view_menu_button := MenuBarButton"));
    assert!(source.contains("window_menu_button := MenuBarButton"));
    assert!(source.contains("help_menu_button := MenuBarButton"));

    assert!(source.contains("out property <FrameRect> file_menu_button_local_frame: {"));
    assert!(source.contains("out property <FrameRect> edit_menu_button_local_frame: {"));
    assert!(source.contains("out property <FrameRect> selection_menu_button_local_frame: {"));
    assert!(source.contains("out property <FrameRect> view_menu_button_local_frame: {"));
    assert!(source.contains("out property <FrameRect> window_menu_button_local_frame: {"));
    assert!(source.contains("out property <FrameRect> help_menu_button_local_frame: {"));

    assert!(source.contains("x: menu_button_row.x / 1px + file_menu_button.x / 1px,"));
    assert!(source.contains("x: menu_button_row.x / 1px + edit_menu_button.x / 1px,"));
    assert!(source.contains("x: menu_button_row.x / 1px + selection_menu_button.x / 1px,"));
    assert!(source.contains("x: menu_button_row.x / 1px + view_menu_button.x / 1px,"));
    assert!(source.contains("x: menu_button_row.x / 1px + window_menu_button.x / 1px,"));
    assert!(source.contains("x: menu_button_row.x / 1px + help_menu_button.x / 1px,"));

    assert!(source.contains("if root.open_menu_index == 0: Rectangle {"));
    assert!(source.contains("x: root.file_menu_button_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 1: Rectangle {"));
    assert!(source.contains("x: root.edit_menu_button_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 2: Rectangle {"));
    assert!(source.contains("x: root.selection_menu_button_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 3: Rectangle {"));
    assert!(source.contains("x: root.view_menu_button_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 4: Rectangle {"));
    assert!(source.contains("x: root.window_menu_button_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 5: Rectangle {"));
    assert!(source.contains("x: root.help_menu_button_frame.x * 1px;"));

    assert!(!source.contains("x: root.outer_margin + 10px;"));
    assert!(!source.contains("x: root.outer_margin + 62px;"));
    assert!(!source.contains("x: root.outer_margin + 117px;"));
    assert!(!source.contains("x: root.outer_margin + 196px;"));
    assert!(!source.contains("x: root.outer_margin + 245px;"));
    assert!(!source.contains("x: root.outer_margin + 307px;"));
}

#[test]
fn menu_popups_use_local_top_bar_y_without_double_counting_outer_margin() {
    let source = shell_source();

    assert!(source.contains("property <length> popup_y: root.top_bar_height + 1px;"));
    assert!(!source
        .contains("property <length> popup_y: root.outer_margin + root.top_bar_height + 1px;"));
}

#[test]
fn drag_overlay_uses_grabbing_cursor_and_target_badge_for_clearer_feedback() {
    let source = shell_source();
    let drag_overlay = block_after(&source, "drag_capture := TouchArea {");
    assert!(drag_overlay.contains("mouse-cursor: MouseCursor.grabbing;"));

    let preview = block_after(&source, "drag_preview := Rectangle {");
    assert!(preview.contains("drag_target_badge := Rectangle {"));
    assert!(preview.contains(
        "text: root.active_drag_target_group != \"\" ? root.drag_target_label : \"Move Tab\";"
    ));
}

#[test]
fn drag_overlay_declares_document_edge_target_keys_and_highlights() {
    let source = shell_source();

    assert!(
        source.contains("root.active_drag_target_group == \"document-left\" ? \"Split Left\" :")
    );
    assert!(
        source.contains("root.active_drag_target_group == \"document-right\" ? \"Split Right\" :")
    );
    assert!(source.contains("root.active_drag_target_group == \"document-top\" ? \"Split Top\" :"));
    assert!(source
        .contains("root.active_drag_target_group == \"document-bottom\" ? \"Split Bottom\" :"));

    let drag_overlay = block_after(&source, "if root.drag_active: Rectangle {");
    assert!(drag_overlay.contains("root.active_drag_target_group == \"document-left\""));
    assert!(drag_overlay.contains("root.active_drag_target_group == \"document-right\""));
    assert!(drag_overlay.contains("root.active_drag_target_group == \"document-top\""));
    assert!(drag_overlay.contains("root.active_drag_target_group == \"document-bottom\""));
}

#[test]
fn floating_window_overlay_declares_projection_input_and_pane_surface_host() {
    let source = shell_source();

    assert!(source.contains("export struct FloatingWindowData {"));
    assert!(source.contains("in property <[FloatingWindowData]> floating_windows;"));
    assert!(source.contains("callback floating_window_header_pointer_clicked(x: float, y: float);"));

    let floating_overlay = block_after(
        &source,
        "for window[index] in root.floating_windows: floating_window_card := Rectangle {",
    );
    assert!(floating_overlay.contains("for tab[index] in window.tabs: floating_tab := TabChip {"));
    assert!(floating_overlay.contains("pointer_clicked(x, y) => {"));
    assert!(floating_overlay.contains("root.document_tab_pointer_clicked("));
    assert!(floating_overlay.contains("close_pointer_clicked(x, y) => {"));
    assert!(floating_overlay.contains("root.document_tab_close_pointer_clicked("));
    assert!(floating_overlay.contains("header_touch := TouchArea {"));
    assert!(floating_overlay.contains("root.floating_window_header_pointer_clicked("));
    assert!(floating_overlay.contains("pane: window.active_pane;"));
}

#[test]
fn floating_window_overlay_consumes_projected_frame_and_route_keys() {
    let source = shell_source();

    assert!(source.contains("frame: FrameRect,"));
    assert!(source.contains("target_group: string,"));
    assert!(source.contains("left_edge_target_group: string,"));
    assert!(source.contains("right_edge_target_group: string,"));
    assert!(source.contains("top_edge_target_group: string,"));
    assert!(source.contains("bottom_edge_target_group: string,"));
    assert!(source.contains("focus_target_id: string,"));

    let floating_overlay = block_after(
        &source,
        "for window[index] in root.floating_windows: floating_window_card := Rectangle {",
    );
    assert!(floating_overlay.contains("x: window.frame.x * 1px;"));
    assert!(floating_overlay.contains("y: window.frame.y * 1px;"));
    assert!(floating_overlay.contains("width: window.frame.width * 1px;"));
    assert!(floating_overlay.contains("height: window.frame.height * 1px;"));
    assert!(!floating_overlay.contains("index * 26px"));
    assert!(!floating_overlay.contains("index * 22px"));
}

#[test]
fn assets_activity_pane_stays_lightweight_and_browser_keeps_advanced_tools() {
    let source = shell_source();
    let assets_source_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/assets.slint");
    let assets_source =
        fs::read_to_string(assets_source_path).expect("assets.slint should be readable");

    let activity_start = assets_source
        .find("export component AssetsActivityPane inherits Rectangle {")
        .expect("missing AssetsActivityPane block");
    let browser_start = assets_source
        .find("export component AssetBrowserPane inherits Rectangle {")
        .expect("missing AssetBrowserPane block");
    let activity_block = &assets_source[activity_start..browser_start];
    assert!(!activity_block
        .contains("Unity-first activity panel for project browsing and quick preview"));
    assert!(!activity_block.contains("Quick Import"));
    assert!(!activity_block.contains("label: \"Metadata\""));
    assert!(!activity_block.contains("label: \"Plugins\""));
    assert!(activity_block.contains("label: \"Browser\""));
    assert!(activity_block.contains("label: \"Preview\""));
    assert!(activity_block.contains("label: \"References\""));

    let browser_block = &assets_source[browser_start..];
    assert!(browser_block.contains("Quick Import"));
    assert!(browser_block.contains("label: \"Metadata\""));
    assert!(browser_block.contains("label: \"Plugins\""));

    assert!(source.contains("import { AssetBrowserPane"));
}

#[test]
fn assets_surfaces_use_responsive_utility_height_constraints() {
    let assets_source_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/assets.slint");
    let assets_source =
        fs::read_to_string(assets_source_path).expect("assets.slint should be readable");

    assert!(assets_source.contains(
        "private property <length> utility_height: min(max(22% * root.height, 132px), 176px);"
    ));
    assert!(assets_source.contains(
        "private property <length> utility_height: min(max(24% * root.height, 176px), 240px);"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_interactive_callbacks_and_multiline_source_editor() {
    let source = shell_source();
    let panes = panes_source();

    assert!(source.contains("callback ui_asset_action(instance_id: string, action_id: string);"));
    assert!(source.contains("callback ui_asset_source_edited(instance_id: string, value: string);"));
    assert!(source
        .contains("callback ui_asset_hierarchy_selected(instance_id: string, item_index: int);"));
    assert!(source
        .contains("callback ui_asset_preview_selected(instance_id: string, item_index: int);"));

    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    assert!(pane_surface.contains("instance_id: root.pane.id;"));
    assert!(pane_surface
        .contains("action(action_id) => { root.ui_asset_action(root.pane.id, action_id); }"));
    assert!(pane_surface
        .contains("source_edited(value) => { root.ui_asset_source_edited(root.pane.id, value); }"));
    assert!(pane_surface.contains("hierarchy_selected(item_index) => { root.ui_asset_hierarchy_selected(root.pane.id, item_index); }"));
    assert!(pane_surface.contains("preview_selected(item_index) => { root.ui_asset_preview_selected(root.pane.id, item_index); }"));

    assert!(panes.contains("import { LineEdit, TextEdit } from \"std-widgets.slint\";"));
    assert!(panes.contains("callback action(action_id: string);"));
    assert!(panes.contains("callback source_edited(value: string);"));
    assert!(panes.contains("callback hierarchy_selected(item_index: int);"));
    assert!(panes.contains("callback preview_selected(item_index: int);"));
    assert!(panes.contains("TextEdit {"));
}

#[test]
fn ui_asset_editor_pane_declares_open_reference_action_and_state_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_can_open_reference: bool,"));
    assert!(pane_surface.contains("can_open_reference: root.pane.ui_asset_can_open_reference;"));

    assert!(pane_block.contains("in property <bool> can_open_reference;"));
    assert!(pane_block.contains("label: \"Open Ref\";"));
    assert!(pane_block.contains("enabled: root.can_open_reference;"));
    assert!(pane_block.contains("active: root.can_open_reference;"));
    assert!(pane_block.contains("root.action(\"reference.open\");"));
}

#[test]
fn ui_asset_editor_pane_declares_preview_preset_controls_and_state_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_preview_preset: string,"));
    assert!(pane_surface.contains("preview_preset: root.pane.ui_asset_preview_preset;"));

    assert!(pane_block.contains("in property <string> preview_preset;"));
    assert!(pane_block.contains("label: \"Docked\";"));
    assert!(pane_block.contains("active: root.preview_preset == \"Editor Docked\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.editor_docked\");"));
    assert!(pane_block.contains("label: \"Float\";"));
    assert!(pane_block.contains("active: root.preview_preset == \"Editor Floating\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.editor_floating\");"));
    assert!(pane_block.contains("label: \"HUD\";"));
    assert!(pane_block.contains("active: root.preview_preset == \"Game HUD\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.game_hud\");"));
    assert!(pane_block.contains("label: \"Dialog\";"));
    assert!(pane_block.contains("active: root.preview_preset == \"Dialog\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.dialog\");"));
}

#[test]
fn ui_asset_editor_pane_declares_slot_aware_external_palette_drop_overlays() {
    let panes = panes_source();
    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(canvas_block.contains("root.external_drag_target_action == \"palette.insert.child\""));
    assert!(canvas_block.contains("root.external_drag_target_action == \"palette.insert.after\""));
    assert!(canvas_block.contains("text: root.external_drag_target_label;"));
    assert!(canvas_block.contains("drop_inside_overlay := Rectangle {"));
    assert!(canvas_block.contains("drop_after_overlay := Rectangle {"));
}

#[test]
fn ui_asset_editor_pane_declares_explicit_palette_slot_target_overlay_projection() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(panes.contains("export struct UiAssetCanvasSlotTargetData {"));
    assert!(source.contains("UiAssetCanvasSlotTargetData"));
    assert!(
        source.contains("ui_asset_palette_drag_slot_target_items: [UiAssetCanvasSlotTargetData],")
    );
    assert!(pane_surface.contains(
        "palette_drag_slot_target_items: root.pane.ui_asset_palette_drag_slot_target_items;"
    ));

    assert!(pane_block
        .contains("in property <[UiAssetCanvasSlotTargetData]> palette_drag_slot_target_items;"));
    assert!(pane_block.contains("external_slot_target_items: root.palette_drag_slot_target_items;"));

    assert!(canvas_block
        .contains("in property <[UiAssetCanvasSlotTargetData]> external_slot_target_items;"));
    assert!(canvas_block.contains(
        "for target[index] in root.external_slot_target_items: external_slot_target := Rectangle {"
    ));
    assert!(canvas_block.contains("target.selected ? 2px : 1px;"));
    assert!(canvas_block.contains("target.label"));
    assert!(canvas_block.contains("root.external_slot_target_items.length == 0"));
}

#[test]
fn ui_asset_editor_pane_declares_mock_preview_controls_and_callbacks() {
    let source = shell_source();
    let panes = panes_source();
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(source.contains("ui_asset_preview_mock_items: [string],"));
    assert!(source.contains("ui_asset_preview_mock_selected_index: int,"));
    assert!(source.contains("ui_asset_preview_mock_property: string,"));
    assert!(source.contains("ui_asset_preview_mock_kind: string,"));
    assert!(source.contains("ui_asset_preview_mock_value: string,"));
    assert!(source.contains("ui_asset_preview_mock_can_edit: bool,"));
    assert!(source.contains("ui_asset_preview_mock_can_clear: bool,"));
    assert!(source.contains(
        "callback ui_asset_preview_mock_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains(
        "callback ui_asset_preview_mock_action(instance_id: string, action_id: string, value: string);"
    ));
    assert!(pane_surface.contains("preview_mock_items: root.pane.ui_asset_preview_mock_items;"));
    assert!(pane_surface
        .contains("preview_mock_selected_index: root.pane.ui_asset_preview_mock_selected_index;"));
    assert!(
        pane_surface.contains("preview_mock_property: root.pane.ui_asset_preview_mock_property;")
    );
    assert!(pane_surface.contains("preview_mock_kind: root.pane.ui_asset_preview_mock_kind;"));
    assert!(pane_surface.contains("preview_mock_value: root.pane.ui_asset_preview_mock_value;"));
    assert!(
        pane_surface.contains("preview_mock_can_edit: root.pane.ui_asset_preview_mock_can_edit;")
    );
    assert!(
        pane_surface.contains("preview_mock_can_clear: root.pane.ui_asset_preview_mock_can_clear;")
    );
    assert!(pane_surface.contains(
        "preview_mock_selected(item_index) => { root.ui_asset_preview_mock_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "preview_mock_action(action_id, value) => { root.ui_asset_preview_mock_action(root.pane.id, action_id, value); }"
    ));

    assert!(pane_block.contains("in property <[string]> preview_mock_items;"));
    assert!(pane_block.contains("in property <int> preview_mock_selected_index;"));
    assert!(pane_block.contains("in property <string> preview_mock_property;"));
    assert!(pane_block.contains("in property <string> preview_mock_kind;"));
    assert!(pane_block.contains("in property <string> preview_mock_value;"));
    assert!(pane_block.contains("in property <bool> preview_mock_can_edit;"));
    assert!(pane_block.contains("in property <bool> preview_mock_can_clear;"));
    assert!(pane_block.contains("callback preview_mock_selected(item_index: int);"));
    assert!(pane_block.contains("callback preview_mock_action(action_id: string, value: string);"));
    assert!(panes.contains("title: \"Mock Preview\";"));
    assert!(panes.contains("items: root.preview_mock_items;"));
    assert!(panes.contains("selected_index: root.preview_mock_selected_index;"));
    assert!(panes.contains("root.preview_mock_selected(item_index);"));
    assert!(panes.contains("text: root.preview_mock_property;"));
    assert!(panes.contains("text: root.preview_mock_kind;"));
    assert!(panes.contains("root.preview_mock_value;"));
    assert!(panes.contains("root.preview_mock_action(\"preview.mock.value.set\""));
    assert!(panes.contains("root.preview_mock_action(\"preview.mock.clear\", \"\");"));
}

#[test]
fn ui_asset_editor_pane_declares_style_authoring_buttons_and_state_bindings() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_can_create_rule: bool,"));
    assert!(source.contains("ui_asset_can_extract_rule: bool,"));
    assert!(source.contains("ui_asset_style_state_hover: bool,"));
    assert!(source.contains("ui_asset_style_state_focus: bool,"));
    assert!(source.contains("ui_asset_style_state_pressed: bool,"));
    assert!(source.contains("ui_asset_style_state_disabled: bool,"));
    assert!(source.contains("ui_asset_style_state_selected: bool,"));

    assert!(pane_surface.contains("can_create_rule: root.pane.ui_asset_can_create_rule;"));
    assert!(pane_surface.contains("can_extract_rule: root.pane.ui_asset_can_extract_rule;"));
    assert!(pane_surface.contains("state_hover: root.pane.ui_asset_style_state_hover;"));
    assert!(pane_surface.contains("state_focus: root.pane.ui_asset_style_state_focus;"));
    assert!(pane_surface.contains("state_pressed: root.pane.ui_asset_style_state_pressed;"));
    assert!(pane_surface.contains("state_disabled: root.pane.ui_asset_style_state_disabled;"));
    assert!(pane_surface.contains("state_selected: root.pane.ui_asset_style_state_selected;"));

    assert!(pane_block.contains("in property <bool> can_create_rule;"));
    assert!(pane_block.contains("in property <bool> can_extract_rule;"));
    assert!(pane_block.contains("in property <bool> state_hover;"));
    assert!(pane_block.contains("in property <bool> state_focus;"));
    assert!(pane_block.contains("in property <bool> state_pressed;"));
    assert!(pane_block.contains("in property <bool> state_disabled;"));
    assert!(pane_block.contains("in property <bool> state_selected;"));
    assert!(pane_block.contains("label: \"Rule\";"));
    assert!(pane_block.contains("root.action(\"style.rule.create\");"));
    assert!(pane_block.contains("label: \"Extract\";"));
    assert!(pane_block.contains("root.action(\"style.rule.extract_inline\");"));
    assert!(pane_block.contains("label: \"Hover\";"));
    assert!(pane_block.contains("root.action(\"style.state.hover\");"));
    assert!(pane_block.contains("label: \"Focus\";"));
    assert!(pane_block.contains("root.action(\"style.state.focus\");"));
    assert!(pane_block.contains("label: \"Pressed\";"));
    assert!(pane_block.contains("root.action(\"style.state.pressed\");"));
    assert!(pane_block.contains("label: \"Disabled\";"));
    assert!(pane_block.contains("root.action(\"style.state.disabled\");"));
    assert!(pane_block.contains("label: \"Selected\";"));
    assert!(pane_block.contains("root.action(\"style.state.selected\");"));
}

#[test]
fn ui_asset_editor_pane_declares_style_class_authoring_controls_and_callback() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_style_class_action(instance_id: string, action_id: string, class_name: string);"
    ));
    assert!(source.contains("ui_asset_style_class_items: [string],"));
    assert!(pane_surface.contains("style_class_items: root.pane.ui_asset_style_class_items;"));
    assert!(pane_surface.contains(
        "style_class_action(action_id, class_name) => { root.ui_asset_style_class_action(root.pane.id, action_id, class_name); }"
    ));

    assert!(pane_block.contains("in property <[string]> style_class_items;"));
    assert!(
        pane_block.contains("callback style_class_action(action_id: string, class_name: string);")
    );
    assert!(panes.contains("property <string> style_class_draft: \"\";"));
    assert!(panes.contains("for class_name in root.style_class_items: Text {"));
    assert!(panes.contains("label: \"Add\";"));
    assert!(panes.contains("root.style_class_action(\"style.class.add\", root.style_class_draft);"));
    assert!(panes.contains("label: \"Remove\";"));
    assert!(
        panes.contains("root.style_class_action(\"style.class.remove\", root.style_class_draft);")
    );
}

#[test]
fn ui_asset_editor_pane_declares_style_rule_editing_controls_and_callback() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_style_rule_action(instance_id: string, action_id: string, item_index: int, selector: string);"
    ));
    assert!(source.contains("ui_asset_style_rule_items: [string],"));
    assert!(source.contains("ui_asset_style_rule_selected_index: int,"));
    assert!(source.contains("ui_asset_style_selected_rule_selector: string,"));
    assert!(source.contains("ui_asset_style_can_edit_rule: bool,"));
    assert!(source.contains("ui_asset_style_can_delete_rule: bool,"));

    assert!(pane_surface.contains("style_rule_items: root.pane.ui_asset_style_rule_items;"));
    assert!(pane_surface
        .contains("style_rule_selected_index: root.pane.ui_asset_style_rule_selected_index;"));
    assert!(pane_surface
        .contains("selected_rule_selector: root.pane.ui_asset_style_selected_rule_selector;"));
    assert!(pane_surface.contains("can_edit_rule: root.pane.ui_asset_style_can_edit_rule;"));
    assert!(pane_surface.contains("can_delete_rule: root.pane.ui_asset_style_can_delete_rule;"));
    assert!(pane_surface.contains(
        "style_rule_action(action_id, item_index, selector) => { root.ui_asset_style_rule_action(root.pane.id, action_id, item_index, selector); }"
    ));

    assert!(pane_block.contains("in property <[string]> style_rule_items;"));
    assert!(pane_block.contains("in property <int> style_rule_selected_index;"));
    assert!(pane_block.contains("in property <string> selected_rule_selector;"));
    assert!(pane_block.contains("in property <bool> can_edit_rule;"));
    assert!(pane_block.contains("in property <bool> can_delete_rule;"));
    assert!(pane_block.contains(
        "callback style_rule_action(action_id: string, item_index: int, selector: string);"
    ));

    assert!(panes.contains("property <string> style_rule_selector_draft: \"\";"));
    assert!(panes.contains("title: \"Rules\";"));
    assert!(panes.contains("items: root.style_rule_items;"));
    assert!(panes.contains("selected_index: root.style_rule_selected_index;"));
    assert!(panes.contains("root.style_rule_selector_draft = root.style_rule_items[item_index];"));
    assert!(panes.contains("root.style_rule_action(\"style.rule.select\", item_index, \"\");"));
    assert!(panes.contains("placeholder: \"selector\";"));
    assert!(panes.contains("label: \"Apply\";"));
    assert!(panes.contains(
        "root.style_rule_action(\"style.rule.rename\", root.style_rule_selected_index, root.style_rule_selector_draft != \"\" ? root.style_rule_selector_draft : root.selected_rule_selector);"
    ));
    assert!(panes.contains("label: \"Delete\";"));
    assert!(panes.contains(
        "root.style_rule_action(\"style.rule.delete\", root.style_rule_selected_index, \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_style_token_editing_controls_and_callback() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_style_token_action(instance_id: string, action_id: string, item_index: int, token_name: string, token_value: string);"
    ));
    assert!(source.contains("ui_asset_style_token_items: [string],"));
    assert!(source.contains("ui_asset_style_token_selected_index: int,"));
    assert!(source.contains("ui_asset_style_selected_token_name: string,"));
    assert!(source.contains("ui_asset_style_selected_token_value: string,"));
    assert!(source.contains("ui_asset_style_can_edit_token: bool,"));
    assert!(source.contains("ui_asset_style_can_delete_token: bool,"));

    assert!(pane_surface.contains("style_token_items: root.pane.ui_asset_style_token_items;"));
    assert!(pane_surface
        .contains("style_token_selected_index: root.pane.ui_asset_style_token_selected_index;"));
    assert!(
        pane_surface.contains("selected_token_name: root.pane.ui_asset_style_selected_token_name;")
    );
    assert!(pane_surface
        .contains("selected_token_value: root.pane.ui_asset_style_selected_token_value;"));
    assert!(pane_surface.contains("can_edit_token: root.pane.ui_asset_style_can_edit_token;"));
    assert!(pane_surface.contains("can_delete_token: root.pane.ui_asset_style_can_delete_token;"));
    assert!(pane_surface.contains(
        "style_token_action(action_id, item_index, token_name, token_value) => { root.ui_asset_style_token_action(root.pane.id, action_id, item_index, token_name, token_value); }"
    ));

    assert!(pane_block.contains("in property <[string]> style_token_items;"));
    assert!(pane_block.contains("in property <int> style_token_selected_index;"));
    assert!(pane_block.contains("in property <string> selected_token_name;"));
    assert!(pane_block.contains("in property <string> selected_token_value;"));
    assert!(pane_block.contains("in property <bool> can_edit_token;"));
    assert!(pane_block.contains("in property <bool> can_delete_token;"));
    assert!(pane_block.contains(
        "callback style_token_action(action_id: string, item_index: int, token_name: string, token_value: string);"
    ));

    assert!(panes.contains("property <string> style_token_name_draft: \"\";"));
    assert!(panes.contains("property <string> style_token_value_draft: \"\";"));
    assert!(panes.contains("title: \"Tokens\";"));
    assert!(panes.contains("items: root.style_token_items;"));
    assert!(panes.contains("selected_index: root.style_token_selected_index;"));
    assert!(panes.contains("root.style_token_name_draft = root.selected_token_name;"));
    assert!(panes.contains("root.style_token_value_draft = root.selected_token_value;"));
    assert!(panes.contains("placeholder: \"token-name\";"));
    assert!(panes.contains("placeholder: \"token-value\";"));
    assert!(panes.contains("label: \"Apply\";"));
    assert!(panes.contains(
        "root.style_token_action(\"style.token.upsert\", root.style_token_selected_index, root.style_token_name_draft != \"\" ? root.style_token_name_draft : root.selected_token_name, root.style_token_value_draft != \"\" ? root.style_token_value_draft : root.selected_token_value);"
    ));
    assert!(panes.contains("label: \"Delete\";"));
    assert!(panes.contains(
        "root.style_token_action(\"style.token.delete\", root.style_token_selected_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_style_rule_declaration_editing_controls_and_callback() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_style_rule_declaration_action(instance_id: string, action_id: string, item_index: int, declaration_path: string, declaration_value: string);"
    ));
    assert!(source.contains("ui_asset_style_rule_declaration_items: [string],"));
    assert!(source.contains("ui_asset_style_rule_declaration_selected_index: int,"));
    assert!(source.contains("ui_asset_style_selected_rule_declaration_path: string,"));
    assert!(source.contains("ui_asset_style_selected_rule_declaration_value: string,"));
    assert!(source.contains("ui_asset_style_can_edit_rule_declaration: bool,"));
    assert!(source.contains("ui_asset_style_can_delete_rule_declaration: bool,"));

    assert!(pane_surface.contains(
        "style_rule_declaration_items: root.pane.ui_asset_style_rule_declaration_items;"
    ));
    assert!(pane_surface.contains(
        "style_rule_declaration_selected_index: root.pane.ui_asset_style_rule_declaration_selected_index;"
    ));
    assert!(pane_surface.contains(
        "selected_rule_declaration_path: root.pane.ui_asset_style_selected_rule_declaration_path;"
    ));
    assert!(pane_surface.contains(
        "selected_rule_declaration_value: root.pane.ui_asset_style_selected_rule_declaration_value;"
    ));
    assert!(pane_surface.contains(
        "can_edit_rule_declaration: root.pane.ui_asset_style_can_edit_rule_declaration;"
    ));
    assert!(pane_surface.contains(
        "can_delete_rule_declaration: root.pane.ui_asset_style_can_delete_rule_declaration;"
    ));
    assert!(pane_surface.contains(
        "style_rule_declaration_action(action_id, item_index, declaration_path, declaration_value) => { root.ui_asset_style_rule_declaration_action(root.pane.id, action_id, item_index, declaration_path, declaration_value); }"
    ));

    assert!(pane_block.contains("in property <[string]> style_rule_declaration_items;"));
    assert!(pane_block.contains("in property <int> style_rule_declaration_selected_index;"));
    assert!(pane_block.contains("in property <string> selected_rule_declaration_path;"));
    assert!(pane_block.contains("in property <string> selected_rule_declaration_value;"));
    assert!(pane_block.contains("in property <bool> can_edit_rule_declaration;"));
    assert!(pane_block.contains("in property <bool> can_delete_rule_declaration;"));
    assert!(pane_block.contains(
        "callback style_rule_declaration_action(action_id: string, item_index: int, declaration_path: string, declaration_value: string);"
    ));

    assert!(panes.contains("property <string> style_rule_declaration_path_draft: \"\";"));
    assert!(panes.contains("property <string> style_rule_declaration_value_draft: \"\";"));
    assert!(panes.contains("title: \"Declarations\";"));
    assert!(panes.contains("items: root.style_rule_declaration_items;"));
    assert!(panes.contains("selected_index: root.style_rule_declaration_selected_index;"));
    assert!(panes.contains(
        "root.style_rule_declaration_action(\"style.rule.declaration.select\", item_index, \"\", \"\");"
    ));
    assert!(panes
        .contains("root.style_rule_declaration_path_draft = root.selected_rule_declaration_path;"));
    assert!(panes.contains(
        "root.style_rule_declaration_value_draft = root.selected_rule_declaration_value;"
    ));
    assert!(panes.contains("placeholder: \"self.background.color\";"));
    assert!(panes.contains("placeholder: \"value\";"));
    assert!(panes.contains(
        "root.style_rule_declaration_action(\"style.rule.declaration.upsert\", root.style_rule_declaration_selected_index, root.style_rule_declaration_path_draft != \"\" ? root.style_rule_declaration_path_draft : root.selected_rule_declaration_path, root.style_rule_declaration_value_draft != \"\" ? root.style_rule_declaration_value_draft : root.selected_rule_declaration_value);"
    ));
    assert!(panes.contains(
        "root.style_rule_declaration_action(\"style.rule.declaration.delete\", root.style_rule_declaration_selected_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_matched_rule_inspection_controls_and_callback() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_matched_style_rule_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains("ui_asset_style_matched_rule_items: [string],"));
    assert!(source.contains("ui_asset_style_matched_rule_selected_index: int,"));
    assert!(source.contains("ui_asset_style_selected_matched_rule_origin: string,"));
    assert!(source.contains("ui_asset_style_selected_matched_rule_selector: string,"));
    assert!(source.contains("ui_asset_style_selected_matched_rule_specificity: int,"));
    assert!(source.contains("ui_asset_style_selected_matched_rule_source_order: int,"));
    assert!(source.contains("ui_asset_style_selected_matched_rule_declaration_items: [string],"));

    assert!(pane_surface
        .contains("style_matched_rule_items: root.pane.ui_asset_style_matched_rule_items;"));
    assert!(pane_surface.contains(
        "style_matched_rule_selected_index: root.pane.ui_asset_style_matched_rule_selected_index;"
    ));
    assert!(pane_surface.contains(
        "selected_matched_rule_origin: root.pane.ui_asset_style_selected_matched_rule_origin;"
    ));
    assert!(pane_surface.contains(
        "selected_matched_rule_selector: root.pane.ui_asset_style_selected_matched_rule_selector;"
    ));
    assert!(pane_surface.contains(
        "selected_matched_rule_specificity: root.pane.ui_asset_style_selected_matched_rule_specificity;"
    ));
    assert!(pane_surface.contains(
        "selected_matched_rule_source_order: root.pane.ui_asset_style_selected_matched_rule_source_order;"
    ));
    assert!(pane_surface.contains(
        "selected_matched_rule_declaration_items: root.pane.ui_asset_style_selected_matched_rule_declaration_items;"
    ));
    assert!(pane_surface.contains(
        "matched_style_rule_selected(item_index) => { root.ui_asset_matched_style_rule_selected(root.pane.id, item_index); }"
    ));

    assert!(pane_block.contains("in property <[string]> style_matched_rule_items;"));
    assert!(pane_block.contains("in property <int> style_matched_rule_selected_index;"));
    assert!(pane_block.contains("in property <string> selected_matched_rule_origin;"));
    assert!(pane_block.contains("in property <string> selected_matched_rule_selector;"));
    assert!(pane_block.contains("in property <int> selected_matched_rule_specificity;"));
    assert!(pane_block.contains("in property <int> selected_matched_rule_source_order;"));
    assert!(pane_block.contains("in property <[string]> selected_matched_rule_declaration_items;"));
    assert!(pane_block.contains("callback matched_style_rule_selected(item_index: int);"));

    assert!(panes.contains("title: \"Matched Rules\";"));
    assert!(panes.contains("items: root.style_matched_rule_items;"));
    assert!(panes.contains("selected_index: root.style_matched_rule_selected_index;"));
    assert!(panes.contains("matched_style_rule_selected(item_index) => {"));
    assert!(panes.contains("text: root.selected_matched_rule_origin != \"\" ? root.selected_matched_rule_origin : \"No matched rule selected\";"));
    assert!(panes.contains("text: root.selected_matched_rule_selector;"));
    assert!(panes.contains("text: root.selected_matched_rule_specificity >= 0 ? \"specificity \" + root.selected_matched_rule_specificity + \" • order \" + root.selected_matched_rule_source_order : \"\";"));
    assert!(panes.contains("for item in root.selected_matched_rule_declaration_items: Text {"));
}

#[test]
fn ui_asset_editor_pane_declares_widget_inspector_editing_controls_and_callback() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_inspector_widget_action(instance_id: string, action_id: string, value: string);"
    ));
    assert!(source.contains("ui_asset_inspector_selected_node_id: string,"));
    assert!(source.contains("ui_asset_inspector_parent_node_id: string,"));
    assert!(source.contains("ui_asset_inspector_mount: string,"));
    assert!(source.contains("ui_asset_inspector_widget_kind: string,"));
    assert!(source.contains("ui_asset_inspector_widget_label: string,"));
    assert!(source.contains("ui_asset_inspector_control_id: string,"));
    assert!(source.contains("ui_asset_inspector_text_prop: string,"));
    assert!(source.contains("ui_asset_inspector_can_edit_control_id: bool,"));
    assert!(source.contains("ui_asset_inspector_can_edit_text_prop: bool,"));

    assert!(pane_surface
        .contains("inspector_selected_node_id: root.pane.ui_asset_inspector_selected_node_id;"));
    assert!(pane_surface
        .contains("inspector_parent_node_id: root.pane.ui_asset_inspector_parent_node_id;"));
    assert!(pane_surface.contains("inspector_mount: root.pane.ui_asset_inspector_mount;"));
    assert!(
        pane_surface.contains("inspector_widget_kind: root.pane.ui_asset_inspector_widget_kind;")
    );
    assert!(
        pane_surface.contains("inspector_widget_label: root.pane.ui_asset_inspector_widget_label;")
    );
    assert!(pane_surface.contains("inspector_control_id: root.pane.ui_asset_inspector_control_id;"));
    assert!(pane_surface.contains("inspector_text_prop: root.pane.ui_asset_inspector_text_prop;"));
    assert!(pane_surface.contains(
        "inspector_can_edit_control_id: root.pane.ui_asset_inspector_can_edit_control_id;"
    ));
    assert!(pane_surface.contains(
        "inspector_can_edit_text_prop: root.pane.ui_asset_inspector_can_edit_text_prop;"
    ));
    assert!(pane_surface.contains(
        "inspector_widget_action(action_id, value) => { root.ui_asset_inspector_widget_action(root.pane.id, action_id, value); }"
    ));

    assert!(pane_block.contains("in property <string> inspector_selected_node_id;"));
    assert!(pane_block.contains("in property <string> inspector_parent_node_id;"));
    assert!(pane_block.contains("in property <string> inspector_mount;"));
    assert!(pane_block.contains("in property <string> inspector_widget_kind;"));
    assert!(pane_block.contains("in property <string> inspector_widget_label;"));
    assert!(pane_block.contains("in property <string> inspector_control_id;"));
    assert!(pane_block.contains("in property <string> inspector_text_prop;"));
    assert!(pane_block.contains("in property <bool> inspector_can_edit_control_id;"));
    assert!(pane_block.contains("in property <bool> inspector_can_edit_text_prop;"));
    assert!(
        pane_block.contains("callback inspector_widget_action(action_id: string, value: string);")
    );

    assert!(panes.contains("text: \"Widget\";"));
    assert!(panes.contains("text: \"Node\";"));
    assert!(panes.contains("text: \"Control Id\";"));
    assert!(panes.contains("text: \"Text\";"));
    assert!(panes.contains("root.inspector_widget_action(\"widget.control_id.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"widget.text.set\", value);"));
}

#[test]
fn ui_asset_editor_pane_declares_slot_inspector_editing_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_inspector_slot_padding: string,"));
    assert!(source.contains("ui_asset_inspector_slot_width_preferred: string,"));
    assert!(source.contains("ui_asset_inspector_slot_height_preferred: string,"));

    assert!(
        pane_surface.contains("inspector_slot_padding: root.pane.ui_asset_inspector_slot_padding;")
    );
    assert!(pane_surface.contains(
        "inspector_slot_width_preferred: root.pane.ui_asset_inspector_slot_width_preferred;"
    ));
    assert!(pane_surface.contains(
        "inspector_slot_height_preferred: root.pane.ui_asset_inspector_slot_height_preferred;"
    ));

    assert!(pane_block.contains("in property <string> inspector_slot_padding;"));
    assert!(pane_block.contains("in property <string> inspector_slot_width_preferred;"));
    assert!(pane_block.contains("in property <string> inspector_slot_height_preferred;"));

    assert!(panes.contains("text: \"Slot\";"));
    assert!(panes.contains("text: \"Mount\";"));
    assert!(panes.contains("text: \"Padding\";"));
    assert!(panes.contains("text: \"Width\";"));
    assert!(panes.contains("text: \"Height\";"));
    assert!(panes.contains("root.inspector_widget_action(\"slot.mount.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"slot.padding.set\", value);"));
    assert!(
        panes.contains("root.inspector_widget_action(\"slot.layout.width.preferred.set\", value);")
    );
    assert!(panes
        .contains("root.inspector_widget_action(\"slot.layout.height.preferred.set\", value);"));
}

#[test]
fn ui_asset_editor_pane_declares_layout_inspector_editing_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_inspector_layout_width_preferred: string,"));
    assert!(source.contains("ui_asset_inspector_layout_height_preferred: string,"));

    assert!(pane_surface.contains(
        "inspector_layout_width_preferred: root.pane.ui_asset_inspector_layout_width_preferred;"
    ));
    assert!(pane_surface.contains(
        "inspector_layout_height_preferred: root.pane.ui_asset_inspector_layout_height_preferred;"
    ));

    assert!(pane_block.contains("in property <string> inspector_layout_width_preferred;"));
    assert!(pane_block.contains("in property <string> inspector_layout_height_preferred;"));

    assert!(panes.contains("text: \"Layout\";"));
    assert!(panes.contains("root.inspector_widget_action(\"layout.width.preferred.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"layout.height.preferred.set\", value);"));
}

#[test]
fn ui_asset_editor_pane_declares_parent_specific_semantic_inspector_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_inspector_slot_semantic_title: string,"));
    assert!(source.contains("ui_asset_inspector_slot_semantic_items: [string],"));
    assert!(source.contains("ui_asset_inspector_slot_semantic_selected_index: int,"));
    assert!(source.contains("ui_asset_inspector_slot_semantic_path: string,"));
    assert!(source.contains("ui_asset_inspector_slot_semantic_value: string,"));
    assert!(source.contains("ui_asset_inspector_layout_semantic_title: string,"));
    assert!(source.contains("ui_asset_inspector_layout_semantic_items: [string],"));
    assert!(source.contains("ui_asset_inspector_layout_semantic_selected_index: int,"));
    assert!(source.contains("ui_asset_inspector_layout_semantic_path: string,"));
    assert!(source.contains("ui_asset_inspector_layout_semantic_value: string,"));
    assert!(source.contains(
        "callback ui_asset_slot_semantic_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains(
        "callback ui_asset_layout_semantic_selected(instance_id: string, item_index: int);"
    ));

    assert!(pane_surface.contains(
        "inspector_slot_semantic_title: root.pane.ui_asset_inspector_slot_semantic_title;"
    ));
    assert!(pane_surface.contains(
        "inspector_slot_semantic_items: root.pane.ui_asset_inspector_slot_semantic_items;"
    ));
    assert!(pane_surface.contains(
        "slot_semantic_selected(item_index) => { root.ui_asset_slot_semantic_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "layout_semantic_selected(item_index) => { root.ui_asset_layout_semantic_selected(root.pane.id, item_index); }"
    ));

    assert!(pane_block.contains("in property <string> inspector_slot_semantic_title;"));
    assert!(pane_block.contains("in property <[string]> inspector_slot_semantic_items;"));
    assert!(pane_block.contains("in property <int> inspector_slot_semantic_selected_index;"));
    assert!(pane_block.contains("in property <string> inspector_slot_semantic_path;"));
    assert!(pane_block.contains("in property <string> inspector_slot_semantic_value;"));
    assert!(pane_block.contains("in property <string> inspector_layout_semantic_title;"));
    assert!(pane_block.contains("in property <[string]> inspector_layout_semantic_items;"));
    assert!(pane_block.contains("in property <int> inspector_layout_semantic_selected_index;"));
    assert!(pane_block.contains("in property <string> inspector_layout_semantic_path;"));
    assert!(pane_block.contains("in property <string> inspector_layout_semantic_value;"));
    assert!(pane_block.contains("callback slot_semantic_selected(item_index: int);"));
    assert!(pane_block.contains("callback layout_semantic_selected(item_index: int);"));

    assert!(panes.contains("text: root.inspector_slot_semantic_title;"));
    assert!(panes.contains("text: root.inspector_layout_semantic_title;"));
    assert!(panes.contains("root.slot_semantic_selected(item_index);"));
    assert!(panes.contains("root.layout_semantic_selected(item_index);"));
    assert!(panes.contains("root.inspector_widget_action(\"slot.semantic.value.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"slot.semantic.delete\", \"\");"));
    assert!(panes.contains("root.inspector_widget_action(\"layout.semantic.value.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"layout.semantic.delete\", \"\");"));
}

#[test]
fn ui_asset_editor_pane_declares_binding_inspector_editing_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_inspector_binding_items: [string],"));
    assert!(source.contains("ui_asset_inspector_binding_selected_index: int,"));
    assert!(source.contains("ui_asset_inspector_binding_id: string,"));
    assert!(source.contains("ui_asset_inspector_binding_event: string,"));
    assert!(source.contains("ui_asset_inspector_binding_event_items: [string],"));
    assert!(source.contains("ui_asset_inspector_binding_event_selected_index: int,"));
    assert!(source.contains("ui_asset_inspector_binding_route: string,"));
    assert!(source.contains("ui_asset_inspector_binding_action_kind_items: [string],"));
    assert!(source.contains("ui_asset_inspector_binding_action_kind_selected_index: int,"));
    assert!(source.contains("ui_asset_inspector_binding_payload_items: [string],"));
    assert!(source.contains("ui_asset_inspector_binding_payload_selected_index: int,"));
    assert!(source.contains("ui_asset_inspector_binding_payload_key: string,"));
    assert!(source.contains("ui_asset_inspector_binding_payload_value: string,"));
    assert!(source
        .contains("callback ui_asset_binding_selected(instance_id: string, item_index: int);"));
    assert!(source.contains(
        "callback ui_asset_binding_event_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains(
        "callback ui_asset_binding_action_kind_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains(
        "callback ui_asset_binding_payload_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains("callback ui_asset_binding_payload_action(instance_id: string, action_id: string, payload_key: string, payload_value: string);"));

    assert!(pane_surface
        .contains("inspector_binding_items: root.pane.ui_asset_inspector_binding_items;"));
    assert!(pane_surface.contains(
        "inspector_binding_selected_index: root.pane.ui_asset_inspector_binding_selected_index;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_event_items: root.pane.ui_asset_inspector_binding_event_items;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_event_selected_index: root.pane.ui_asset_inspector_binding_event_selected_index;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_action_kind_items: root.pane.ui_asset_inspector_binding_action_kind_items;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_action_kind_selected_index: root.pane.ui_asset_inspector_binding_action_kind_selected_index;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_payload_items: root.pane.ui_asset_inspector_binding_payload_items;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_payload_selected_index: root.pane.ui_asset_inspector_binding_payload_selected_index;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_payload_key: root.pane.ui_asset_inspector_binding_payload_key;"
    ));
    assert!(pane_surface.contains(
        "inspector_binding_payload_value: root.pane.ui_asset_inspector_binding_payload_value;"
    ));
    assert!(pane_surface.contains(
        "binding_selected(item_index) => { root.ui_asset_binding_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "binding_event_selected(item_index) => { root.ui_asset_binding_event_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "binding_action_kind_selected(item_index) => { root.ui_asset_binding_action_kind_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "binding_payload_selected(item_index) => { root.ui_asset_binding_payload_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "binding_payload_action(action_id, payload_key, payload_value) => { root.ui_asset_binding_payload_action(root.pane.id, action_id, payload_key, payload_value); }"
    ));

    assert!(pane_block.contains("in property <[string]> inspector_binding_items;"));
    assert!(pane_block.contains("in property <int> inspector_binding_selected_index;"));
    assert!(pane_block.contains("in property <string> inspector_binding_id;"));
    assert!(pane_block.contains("in property <string> inspector_binding_event;"));
    assert!(pane_block.contains("in property <[string]> inspector_binding_event_items;"));
    assert!(pane_block.contains("in property <int> inspector_binding_event_selected_index;"));
    assert!(pane_block.contains("in property <string> inspector_binding_route;"));
    assert!(pane_block.contains("in property <[string]> inspector_binding_action_kind_items;"));
    assert!(pane_block.contains("in property <int> inspector_binding_action_kind_selected_index;"));
    assert!(pane_block.contains("in property <[string]> inspector_binding_payload_items;"));
    assert!(pane_block.contains("in property <int> inspector_binding_payload_selected_index;"));
    assert!(pane_block.contains("in property <string> inspector_binding_payload_key;"));
    assert!(pane_block.contains("in property <string> inspector_binding_payload_value;"));
    assert!(pane_block.contains("callback binding_selected(item_index: int);"));
    assert!(pane_block.contains("callback binding_event_selected(item_index: int);"));
    assert!(pane_block.contains("callback binding_action_kind_selected(item_index: int);"));
    assert!(pane_block.contains("callback binding_payload_selected(item_index: int);"));
    assert!(pane_block.contains(
        "callback binding_payload_action(action_id: string, payload_key: string, payload_value: string);"
    ));

    assert!(panes.contains("text: \"Bindings\";"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.add\", \"\");"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.delete\", \"\");"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.id.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.route.set\", value);"));
    assert!(panes.contains("title: \"Event\";"));
    assert!(panes.contains("items: root.inspector_binding_event_items;"));
    assert!(panes.contains("selected_index: root.inspector_binding_event_selected_index;"));
    assert!(panes.contains("root.binding_event_selected(item_index);"));
    assert!(panes.contains("title: \"Action Kind\";"));
    assert!(panes.contains("items: root.inspector_binding_action_kind_items;"));
    assert!(panes.contains("selected_index: root.inspector_binding_action_kind_selected_index;"));
    assert!(panes.contains("root.binding_action_kind_selected(item_index);"));
    assert!(panes.contains("title: \"Payload\";"));
    assert!(panes.contains("items: root.inspector_binding_payload_items;"));
    assert!(panes.contains("selected_index: root.inspector_binding_payload_selected_index;"));
    assert!(panes.contains("root.binding_payload_selected(item_index);"));
    assert!(panes.contains("root.binding_payload_action(\"binding.payload.upsert\""));
    assert!(panes.contains("root.binding_payload_action(\"binding.payload.delete\", \"\", \"\");"));
}

#[test]
fn ui_asset_editor_pane_declares_palette_tree_authoring_and_selection_sync_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source
        .contains("callback ui_asset_palette_selected(instance_id: string, item_index: int);"));
    assert!(source.contains("ui_asset_palette_selected_index: int,"));
    assert!(source.contains("ui_asset_hierarchy_selected_index: int,"));
    assert!(source.contains("ui_asset_preview_selected_index: int,"));
    assert!(source.contains("ui_asset_source_selected_block_label: string,"));
    assert!(source.contains("ui_asset_source_selected_line: int,"));
    assert!(source.contains("ui_asset_source_selected_excerpt: string,"));
    assert!(source.contains("ui_asset_source_roundtrip_status: string,"));
    assert!(source.contains("ui_asset_source_outline_items: [string],"));
    assert!(source.contains("ui_asset_source_outline_selected_index: int,"));
    assert!(source.contains("ui_asset_preview_surface_width: float,"));
    assert!(source.contains("ui_asset_preview_surface_height: float,"));
    assert!(source.contains("ui_asset_preview_canvas_items: [UiAssetCanvasNodeData],"));

    assert!(
        pane_surface.contains("palette_selected_index: root.pane.ui_asset_palette_selected_index;")
    );
    assert!(pane_surface
        .contains("hierarchy_selected_index: root.pane.ui_asset_hierarchy_selected_index;"));
    assert!(
        pane_surface.contains("preview_selected_index: root.pane.ui_asset_preview_selected_index;")
    );
    assert!(pane_surface
        .contains("source_selected_block_label: root.pane.ui_asset_source_selected_block_label;"));
    assert!(pane_surface.contains("source_selected_line: root.pane.ui_asset_source_selected_line;"));
    assert!(pane_surface
        .contains("source_selected_excerpt: root.pane.ui_asset_source_selected_excerpt;"));
    assert!(pane_surface
        .contains("source_roundtrip_status: root.pane.ui_asset_source_roundtrip_status;"));
    assert!(pane_surface.contains("source_outline_items: root.pane.ui_asset_source_outline_items;"));
    assert!(pane_surface.contains(
        "source_outline_selected_index: root.pane.ui_asset_source_outline_selected_index;"
    ));
    assert!(
        pane_surface.contains("preview_surface_width: root.pane.ui_asset_preview_surface_width;")
    );
    assert!(
        pane_surface.contains("preview_surface_height: root.pane.ui_asset_preview_surface_height;")
    );
    assert!(pane_surface.contains("preview_canvas_items: root.pane.ui_asset_preview_canvas_items;"));
    assert!(pane_surface.contains(
        "palette_selected(item_index) => { root.ui_asset_palette_selected(root.pane.id, item_index); }"
    ));

    assert!(pane_block.contains("in property <int> palette_selected_index;"));
    assert!(pane_block.contains("in property <int> hierarchy_selected_index;"));
    assert!(pane_block.contains("in property <int> preview_selected_index;"));
    assert!(pane_block.contains("in property <string> source_selected_block_label;"));
    assert!(pane_block.contains("in property <int> source_selected_line;"));
    assert!(pane_block.contains("in property <string> source_selected_excerpt;"));
    assert!(pane_block.contains("in property <string> source_roundtrip_status;"));
    assert!(pane_block.contains("in property <[string]> source_outline_items;"));
    assert!(pane_block.contains("in property <int> source_outline_selected_index;"));
    assert!(pane_block.contains("in property <float> preview_surface_width;"));
    assert!(pane_block.contains("in property <float> preview_surface_height;"));
    assert!(pane_block.contains("in property <[UiAssetCanvasNodeData]> preview_canvas_items;"));
    assert!(pane_block.contains("callback palette_selected(item_index: int);"));

    assert!(panes.contains("export struct UiAssetCanvasNodeData {"));
    assert!(panes.contains("component UiAssetCanvasSurface inherits Rectangle {"));
    assert!(panes.contains("text: \"Designer Canvas\";"));
    assert!(panes.contains("items: root.preview_canvas_items;"));
    assert!(panes.contains("surface_width: root.preview_surface_width;"));
    assert!(panes.contains("surface_height: root.preview_surface_height;"));
    assert!(panes.contains("title: \"Render Stack\";"));
    assert!(panes.contains("title: \"Source Outline\";"));
    assert!(panes.contains("selected_index: root.source_outline_selected_index;"));
    assert!(panes.contains("root.source_outline_selected(item_index);"));
    assert!(panes.contains("title: \"Palette\";"));
    assert!(panes.contains("selected_index: root.palette_selected_index;"));
    assert!(panes.contains("root.palette_selected(item_index);"));
    assert!(panes.contains("selected_index: root.hierarchy_selected_index;"));
    assert!(panes.contains("selected_index: root.preview_selected_index;"));
    assert!(panes.contains("root.action(\"palette.insert.child\");"));
    assert!(panes.contains("root.action(\"palette.insert.after\");"));
    assert!(panes.contains("root.action(\"canvas.move.up\");"));
    assert!(panes.contains("root.action(\"canvas.move.down\");"));
    assert!(panes.contains("root.action(\"canvas.reparent.into_previous\");"));
    assert!(panes.contains("root.action(\"canvas.reparent.into_next\");"));
    assert!(panes.contains("root.action(\"canvas.reparent.outdent\");"));
    assert!(panes.contains("root.action(\"canvas.convert.reference\");"));
    assert!(panes.contains("root.action(\"canvas.extract.component\");"));
    assert!(panes.contains("root.action(\"canvas.wrap.vertical_box\");"));
    assert!(panes.contains("root.action(\"canvas.unwrap\");"));
    assert!(panes.contains("text: root.source_selected_block_label != \"\" ? root.source_selected_block_label : \"No source block\";"));
    assert!(panes.contains(
        "text: root.source_selected_line >= 0 ? \"line \" + root.source_selected_line : \"\";"
    ));
    assert!(panes.contains("text: root.source_roundtrip_status;"));
    assert!(panes.contains("text: root.source_selected_excerpt;"));
}

#[test]
fn ui_asset_editor_pane_declares_convert_to_reference_action_and_state_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_can_convert_to_reference: bool,"));
    assert!(pane_surface
        .contains("can_convert_to_reference: root.pane.ui_asset_can_convert_to_reference;"));
    assert!(pane_block.contains("in property <bool> can_convert_to_reference;"));
    assert!(panes.contains("label: \"To Ref\";"));
    assert!(panes.contains("enabled: root.can_convert_to_reference;"));
    assert!(panes.contains("active: root.can_convert_to_reference;"));
    assert!(panes.contains("root.action(\"canvas.convert.reference\");"));
}

#[test]
fn ui_asset_editor_pane_declares_extract_component_action_and_state_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_can_extract_component: bool,"));
    assert!(
        pane_surface.contains("can_extract_component: root.pane.ui_asset_can_extract_component;")
    );
    assert!(pane_block.contains("in property <bool> can_extract_component;"));
    assert!(panes.contains("label: \"Extract\";"));
    assert!(panes.contains("enabled: root.can_extract_component;"));
    assert!(panes.contains("active: root.can_extract_component;"));
    assert!(panes.contains("root.action(\"canvas.extract.component\");"));
}

#[test]
fn ui_asset_editor_pane_declares_promote_widget_action_and_state_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_can_promote_to_external_widget: bool,"));
    assert!(pane_surface.contains(
        "can_promote_to_external_widget: root.pane.ui_asset_can_promote_to_external_widget;"
    ));
    assert!(pane_block.contains("in property <bool> can_promote_to_external_widget;"));
    assert!(panes.contains("label: \"Promote\";"));
    assert!(panes.contains("enabled: root.can_promote_to_external_widget;"));
    assert!(panes.contains("active: root.can_promote_to_external_widget;"));
    assert!(panes.contains("root.action(\"canvas.promote.widget\");"));
}

#[test]
fn ui_asset_editor_pane_declares_promote_widget_draft_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_inspector_promote_asset_id: string,"));
    assert!(source.contains("ui_asset_inspector_promote_component_name: string,"));
    assert!(source.contains("ui_asset_inspector_promote_document_id: string,"));
    assert!(source.contains("ui_asset_inspector_can_edit_promote_draft: bool,"));

    assert!(pane_surface
        .contains("inspector_promote_asset_id: root.pane.ui_asset_inspector_promote_asset_id;"));
    assert!(pane_surface.contains(
        "inspector_promote_component_name: root.pane.ui_asset_inspector_promote_component_name;"
    ));
    assert!(pane_surface.contains(
        "inspector_promote_document_id: root.pane.ui_asset_inspector_promote_document_id;"
    ));
    assert!(pane_surface.contains(
        "inspector_can_edit_promote_draft: root.pane.ui_asset_inspector_can_edit_promote_draft;"
    ));

    assert!(pane_block.contains("in property <string> inspector_promote_asset_id;"));
    assert!(pane_block.contains("in property <string> inspector_promote_component_name;"));
    assert!(pane_block.contains("in property <string> inspector_promote_document_id;"));
    assert!(pane_block.contains("in property <bool> inspector_can_edit_promote_draft;"));

    assert!(panes.contains("text: \"Promote Draft\";"));
    assert!(panes.contains("text: \"Asset\";"));
    assert!(panes.contains("text: \"Comp\";"));
    assert!(panes.contains("text: \"Doc\";"));
    assert!(panes.contains("root.inspector_widget_action(\"promote.asset_id.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"promote.component_name.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"promote.document_id.set\", value);"));
}

#[test]
fn ui_asset_editor_pane_declares_hierarchy_activation_callback_and_double_click_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source
        .contains("callback ui_asset_hierarchy_activated(instance_id: string, item_index: int);"));
    assert!(pane_surface.contains(
        "hierarchy_activated(item_index) => { root.ui_asset_hierarchy_activated(root.pane.id, item_index); }"
    ));

    assert!(panes.contains("callback item_activated(item_index: int);"));
    assert!(panes.contains("double-clicked => {"));
    assert!(panes.contains("root.item_activated(index);"));
    assert!(pane_block.contains("callback hierarchy_activated(item_index: int);"));
    assert!(
        panes.contains("item_activated(item_index) => { root.hierarchy_activated(item_index); }")
    );
}

#[test]
fn ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source
        .contains("callback ui_asset_preview_activated(instance_id: string, item_index: int);"));
    assert!(pane_surface.contains(
        "preview_activated(item_index) => { root.ui_asset_preview_activated(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "source_outline_selected(item_index) => { root.ui_asset_source_outline_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_block.contains("callback preview_activated(item_index: int);"));
    assert!(pane_block.contains("callback source_outline_selected(item_index: int);"));
    assert!(panes.contains("item_activated(item_index) => { root.preview_activated(item_index); }"));
    assert!(panes.contains("UiAssetCanvasSurface {"));
}

#[test]
fn ui_asset_editor_canvas_declares_selected_frame_authoring_overlay_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(canvas_block.contains("in property <bool> palette_has_selection: false;"));
    assert!(canvas_block.contains("in property <bool> can_insert_child: false;"));
    assert!(canvas_block.contains("in property <bool> can_insert_after: false;"));
    assert!(canvas_block.contains("in property <bool> can_move_up: false;"));
    assert!(canvas_block.contains("in property <bool> can_move_down: false;"));
    assert!(canvas_block.contains("in property <bool> can_reparent_into_previous: false;"));
    assert!(canvas_block.contains("in property <bool> can_reparent_into_next: false;"));
    assert!(canvas_block.contains("in property <bool> can_reparent_outdent: false;"));
    assert!(canvas_block.contains("in property <bool> can_open_reference: false;"));
    assert!(canvas_block.contains("in property <bool> can_convert_to_reference: false;"));
    assert!(canvas_block.contains("in property <bool> can_extract_component: false;"));
    assert!(canvas_block.contains("in property <bool> can_promote_to_external_widget: false;"));
    assert!(canvas_block.contains("in property <bool> can_wrap_in_vertical_box: false;"));
    assert!(canvas_block.contains("in property <bool> can_unwrap: false;"));
    assert!(canvas_block.contains("callback action_requested(action_id: string);"));
    assert!(canvas_block.contains("property <int> overlay_index:"));
    assert!(canvas_block.contains("if root.overlay_index >= 0: overlay := Rectangle {"));
    assert!(canvas_block.contains("text: root.items[root.overlay_index].label + \" • \" + root.items[root.overlay_index].kind;"));
    assert!(canvas_block.contains("label: \"Add In\";"));
    assert!(canvas_block.contains("enabled: root.can_insert_child;"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.child\");"));
    assert!(canvas_block.contains("enabled: root.can_insert_after;"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.after\");"));
    assert!(canvas_block.contains("enabled: root.can_move_up;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.move.up\");"));
    assert!(canvas_block.contains("enabled: root.can_move_down;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.move.down\");"));
    assert!(canvas_block.contains("enabled: root.can_reparent_into_previous;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_previous\");"));
    assert!(canvas_block.contains("enabled: root.can_reparent_into_next;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_next\");"));
    assert!(canvas_block.contains("enabled: root.can_reparent_outdent;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.outdent\");"));
    assert!(canvas_block.contains("root.action_requested(\"reference.open\");"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.convert.reference\");"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.extract.component\");"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.promote.widget\");"));
    assert!(canvas_block.contains("enabled: root.can_wrap_in_vertical_box;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.wrap.vertical_box\");"));
    assert!(canvas_block.contains("enabled: root.can_unwrap;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.unwrap\");"));

    assert!(source.contains("ui_asset_can_insert_child: bool,"));
    assert!(source.contains("ui_asset_can_insert_after: bool,"));
    assert!(source.contains("ui_asset_can_move_up: bool,"));
    assert!(source.contains("ui_asset_can_move_down: bool,"));
    assert!(source.contains("ui_asset_can_reparent_into_previous: bool,"));
    assert!(source.contains("ui_asset_can_reparent_into_next: bool,"));
    assert!(source.contains("ui_asset_can_reparent_outdent: bool,"));
    assert!(source.contains("ui_asset_can_wrap_in_vertical_box: bool,"));
    assert!(source.contains("ui_asset_can_unwrap: bool,"));
    assert!(pane_surface.contains("can_insert_child: root.pane.ui_asset_can_insert_child;"));
    assert!(pane_surface.contains("can_insert_after: root.pane.ui_asset_can_insert_after;"));
    assert!(pane_surface.contains("can_move_up: root.pane.ui_asset_can_move_up;"));
    assert!(pane_surface.contains("can_move_down: root.pane.ui_asset_can_move_down;"));
    assert!(pane_surface
        .contains("can_reparent_into_previous: root.pane.ui_asset_can_reparent_into_previous;"));
    assert!(
        pane_surface.contains("can_reparent_into_next: root.pane.ui_asset_can_reparent_into_next;")
    );
    assert!(pane_surface.contains("can_reparent_outdent: root.pane.ui_asset_can_reparent_outdent;"));
    assert!(pane_surface
        .contains("can_wrap_in_vertical_box: root.pane.ui_asset_can_wrap_in_vertical_box;"));
    assert!(pane_surface.contains("can_unwrap: root.pane.ui_asset_can_unwrap;"));

    assert!(panes.contains("palette_has_selection: root.palette_selected_index >= 0;"));
    assert!(panes.contains("can_insert_child: root.can_insert_child;"));
    assert!(panes.contains("can_insert_after: root.can_insert_after;"));
    assert!(panes.contains("can_move_up: root.can_move_up;"));
    assert!(panes.contains("can_move_down: root.can_move_down;"));
    assert!(panes.contains("can_reparent_into_previous: root.can_reparent_into_previous;"));
    assert!(panes.contains("can_reparent_into_next: root.can_reparent_into_next;"));
    assert!(panes.contains("can_reparent_outdent: root.can_reparent_outdent;"));
    assert!(panes.contains("can_open_reference: root.can_open_reference;"));
    assert!(panes.contains("can_convert_to_reference: root.can_convert_to_reference;"));
    assert!(panes.contains("can_extract_component: root.can_extract_component;"));
    assert!(panes.contains("can_promote_to_external_widget: root.can_promote_to_external_widget;"));
    assert!(panes.contains("can_wrap_in_vertical_box: root.can_wrap_in_vertical_box;"));
    assert!(panes.contains("can_unwrap: root.can_unwrap;"));
    assert!(panes.contains("action_requested(action_id) => { root.action(action_id); }"));
}

#[test]
fn ui_asset_editor_canvas_declares_contextual_insert_and_reparent_targets() {
    let panes = panes_source();

    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(canvas_block.contains("property <float> target_pad_thickness: 14.0;"));
    assert!(canvas_block.contains(
        "if root.overlay_index >= 0 && root.can_insert_child: insert_child_target := Rectangle {"
    ));
    assert!(canvas_block.contains("text: \"Insert In\";"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.child\");"));
    assert!(canvas_block.contains(
        "if root.overlay_index >= 0 && root.can_insert_after: insert_after_target := Rectangle {"
    ));
    assert!(canvas_block.contains("text: \"Insert After\";"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.after\");"));
    assert!(
        canvas_block.contains("if root.overlay_index >= 0 && root.can_reparent_into_previous: reparent_prev_target := Rectangle {")
    );
    assert!(canvas_block.contains("text: \"Into Prev\";"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_previous\");"));
    assert!(
        canvas_block.contains("if root.overlay_index >= 0 && root.can_reparent_into_next: reparent_next_target := Rectangle {")
    );
    assert!(canvas_block.contains("text: \"Into Next\";"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_next\");"));
    assert!(canvas_block.contains(
        "if root.overlay_index >= 0 && root.can_reparent_outdent: outdent_target := Rectangle {"
    ));
    assert!(canvas_block.contains("text: \"Outdent\";"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.outdent\");"));
}

#[test]
fn ui_asset_editor_canvas_declares_drag_authoring_state_and_drop_resolution() {
    let panes = panes_source();

    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(canvas_block.contains("property <float> drag_threshold_px: 8.0;"));
    assert!(canvas_block.contains("property <bool> drag_pressed: false;"));
    assert!(canvas_block.contains("property <bool> drag_active: false;"));
    assert!(canvas_block.contains("property <int> drag_source_index: -1;"));
    assert!(canvas_block.contains("property <float> drag_press_x: 0.0;"));
    assert!(canvas_block.contains("property <float> drag_press_y: 0.0;"));
    assert!(canvas_block.contains("property <float> drag_pointer_x: 0.0;"));
    assert!(canvas_block.contains("property <float> drag_pointer_y: 0.0;"));
    assert!(canvas_block.contains("property <string> drag_target_action:"));
    assert!(canvas_block
        .contains("root.drag_target_action == \"palette.insert.child\" ? \"Insert In\" :"));
    assert!(canvas_block.contains(
        "root.drag_target_action == \"canvas.reparent.into_previous\" ? \"Into Prev\" :"
    ));
    assert!(canvas_block.contains("if root.drag_active: drag_overlay := Rectangle {"));
    assert!(canvas_block.contains(
        "text: root.drag_target_label != \"\" ? root.drag_target_label : \"Drag Authoring\";"
    ));
    assert!(canvas_block.contains("mouse-cursor: MouseCursor.grabbing;"));
    assert!(canvas_block.contains("root.drag_source_index = index;"));
    assert!(canvas_block.contains("root.drag_pointer_x = frame_rect.x / 1px + self.mouse-x / 1px;"));
    assert!(canvas_block.contains("root.drag_pointer_y = frame_rect.y / 1px + self.mouse-y / 1px;"));
    assert!(canvas_block.contains("root.drag_active = true;"));
    assert!(canvas_block.contains("root.action_requested(root.drag_target_action);"));
    assert!(canvas_block.contains("root.drag_active = false;"));
    assert!(canvas_block.contains("root.drag_source_index = -1;"));
}

#[test]
fn ui_asset_editor_pane_declares_palette_drag_creation_flow() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(source.contains(
        "callback ui_asset_palette_drag_hover(instance_id: string, surface_x: float, surface_y: float);"
    ));
    assert!(source.contains("callback ui_asset_palette_drag_drop(instance_id: string);"));
    assert!(source.contains("callback ui_asset_palette_drag_cancel(instance_id: string);"));
    assert!(source.contains("ui_asset_palette_drag_target_preview_index: int,"));
    assert!(source.contains("ui_asset_palette_drag_target_action: string,"));
    assert!(source.contains("ui_asset_palette_drag_target_label: string,"));

    assert!(pane_surface.contains(
        "palette_drag_target_preview_index: root.pane.ui_asset_palette_drag_target_preview_index;"
    ));
    assert!(pane_surface
        .contains("palette_drag_target_action: root.pane.ui_asset_palette_drag_target_action;"));
    assert!(pane_surface
        .contains("palette_drag_target_label: root.pane.ui_asset_palette_drag_target_label;"));
    assert!(pane_surface.contains(
        "palette_drag_hovered(surface_x, surface_y) => { root.ui_asset_palette_drag_hover(root.pane.id, surface_x, surface_y); }"
    ));
    assert!(pane_surface
        .contains("palette_drag_dropped() => { root.ui_asset_palette_drag_drop(root.pane.id); }"));
    assert!(pane_surface.contains(
        "palette_drag_cancelled() => { root.ui_asset_palette_drag_cancel(root.pane.id); }"
    ));

    assert!(pane_block.contains("in-out property <bool> palette_drag_active: false;"));
    assert!(pane_block.contains("in-out property <int> palette_drag_source_index: -1;"));
    assert!(pane_block.contains("in-out property <float> palette_drag_pointer_x: 0.0;"));
    assert!(pane_block.contains("in-out property <float> palette_drag_pointer_y: 0.0;"));
    assert!(pane_block.contains("property <string> palette_drag_label:"));
    assert!(pane_block.contains("in property <int> palette_drag_target_preview_index: -1;"));
    assert!(pane_block.contains("in property <string> palette_drag_target_action;"));
    assert!(pane_block.contains("in property <string> palette_drag_target_label;"));
    assert!(
        pane_block.contains("callback palette_drag_hovered(surface_x: float, surface_y: float);")
    );
    assert!(pane_block.contains("callback palette_drag_dropped();"));
    assert!(pane_block.contains("callback palette_drag_cancelled();"));
    assert!(pane_block.contains("if root.palette_drag_active: TouchArea {"));
    assert!(pane_block.contains("mouse-cursor: MouseCursor.grabbing;"));
    assert!(pane_block.contains("root.palette_drag_hovered("));
    assert!(pane_block.contains("preview_canvas.surface_origin_x"));
    assert!(pane_block.contains("preview_canvas.surface_origin_y"));
    assert!(pane_block.contains("preview_canvas.surface_scale"));
    assert!(pane_block.contains("if (event.kind == PointerEventKind.up) {"));
    assert!(pane_block.contains("if (root.palette_drag_target_action != \"\") {"));
    assert!(pane_block.contains("root.palette_drag_dropped();"));
    assert!(pane_block.contains("root.palette_drag_cancelled();"));
    assert!(pane_block.contains("root.palette_drag_active = false;"));
    assert!(pane_block.contains("root.palette_drag_source_index = -1;"));

    assert!(pane_block.contains("drag_enabled: true;"));
    assert!(pane_block.contains("item_drag_started(item_index, x, y) => {"));
    assert!(pane_block.contains("root.palette_selected(item_index);"));
    assert!(pane_block.contains("root.palette_drag_active = true;"));
    assert!(pane_block.contains("root.palette_drag_source_index = item_index;"));
    assert!(pane_block.contains("root.palette_drag_pointer_x = palette_section.x / 1px + x;"));
    assert!(pane_block.contains("root.palette_drag_pointer_y = palette_section.y / 1px + y;"));

    assert!(canvas_block.contains("in property <bool> external_drag_active: false;"));
    assert!(canvas_block.contains("in property <float> external_drag_pointer_x: 0.0;"));
    assert!(canvas_block.contains("in property <float> external_drag_pointer_y: 0.0;"));
    assert!(canvas_block.contains("in property <int> external_drag_target_index: -1;"));
    assert!(canvas_block.contains("in property <string> external_drag_target_action;"));
    assert!(canvas_block.contains("in property <string> external_drag_target_label;"));
    assert!(canvas_block.contains("out property <float> surface_scale:"));
    assert!(canvas_block.contains("out property <float> surface_origin_x:"));
    assert!(canvas_block.contains("out property <float> surface_origin_y:"));
    assert!(canvas_block.contains("property <bool> external_target_active:"));
    assert!(canvas_block.contains("self.external_target_active ?"));
    assert!(
        canvas_block.contains("if root.external_drag_active: external_drag_overlay := Rectangle {")
    );
    assert!(canvas_block.contains(
        "text: root.external_drag_target_label != \"\" ? root.external_drag_target_label : \"Drop On Canvas\";"
    ));
    assert!(canvas_block.contains("drop_inside_overlay := Rectangle {"));
    assert!(canvas_block.contains("drop_after_overlay := Rectangle {"));
    assert!(pane_block.contains("external_drag_active: root.palette_drag_active;"));
    assert!(pane_block.contains(
        "external_drag_pointer_x: root.palette_drag_pointer_x - preview_canvas.x / 1px;"
    ));
    assert!(pane_block.contains(
        "external_drag_pointer_y: root.palette_drag_pointer_y - preview_canvas.y / 1px;"
    ));
    assert!(
        pane_block.contains("external_drag_target_index: root.palette_drag_target_preview_index;")
    );
    assert!(pane_block.contains("external_drag_target_action: root.palette_drag_target_action;"));
    assert!(pane_block.contains("external_drag_target_label: root.palette_drag_target_label;"));
}

#[test]
fn ui_asset_editor_pane_declares_palette_target_cycle_panel_and_keyboard_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    let drag_overlay = block_after(&panes, "if root.palette_drag_active: TouchArea {");

    assert!(source.contains("ui_asset_palette_drag_candidate_items: [string],"));
    assert!(source.contains("ui_asset_palette_drag_candidate_selected_index: int,"));
    assert!(pane_surface
        .contains("palette_drag_candidate_items: root.pane.ui_asset_palette_drag_candidate_items;"));
    assert!(pane_surface.contains(
        "palette_drag_candidate_selected_index: root.pane.ui_asset_palette_drag_candidate_selected_index;"
    ));

    assert!(pane_block.contains("in property <[string]> palette_drag_candidate_items;"));
    assert!(pane_block.contains("in property <int> palette_drag_candidate_selected_index: -1;"));
    assert!(pane_block.contains("title: \"Target Cycle\";"));
    assert!(pane_block.contains("items: root.palette_drag_candidate_items;"));
    assert!(pane_block.contains(
        "selected_index: root.palette_drag_candidate_selected_index;"
    ));

    assert!(drag_overlay.contains("drag_focus := FocusScope {"));
    assert!(drag_overlay.contains("key-pressed(event) => {"));
    assert!(drag_overlay.contains("root.action(\"palette.target.previous\");"));
    assert!(drag_overlay.contains("root.action(\"palette.target.next\");"));
    assert!(drag_overlay.contains("root.palette_drag_dropped();"));
    assert!(drag_overlay.contains("root.palette_drag_cancelled();"));
}

#[test]
fn ui_asset_editor_pane_declares_sticky_palette_target_chooser_controls() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains("ui_asset_palette_target_chooser_active: bool,"));
    assert!(source.contains(
        "callback ui_asset_palette_target_candidate_selected(instance_id: string, item_index: int);"
    ));
    assert!(source.contains("callback ui_asset_palette_target_confirm(instance_id: string);"));
    assert!(source.contains("callback ui_asset_palette_target_cancel(instance_id: string);"));
    assert!(pane_surface.contains(
        "palette_target_chooser_active: root.pane.ui_asset_palette_target_chooser_active;"
    ));
    assert!(pane_surface.contains(
        "palette_target_candidate_selected(item_index) => { root.ui_asset_palette_target_candidate_selected(root.pane.id, item_index); }"
    ));
    assert!(pane_surface.contains(
        "palette_target_confirm() => { root.ui_asset_palette_target_confirm(root.pane.id); }"
    ));
    assert!(pane_surface.contains(
        "palette_target_cancel() => { root.ui_asset_palette_target_cancel(root.pane.id); }"
    ));

    assert!(pane_block.contains("in property <bool> palette_target_chooser_active: false;"));
    assert!(pane_block.contains("callback palette_target_candidate_selected(item_index: int);"));
    assert!(pane_block.contains("callback palette_target_confirm();"));
    assert!(pane_block.contains("callback palette_target_cancel();"));
    assert!(panes.contains(
        "if root.palette_drag_candidate_items.length > 1 && (root.palette_drag_active || root.palette_target_chooser_active): Rectangle {"
    ));
    assert!(panes.contains("root.palette_target_candidate_selected(item_index);"));
    assert!(panes.contains("root.palette_target_confirm();"));
    assert!(panes.contains("root.palette_target_cancel();"));
}

#[test]
fn ui_asset_editor_pane_declares_typed_parent_specific_slot_layout_and_binding_fields() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(source.contains("ui_asset_inspector_slot_kind: string,"));
    assert!(source.contains("ui_asset_inspector_slot_overlay_anchor_x: string,"));
    assert!(source.contains("ui_asset_inspector_slot_grid_row: string,"));
    assert!(source.contains("ui_asset_inspector_slot_flow_alignment: string,"));
    assert!(source.contains("ui_asset_inspector_layout_kind: string,"));
    assert!(source.contains("ui_asset_inspector_layout_scroll_axis: string,"));
    assert!(source.contains("ui_asset_inspector_layout_scrollbar_visibility: string,"));
    assert!(source.contains("ui_asset_inspector_binding_route_target: string,"));
    assert!(source.contains("ui_asset_inspector_binding_action_target: string,"));

    assert!(pane_block.contains("in property <string> inspector_slot_kind;"));
    assert!(pane_block.contains("in property <string> inspector_slot_overlay_anchor_x;"));
    assert!(pane_block.contains("in property <string> inspector_slot_grid_row;"));
    assert!(pane_block.contains("in property <string> inspector_slot_flow_alignment;"));
    assert!(pane_block.contains("in property <string> inspector_layout_kind;"));
    assert!(pane_block.contains("in property <string> inspector_layout_scroll_axis;"));
    assert!(pane_block.contains("in property <string> inspector_layout_scrollbar_visibility;"));
    assert!(pane_block.contains("in property <string> inspector_binding_route_target;"));
    assert!(pane_block.contains("in property <string> inspector_binding_action_target;"));

    assert!(panes.contains("root.inspector_widget_action(\"slot.overlay.anchor_x.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"slot.grid.row.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"slot.flow.alignment.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"layout.scroll.axis.set\", value);"));
    assert!(panes.contains(
        "root.inspector_widget_action(\"layout.scroll.scrollbar_visibility.set\", value);"
    ));
    assert!(panes.contains(
        "root.inspector_widget_action(\"binding.route_target.set\", value);"
    ));
    assert!(panes.contains(
        "root.inspector_widget_action(\"binding.action_target.set\", value);"
    ));
}

#[test]
fn tab_drag_controls_use_low_drag_threshold_for_single_click_responsiveness() {
    let chrome_source_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/chrome.slint");
    let chrome_source =
        fs::read_to_string(chrome_source_path).expect("chrome.slint should be readable");

    assert_eq!(
        chrome_source
            .matches("property <float> drag_threshold_px: 4.0;")
            .count(),
        2
    );
    assert!(!chrome_source.contains("property <float> drag_threshold_px: 10.0;"));
}

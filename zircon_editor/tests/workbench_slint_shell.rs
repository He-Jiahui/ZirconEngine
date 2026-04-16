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
    assert!(source.contains("x: top_bar.file_menu_button_local_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 1: Rectangle {"));
    assert!(source.contains("x: top_bar.edit_menu_button_local_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 2: Rectangle {"));
    assert!(source.contains("x: top_bar.selection_menu_button_local_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 3: Rectangle {"));
    assert!(source.contains("x: top_bar.view_menu_button_local_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 4: Rectangle {"));
    assert!(source.contains("x: top_bar.window_menu_button_local_frame.x * 1px;"));
    assert!(source.contains("if root.open_menu_index == 5: Rectangle {"));
    assert!(source.contains("x: top_bar.help_menu_button_local_frame.x * 1px;"));

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
    assert!(source.contains("ui_asset_inspector_binding_route: string,"));
    assert!(source
        .contains("callback ui_asset_binding_selected(instance_id: string, item_index: int);"));

    assert!(pane_surface
        .contains("inspector_binding_items: root.pane.ui_asset_inspector_binding_items;"));
    assert!(pane_surface.contains(
        "inspector_binding_selected_index: root.pane.ui_asset_inspector_binding_selected_index;"
    ));
    assert!(pane_surface.contains(
        "binding_selected(item_index) => { root.ui_asset_binding_selected(root.pane.id, item_index); }"
    ));

    assert!(pane_block.contains("in property <[string]> inspector_binding_items;"));
    assert!(pane_block.contains("in property <int> inspector_binding_selected_index;"));
    assert!(pane_block.contains("in property <string> inspector_binding_id;"));
    assert!(pane_block.contains("in property <string> inspector_binding_event;"));
    assert!(pane_block.contains("in property <string> inspector_binding_route;"));
    assert!(pane_block.contains("callback binding_selected(item_index: int);"));

    assert!(panes.contains("text: \"Bindings\";"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.add\", \"\");"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.delete\", \"\");"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.id.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.event.set\", value);"));
    assert!(panes.contains("root.inspector_widget_action(\"binding.route.set\", value);"));
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

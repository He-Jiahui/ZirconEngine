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

fn pane_surface_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/pane_surface.slint");
    fs::read_to_string(path).expect("pane_surface.slint should be readable")
}

const UI_ASSET_EDITOR_PANE_MARKER: &str =
    "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {";

fn apply_presentation_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/ui/slint_host/ui/apply_presentation.rs");
    fs::read_to_string(path).expect("apply_presentation.rs should be readable")
}

fn slint_host_source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("{relative} should be readable"))
}

fn block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    if let Some(start) = source.find(marker) {
        return &source[start..];
    }

    if marker == UI_ASSET_EDITOR_PANE_MARKER {
        let pane_surface = pane_surface_source();
        let leaked = Box::leak(pane_surface.into_boxed_str());
        let start = leaked
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker `{marker}` in workbench/pane_surface.slint"));
        return &leaked[start..];
    }

    panic!("missing marker `{marker}` in workbench.slint");
}

fn scoped_block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker `{marker}` in workbench.slint"));
    let mut depth = 0usize;
    let mut opened = false;

    for (offset, ch) in source[start..].char_indices() {
        match ch {
            '{' => {
                depth += 1;
                opened = true;
            }
            '}' if opened => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..start + offset + 1];
                }
            }
            _ => {}
        }
    }

    panic!("missing closing brace for `{marker}` in workbench.slint");
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
    let shell_block =
        scoped_block_after(&source, "export component UiHostWindow inherits Window {");

    assert!(shell_block.contains("no-frame: false;"));
    assert!(shell_block.contains("resize-border-width: 8px;"));
    assert!(shell_block.contains("max-width:"));
    assert!(shell_block.contains("max-height:"));
}

#[test]
fn ui_host_window_root_delegates_to_internal_scaffold_only() {
    let source = shell_source();
    let shell_block =
        scoped_block_after(&source, "export component UiHostWindow inherits Window {");

    assert!(shell_block.contains("host := WorkbenchHostScaffold {"));
    assert!(shell_block.contains("width: root.width;"));
    assert!(shell_block.contains("height: root.height;"));
    assert!(!shell_block.contains("top_bar := Rectangle {"));
    assert!(!shell_block.contains("for window[index] in root.floating_windows"));
    assert!(!shell_block.contains("main_content_zone := Rectangle {"));
}

#[test]
fn workbench_shell_extracts_business_pane_surface_catalog_out_of_root_file() {
    let source = shell_source();
    let pane_surface = pane_surface_source();

    assert!(source.contains("import { PaneSurface } from \"workbench/pane_surface.slint\";"));
    assert!(!source.contains("component PaneSurface inherits Rectangle {"));

    assert!(pane_surface.contains("component PaneSurface inherits Rectangle {"));
    assert!(pane_surface.contains("if root.pane.kind == \"Welcome\": WelcomePane {"));
    assert!(pane_surface.contains(
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {"
    ));
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
fn shell_source_drops_legacy_drawer_extent_bindings() {
    let source = shell_source();
    let apply_presentation = apply_presentation_source();

    for removed_property in [
        "in property <float> left_drawer_extent",
        "in property <float> right_drawer_extent",
        "in property <float> bottom_drawer_extent",
        "left_drawer_extent <=> host.left_drawer_extent",
        "right_drawer_extent <=> host.right_drawer_extent",
        "bottom_drawer_extent <=> host.bottom_drawer_extent",
    ] {
        assert!(
            !source.contains(removed_property),
            "workbench shell should not keep legacy drawer extent binding `{removed_property}`"
        );
    }

    for removed_setter in [
        "set_left_drawer_extent(",
        "set_right_drawer_extent(",
        "set_bottom_drawer_extent(",
    ] {
        assert!(
            !apply_presentation.contains(removed_setter),
            "apply_presentation should not keep legacy drawer extent setter `{removed_setter}`"
        );
    }
}

#[test]
fn shell_source_drops_legacy_root_shell_geometry_fallback_helpers() {
    let root_shell_projection = slint_host_source("src/ui/slint_host/root_shell_projection.rs");
    let helpers = slint_host_source("src/ui/slint_host/app/helpers.rs");
    let viewport = slint_host_source("src/ui/slint_host/app/viewport.rs");
    let workspace_docking = slint_host_source("src/ui/slint_host/app/workspace_docking.rs");

    for removed_helper in [
        "geometry.region_frame(",
        "geometry.center_band_frame",
        "geometry.status_bar_frame",
        "geometry.viewport_content_frame",
        "legacy_root_activity_rail_frame(",
        "resolve_root_visible_drawer_region_frame(",
        "resolve_root_visible_drawer_document_region_frame(",
    ] {
        assert!(
            !root_shell_projection.contains(removed_helper),
            "root_shell_projection should not keep legacy geometry fallback helper `{removed_helper}`"
        );
    }

    for removed_fallback in [
        "frame_size(geometry.region_frame(region))",
        "frame_size(geometry.region_frame(ShellRegionId::Document))",
    ] {
        assert!(
            !helpers.contains(removed_fallback),
            "helpers should not keep legacy geometry fallback `{removed_fallback}`"
        );
    }

    for removed_fallback in [
        "geometry.region_frame(ShellRegionId::Document).width",
        "geometry.region_frame(region).width",
    ] {
        assert!(
            !viewport.contains(removed_fallback),
            "viewport toolbar sizing should not keep legacy geometry fallback `{removed_fallback}`"
        );
    }

    assert!(
        !workspace_docking.contains("ShellRegionId::Document => geometry.region_frame(region)"),
        "workspace docking resize capture should not keep legacy document-region geometry fallback"
    );
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
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(source.contains("callback ui_asset_action(instance_id: string, action_id: string);"));
    assert!(source.contains("callback ui_asset_source_edited(instance_id: string, value: string);"));
    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));

    let _pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface
        .contains("action(action_id) => { root.ui_asset_action(root.pane.id, action_id); }"));
    assert!(pane_surface
        .contains("source_edited(value) => { root.ui_asset_source_edited(root.pane.id, value); }"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(panes.contains("import { LineEdit, TextEdit } from \"std-widgets.slint\";"));
    assert!(pane_block.contains("in property <UiAssetEditorPaneData> pane;"));
    assert!(panes.contains("callback action(action_id: string);"));
    assert!(panes.contains("callback source_edited(value: string);"));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(panes.contains("component UiAssetSourceTextInput inherits TextInput {"));
    assert!(panes.contains("UiAssetSourceTextInput {"));
}

#[test]
fn ui_asset_editor_pane_genericizes_collection_event_boundary() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let _pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));

    for legacy_callback in [
        "callback ui_asset_theme_source_selected(instance_id: string, item_index: int);",
        "callback ui_asset_matched_style_rule_selected(instance_id: string, item_index: int);",
        "callback ui_asset_palette_selected(instance_id: string, item_index: int);",
        "callback ui_asset_palette_target_candidate_selected(instance_id: string, item_index: int);",
        "callback ui_asset_hierarchy_selected(instance_id: string, item_index: int);",
        "callback ui_asset_hierarchy_activated(instance_id: string, item_index: int);",
        "callback ui_asset_preview_selected(instance_id: string, item_index: int);",
        "callback ui_asset_preview_activated(instance_id: string, item_index: int);",
        "callback ui_asset_source_outline_selected(instance_id: string, item_index: int);",
        "callback ui_asset_preview_mock_selected(instance_id: string, item_index: int);",
        "callback ui_asset_binding_selected(instance_id: string, item_index: int);",
        "callback ui_asset_binding_event_selected(instance_id: string, item_index: int);",
        "callback ui_asset_binding_action_kind_selected(instance_id: string, item_index: int);",
        "callback ui_asset_binding_payload_selected(instance_id: string, item_index: int);",
        "callback ui_asset_slot_semantic_selected(instance_id: string, item_index: int);",
        "callback ui_asset_layout_semantic_selected(instance_id: string, item_index: int);",
    ] {
        assert!(
            !source.contains(legacy_callback),
            "root host should drop legacy UI asset collection callback `{legacy_callback}`"
        );
    }

    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    for legacy_callback in [
        "callback matched_style_rule_selected(item_index: int);",
        "callback palette_selected(item_index: int);",
        "callback palette_target_candidate_selected(item_index: int);",
        "callback hierarchy_selected(item_index: int);",
        "callback hierarchy_activated(item_index: int);",
        "callback preview_selected(item_index: int);",
        "callback preview_activated(item_index: int);",
        "callback source_outline_selected(item_index: int);",
        "callback preview_mock_selected(item_index: int);",
        "callback binding_selected(item_index: int);",
        "callback binding_event_selected(item_index: int);",
        "callback binding_action_kind_selected(item_index: int);",
        "callback binding_payload_selected(item_index: int);",
        "callback slot_semantic_selected(item_index: int);",
        "callback layout_semantic_selected(item_index: int);",
    ] {
        assert!(
            !pane_block.contains(legacy_callback),
            "UiAssetEditorPane should drop legacy collection callback `{legacy_callback}`"
        );
    }

    for pane_forwarder in [
        "root.collection_event(\"matched_style_rule\", \"selected\", item_index);",
        "root.collection_event(\"palette\", \"selected\", item_index);",
        "root.collection_event(\"palette_target_candidate\", \"selected\", item_index);",
        "root.collection_event(\"hierarchy\", \"selected\", item_index);",
        "root.collection_event(\"hierarchy\", \"activated\", item_index);",
        "root.collection_event(\"preview\", \"selected\", item_index);",
        "root.collection_event(\"preview\", \"activated\", item_index);",
        "root.collection_event(\"source_outline\", \"selected\", item_index);",
        "root.collection_event(\"preview_mock\", \"selected\", item_index);",
        "root.collection_event(\"binding\", \"selected\", item_index);",
        "root.collection_event(\"binding_event\", \"selected\", item_index);",
        "root.collection_event(\"binding_action_kind\", \"selected\", item_index);",
        "root.collection_event(\"binding_payload\", \"selected\", item_index);",
        "root.collection_event(\"slot_semantic\", \"selected\", item_index);",
        "root.collection_event(\"layout_semantic\", \"selected\", item_index);",
    ] {
        assert!(
            panes.contains(pane_forwarder),
            "UiAssetEditorPane should route collection events via `{pane_forwarder}`"
        );
    }

    assert!(!pane_surface.contains(
        "theme_source_selected(item_index) => { root.ui_asset_theme_source_selected(root.pane.id, item_index); }"
    ));
    assert!(panes.contains("root.action(\"theme.source.select.\" + item_index);"));
}

#[test]
fn ui_asset_editor_pane_groups_string_selection_properties() {
    let source = shell_source();
    let panes = panes_source();
    let pane_catalog = pane_surface_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(panes.contains("export struct UiAssetStringSelectionData {"));
    assert!(panes.contains("items: [string],"));
    assert!(panes.contains("selected_index: int,"));
    assert!(pane_catalog.contains("ui_asset: UiAssetEditorPaneData,"));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));

    for pane_property in [
        "property <UiAssetStringSelectionData> palette_collection: root.pane.palette_collection;",
        "property <UiAssetStringSelectionData> hierarchy_collection: root.pane.hierarchy_collection;",
        "property <UiAssetStringSelectionData> preview_collection: root.pane.preview_collection;",
        "property <UiAssetSourceDetailData> source_detail: root.pane.source_detail;",
        "property <UiAssetPreviewMockData> preview_mock: root.pane.preview_mock;",
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;",
        "property <UiAssetMatchedStyleRuleData> matched_style_rule: root.pane.matched_style_rule;",
        "property <UiAssetInspectorSlotData> inspector_slot: root.pane.inspector_slot;",
        "property <UiAssetInspectorLayoutData> inspector_layout: root.pane.inspector_layout;",
        "property <UiAssetInspectorBindingData> inspector_binding: root.pane.inspector_binding;",
    ] {
        assert!(
            pane_block.contains(pane_property),
            "UiAssetEditorPane should declare grouped selection property `{pane_property}`"
        );
    }

    for nested_selection_usage in [
        "items: root.source_detail.outline.items;",
        "selected_index: root.source_detail.outline.selected_index;",
        "items: root.preview_mock.collection.items;",
        "selected_index: root.preview_mock.collection.selected_index;",
        "items: root.theme_source.collection.items;",
        "selected_index: root.theme_source.collection.selected_index;",
        "items: root.matched_style_rule.collection.items;",
        "selected_index: root.matched_style_rule.collection.selected_index;",
        "items: root.inspector_slot.semantic.collection.items;",
        "selected_index: root.inspector_slot.semantic.collection.selected_index;",
        "items: root.inspector_layout.semantic.collection.items;",
        "selected_index: root.inspector_layout.semantic.collection.selected_index;",
        "items: root.inspector_binding.collection.items;",
        "selected_index: root.inspector_binding.collection.selected_index;",
        "items: root.inspector_binding.event_collection.items;",
        "selected_index: root.inspector_binding.event_collection.selected_index;",
        "items: root.inspector_binding.action_kind_collection.items;",
        "selected_index: root.inspector_binding.action_kind_collection.selected_index;",
        "items: root.inspector_binding.payload_collection.items;",
        "selected_index: root.inspector_binding.payload_collection.selected_index;",
    ] {
        assert!(
            panes.contains(nested_selection_usage),
            "UiAssetEditorPane should consume grouped selection data via `{nested_selection_usage}`"
        );
    }

    for legacy_property in [
        "in property <int> palette_selected_index;",
        "in property <int> hierarchy_selected_index;",
        "in property <int> preview_selected_index;",
        "in property <int> source_outline_selected_index;",
        "in property <int> preview_mock_selected_index;",
        "in property <int> theme_source_selected_index;",
        "in property <int> style_matched_rule_selected_index;",
        "in property <int> inspector_slot_semantic_selected_index;",
        "in property <int> inspector_layout_semantic_selected_index;",
        "in property <int> inspector_binding_selected_index;",
        "in property <int> inspector_binding_event_selected_index;",
        "in property <int> inspector_binding_action_kind_selected_index;",
        "in property <int> inspector_binding_payload_selected_index;",
    ] {
        assert!(
            !pane_block.contains(legacy_property),
            "UiAssetEditorPane should drop legacy selected-index property `{legacy_property}`"
        );
    }
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <bool> can_open_reference: root.pane.can_open_reference;"));

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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <string> preview_preset: root.pane.preview_preset;"));

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
    assert!(panes.contains("slot_target_items: [UiAssetCanvasSlotTargetData],"));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains(
        "property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"
    ));
    assert!(pane_block.contains("external_slot_target_items: root.palette_drag_projection.slot_target_items;"));

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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(source.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains(
        "property <UiAssetPreviewMockData> preview_mock: root.pane.preview_mock;"
    ));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(panes.contains("title: \"Mock Preview\";"));
    assert!(panes.contains("items: root.preview_mock.collection.items;"));
    assert!(panes.contains("selected_index: root.preview_mock.collection.selected_index;"));
    assert!(panes.contains("root.collection_event(\"preview_mock\", \"selected\", item_index);"));
    assert!(panes.contains("text: root.preview_mock.property;"));
    assert!(panes.contains("text: root.preview_mock.kind;"));
    assert!(panes.contains("root.preview_mock.value;"));
    assert!(panes.contains("root.detail_event(\"preview_mock\", \"preview.mock.value.set\""));
    assert!(panes.contains("root.detail_event(\"preview_mock\", \"preview.mock.clear\", root.preview_mock.collection.selected_index, \"\", \"\");"));
    assert!(panes.contains("title: \"Preview State Graph\";"));
    assert!(panes.contains("items: root.preview_mock.state_graph_items;"));
}

#[test]
fn ui_asset_editor_pane_groups_detail_contract_and_genericizes_detail_event_dispatch() {
    let source = shell_source();
    let panes = panes_source();
    let pane_catalog = pane_surface_source();
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    for grouped_struct in [
        "export struct UiAssetSourceDetailData {",
        "export struct UiAssetPreviewMockData {",
        "export struct UiAssetThemeSourceData {",
        "export struct UiAssetMatchedStyleRuleData {",
        "export struct UiAssetStyleRuleDeclarationData {",
        "export struct UiAssetStyleTokenData {",
        "export struct UiAssetInspectorSlotData {",
        "export struct UiAssetInspectorLayoutData {",
        "export struct UiAssetInspectorBindingData {",
        "export struct UiAssetInspectorWidgetData {",
        "export struct UiAssetEditorPaneData {",
    ] {
        assert!(
            panes.contains(grouped_struct),
            "UiAsset pane source should declare grouped detail struct `{grouped_struct}`"
        );
    }

    assert!(pane_catalog.contains("ui_asset: UiAssetEditorPaneData,"));
    for removed_flat_field in [
        "ui_asset_source_selected_block_label: string,",
        "ui_asset_preview_mock_property: string,",
        "ui_asset_theme_selected_source_reference: string,",
        "ui_asset_style_selected_rule_declaration_path: string,",
        "ui_asset_style_selected_token_name: string,",
        "ui_asset_inspector_slot_padding: string,",
        "ui_asset_inspector_binding_payload_key: string,",
    ] {
        assert!(
            !pane_catalog.contains(removed_flat_field),
            "PaneData should drop flattened UI asset detail field `{removed_flat_field}`"
        );
    }

    assert!(source.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    for removed_callback in [
        "callback ui_asset_inspector_widget_action(instance_id: string, action_id: string, value: string);",
        "callback ui_asset_style_rule_action(instance_id: string, action_id: string, item_index: int, selector: string);",
        "callback ui_asset_style_rule_declaration_action(instance_id: string, action_id: string, item_index: int, declaration_path: string, declaration_value: string);",
        "callback ui_asset_style_token_action(instance_id: string, action_id: string, item_index: int, token_name: string, token_value: string);",
        "callback ui_asset_preview_mock_action(instance_id: string, action_id: string, value: string);",
        "callback ui_asset_binding_payload_action(instance_id: string, action_id: string, payload_key: string, payload_value: string);",
    ] {
        assert!(
            !source.contains(removed_callback),
            "workbench shell should drop legacy detail callback `{removed_callback}`"
        );
    }

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));
    for removed_mapping in [
        "mode: root.pane.ui_asset_mode;",
        "preview_mock_property: root.pane.ui_asset_preview_mock_property;",
        "selected_theme_source_reference: root.pane.ui_asset_theme_selected_source_reference;",
        "selected_rule_declaration_path: root.pane.ui_asset_style_selected_rule_declaration_path;",
        "selected_token_name: root.pane.ui_asset_style_selected_token_name;",
        "inspector_slot_padding: root.pane.ui_asset_inspector_slot_padding;",
        "inspector_binding_payload_key: root.pane.ui_asset_inspector_binding_payload_key;",
        "inspector_widget_action(action_id, value) => { root.ui_asset_inspector_widget_action(root.pane.id, action_id, value); }",
        "style_rule_action(action_id, item_index, selector) => { root.ui_asset_style_rule_action(root.pane.id, action_id, item_index, selector); }",
        "style_rule_declaration_action(action_id, item_index, declaration_path, declaration_value) => { root.ui_asset_style_rule_declaration_action(root.pane.id, action_id, item_index, declaration_path, declaration_value); }",
        "style_token_action(action_id, item_index, token_name, token_value) => { root.ui_asset_style_token_action(root.pane.id, action_id, item_index, token_name, token_value); }",
        "preview_mock_action(action_id, value) => { root.ui_asset_preview_mock_action(root.pane.id, action_id, value); }",
        "binding_payload_action(action_id, payload_key, payload_value) => { root.ui_asset_binding_payload_action(root.pane.id, action_id, payload_key, payload_value); }",
    ] {
        assert!(
            !pane_surface.contains(removed_mapping),
            "PaneSurface should drop legacy detail mapping `{removed_mapping}`"
        );
    }

    assert!(pane_block.contains("in property <UiAssetEditorPaneData> pane;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    for grouped_property in [
        "property <UiAssetSourceDetailData> source_detail: root.pane.source_detail;",
        "property <UiAssetPreviewMockData> preview_mock: root.pane.preview_mock;",
        "property <UiAssetInspectorWidgetData> inspector_widget: root.pane.inspector_widget;",
        "property <UiAssetInspectorSlotData> inspector_slot: root.pane.inspector_slot;",
        "property <UiAssetInspectorLayoutData> inspector_layout: root.pane.inspector_layout;",
        "property <UiAssetInspectorBindingData> inspector_binding: root.pane.inspector_binding;",
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;",
        "property <UiAssetStyleRuleData> style_rule: root.pane.style_rule;",
        "property <UiAssetMatchedStyleRuleData> matched_style_rule: root.pane.matched_style_rule;",
        "property <UiAssetStyleRuleDeclarationData> style_rule_declaration: root.pane.style_rule_declaration;",
        "property <UiAssetStyleTokenData> style_token: root.pane.style_token;",
        "property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;",
    ] {
        assert!(
            pane_block.contains(grouped_property),
            "UiAssetEditorPane should keep grouped nested property `{grouped_property}`"
        );
    }
    for removed_scalar_alias in [
        "property <string> source_selected_block_label:",
        "property <int> source_selected_line:",
        "property <UiAssetStringSelectionData> preview_mock_collection:",
        "property <string> inspector_slot_padding:",
        "property <string> inspector_layout_box_gap:",
        "property <UiAssetStringSelectionData> inspector_binding_collection:",
        "property <string> selected_theme_source_reference:",
        "property <[string]> style_rule_items:",
        "property <[string]> style_rule_declaration_items:",
        "property <[string]> style_token_items:",
        "property <int> palette_drag_target_preview_index:",
        "property <string> palette_drag_target_action:",
    ] {
        assert!(
            !pane_block.contains(removed_scalar_alias),
            "UiAssetEditorPane should drop scalar detail alias `{removed_scalar_alias}`"
        );
    }
    for removed_callback in [
        "callback inspector_widget_action(action_id: string, value: string);",
        "callback style_rule_action(action_id: string, item_index: int, selector: string);",
        "callback style_rule_declaration_action(action_id: string, item_index: int, declaration_path: string, declaration_value: string);",
        "callback style_token_action(action_id: string, item_index: int, token_name: string, token_value: string);",
        "callback preview_mock_action(action_id: string, value: string);",
        "callback binding_payload_action(action_id: string, payload_key: string, payload_value: string);",
    ] {
        assert!(
            !pane_block.contains(removed_callback),
            "UiAssetEditorPane should drop legacy detail callback `{removed_callback}`"
        );
    }
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <bool> can_create_rule: root.pane.can_create_rule;"));
    assert!(pane_block.contains("property <bool> can_extract_rule: root.pane.can_extract_rule;"));
    assert!(pane_block.contains("property <bool> state_hover: root.pane.state_hover;"));
    assert!(pane_block.contains("property <bool> state_focus: root.pane.state_focus;"));
    assert!(pane_block.contains("property <bool> state_pressed: root.pane.state_pressed;"));
    assert!(pane_block.contains("property <bool> state_disabled: root.pane.state_disabled;"));
    assert!(pane_block.contains("property <bool> state_selected: root.pane.state_selected;"));
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
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "style_class_action(action_id, class_name) => { root.ui_asset_style_class_action(root.pane.id, action_id, class_name); }"
    ));

    assert!(pane_block.contains("property <[string]> style_class_items: root.pane.style_class_items;"));
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
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains("property <UiAssetStyleRuleData> style_rule: root.pane.style_rule;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("property <string> style_rule_selector_draft: \"\";"));
    assert!(panes.contains("title: \"Rules\";"));
    assert!(panes.contains("items: root.style_rule.items;"));
    assert!(panes.contains("selected_index: root.style_rule.selected_index;"));
    assert!(panes.contains("root.style_rule_selector_draft = root.style_rule.items[item_index];"));
    assert!(panes.contains("root.detail_event(\"style_rule\", \"style.rule.select\", item_index, \"\", \"\");"));
    assert!(panes.contains("placeholder: \"selector\";"));
    assert!(panes.contains("label: \"Apply\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.rename\", root.style_rule.selected_index, root.style_rule_selector_draft != \"\" ? root.style_rule_selector_draft : root.style_rule.selected_selector, \"\");"
    ));
    assert!(panes.contains("label: \"Delete\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.delete\", root.style_rule.selected_index, \"\", \"\");"
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
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains("property <UiAssetStyleTokenData> style_token: root.pane.style_token;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("property <string> style_token_name_draft: \"\";"));
    assert!(panes.contains("property <string> style_token_value_draft: \"\";"));
    assert!(panes.contains("title: \"Tokens\";"));
    assert!(panes.contains("items: root.style_token.items;"));
    assert!(panes.contains("selected_index: root.style_token.selected_index;"));
    assert!(panes.contains("root.style_token_name_draft = root.style_token.selected_name;"));
    assert!(panes.contains("root.style_token_value_draft = root.style_token.selected_value;"));
    assert!(panes.contains("placeholder: \"token-name\";"));
    assert!(panes.contains("placeholder: \"token-value\";"));
    assert!(panes.contains("label: \"Apply\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_token\", \"style.token.upsert\", root.style_token.selected_index, root.style_token_name_draft != \"\" ? root.style_token_name_draft : root.style_token.selected_name, root.style_token_value_draft != \"\" ? root.style_token_value_draft : root.style_token.selected_value);"
    ));
    assert!(panes.contains("label: \"Delete\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_token\", \"style.token.delete\", root.style_token.selected_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_theme_source_selection_and_inspection_controls() {
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains(
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"
    ));
    assert!(pane_block.contains("property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"));

    assert!(panes.contains("title: \"Theme Sources\";"));
    assert!(panes.contains("items: root.theme_source.collection.items;"));
    assert!(panes.contains("selected_index: root.theme_source.collection.selected_index;"));
    assert!(panes.contains("root.action(\"theme.source.select.\" + item_index);"));
    assert!(panes.contains("text: root.theme_source.selected_source_kind != \"\" ? root.theme_source.selected_source_kind + \" Theme\" : \"No theme source selected\";"));
    assert!(panes.contains("title: \"Theme Tokens\";"));
    assert!(panes.contains("items: root.theme_source.selected_source_token_items;"));
    assert!(panes.contains("title: \"Theme Rules\";"));
    assert!(panes.contains("items: root.theme_source.selected_source_rule_items;"));
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
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains("property <UiAssetStyleRuleDeclarationData> style_rule_declaration: root.pane.style_rule_declaration;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("property <string> style_rule_declaration_path_draft: \"\";"));
    assert!(panes.contains("property <string> style_rule_declaration_value_draft: \"\";"));
    assert!(panes.contains("title: \"Declarations\";"));
    assert!(panes.contains("items: root.style_rule_declaration.items;"));
    assert!(panes.contains("selected_index: root.style_rule_declaration.selected_index;"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule_declaration\", \"style.rule.declaration.select\", item_index, \"\", \"\");"
    ));
    assert!(panes
        .contains("root.style_rule_declaration_path_draft = root.style_rule_declaration.selected_path;"));
    assert!(panes.contains(
        "root.style_rule_declaration_value_draft = root.style_rule_declaration.selected_value;"
    ));
    assert!(panes.contains("placeholder: \"self.background.color\";"));
    assert!(panes.contains("placeholder: \"value\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule_declaration\", \"style.rule.declaration.upsert\", root.style_rule_declaration.selected_index, root.style_rule_declaration_path_draft != \"\" ? root.style_rule_declaration_path_draft : root.style_rule_declaration.selected_path, root.style_rule_declaration_value_draft != \"\" ? root.style_rule_declaration_value_draft : root.style_rule_declaration.selected_value);"
    ));
    assert!(panes.contains(
        "root.detail_event(\"style_rule_declaration\", \"style.rule.declaration.delete\", root.style_rule_declaration.selected_index, \"\", \"\");"
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
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(pane_block.contains(
        "property <UiAssetMatchedStyleRuleData> matched_style_rule: root.pane.matched_style_rule;"
    ));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));

    assert!(panes.contains("title: \"Matched Rules\";"));
    assert!(panes.contains("items: root.matched_style_rule.collection.items;"));
    assert!(panes.contains("selected_index: root.matched_style_rule.collection.selected_index;"));
    assert!(panes.contains(
        "root.collection_event(\"matched_style_rule\", \"selected\", item_index);"
    ));
    assert!(panes.contains("text: root.matched_style_rule.selected_origin != \"\" ? root.matched_style_rule.selected_origin : \"No matched rule selected\";"));
    assert!(panes.contains("text: root.matched_style_rule.selected_selector;"));
    assert!(panes.contains("text: root.matched_style_rule.selected_specificity >= 0 ? \"specificity \" + root.matched_style_rule.selected_specificity + \" • order \" + root.matched_style_rule.selected_source_order : \"\";"));
    assert!(panes.contains("for item in root.matched_style_rule.selected_declaration_items: Text {"));
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
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains(
        "property <UiAssetInspectorWidgetData> inspector_widget: root.pane.inspector_widget;"
    ));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("text: \"Widget\";"));
    assert!(panes.contains("text: \"Node\";"));
    assert!(panes.contains("text: \"Control Id\";"));
    assert!(panes.contains("text: \"Text\";"));
    assert!(panes.contains(
        "text: root.inspector_widget.selected_node_id != \"\" ? root.inspector_widget.selected_node_id : \"No selection\";"
    ));
    assert!(panes.contains("text: root.inspector_widget.control_id;"));
    assert!(panes.contains("text: root.inspector_widget.text_prop;"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"widget.control_id.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"widget.text.set\", -1, value, \"\");"));
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains(
        "property <UiAssetInspectorSlotData> inspector_slot: root.pane.inspector_slot;"
    ));

    assert!(panes.contains("text: \"Slot\";"));
    assert!(panes.contains("text: \"Mount\";"));
    assert!(panes.contains("text: \"Padding\";"));
    assert!(panes.contains("text: \"Width\";"));
    assert!(panes.contains("text: \"Height\";"));
    assert!(panes.contains("text: root.inspector_slot.padding;"));
    assert!(panes.contains("text: root.inspector_slot.width_preferred;"));
    assert!(panes.contains("text: root.inspector_slot.height_preferred;"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.mount.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.padding.set\", -1, value, \"\");"));
    assert!(
        panes.contains("root.detail_event(\"inspector_widget\", \"slot.layout.width.preferred.set\", -1, value, \"\");")
    );
    assert!(panes
        .contains("root.detail_event(\"inspector_widget\", \"slot.layout.height.preferred.set\", -1, value, \"\");"));
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains(
        "property <UiAssetInspectorLayoutData> inspector_layout: root.pane.inspector_layout;"
    ));

    assert!(panes.contains("text: \"Layout\";"));
    assert!(panes.contains("text: root.inspector_layout.width_preferred;"));
    assert!(panes.contains("text: root.inspector_layout.height_preferred;"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"layout.width.preferred.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"layout.height.preferred.set\", -1, value, \"\");"));
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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(pane_block.contains(
        "property <UiAssetInspectorSlotData> inspector_slot: root.pane.inspector_slot;"
    ));
    assert!(pane_block.contains(
        "property <UiAssetInspectorLayoutData> inspector_layout: root.pane.inspector_layout;"
    ));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));

    assert!(panes.contains("text: root.inspector_slot.semantic.title;"));
    assert!(panes.contains("text: root.inspector_layout.semantic.title;"));
    assert!(panes.contains("items: root.inspector_slot.semantic.collection.items;"));
    assert!(panes.contains("items: root.inspector_layout.semantic.collection.items;"));
    assert!(panes.contains("root.collection_event(\"slot_semantic\", \"selected\", item_index);"));
    assert!(panes.contains("root.collection_event(\"layout_semantic\", \"selected\", item_index);"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.semantic.value.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.semantic.delete\", -1, \"\", \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"layout.semantic.value.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"layout.semantic.delete\", -1, \"\", \"\");"));
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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(source.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { root.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains(
        "property <UiAssetInspectorBindingData> inspector_binding: root.pane.inspector_binding;"
    ));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("text: \"Bindings\";"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"binding.add\", -1, \"\", \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"binding.delete\", -1, \"\", \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"binding.id.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"binding.route.set\", -1, value, \"\");"));
    assert!(panes.contains("title: \"Event\";"));
    assert!(panes.contains("items: root.inspector_binding.event_collection.items;"));
    assert!(panes.contains("selected_index: root.inspector_binding.event_collection.selected_index;"));
    assert!(panes.contains("root.collection_event(\"binding_event\", \"selected\", item_index);"));
    assert!(panes.contains("title: \"Action Kind\";"));
    assert!(panes.contains("items: root.inspector_binding.action_kind_collection.items;"));
    assert!(panes.contains("selected_index: root.inspector_binding.action_kind_collection.selected_index;"));
    assert!(panes.contains(
        "root.collection_event(\"binding_action_kind\", \"selected\", item_index);"
    ));
    assert!(panes.contains("title: \"Payload\";"));
    assert!(panes.contains("items: root.inspector_binding.payload_collection.items;"));
    assert!(panes.contains("selected_index: root.inspector_binding.payload_collection.selected_index;"));
    assert!(panes.contains("root.collection_event(\"binding_payload\", \"selected\", item_index);"));
    assert!(panes.contains("root.detail_event(\"binding_payload\", \"binding.payload.upsert\""));
    assert!(panes.contains("root.detail_event(\"binding_payload\", \"binding.payload.delete\", root.inspector_binding.payload_collection.selected_index, \"\", \"\");"));
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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(pane_block.contains("property <UiAssetStringSelectionData> palette_collection: root.pane.palette_collection;"));
    assert!(pane_block.contains("property <UiAssetStringSelectionData> hierarchy_collection: root.pane.hierarchy_collection;"));
    assert!(pane_block.contains("property <UiAssetStringSelectionData> preview_collection: root.pane.preview_collection;"));
    assert!(pane_block.contains(
        "property <UiAssetSourceDetailData> source_detail: root.pane.source_detail;"
    ));
    assert!(pane_block.contains("property <float> preview_surface_width: root.pane.preview_surface_width;"));
    assert!(pane_block.contains("property <float> preview_surface_height: root.pane.preview_surface_height;"));
    assert!(pane_block.contains("property <[UiAssetCanvasNodeData]> preview_canvas_items: root.pane.preview_canvas_items;"));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));

    assert!(panes.contains("export struct UiAssetCanvasNodeData {"));
    assert!(panes.contains("component UiAssetCanvasSurface inherits Rectangle {"));
    assert!(panes.contains("text: \"Designer Canvas\";"));
    assert!(panes.contains("items: root.preview_canvas_items;"));
    assert!(panes.contains("surface_width: root.preview_surface_width;"));
    assert!(panes.contains("surface_height: root.preview_surface_height;"));
    assert!(panes.contains("title: \"Render Stack\";"));
    assert!(panes.contains("title: \"Source Outline\";"));
    assert!(panes.contains("selected_index: root.source_detail.outline.selected_index;"));
    assert!(panes.contains("root.collection_event(\"source_outline\", \"selected\", item_index);"));
    assert!(panes.contains("title: \"Palette\";"));
    assert!(panes.contains("selected_index: root.palette_collection.selected_index;"));
    assert!(panes.contains("root.collection_event(\"palette\", \"selected\", item_index);"));
    assert!(panes.contains("selected_index: root.hierarchy_collection.selected_index;"));
    assert!(panes.contains("selected_index: root.preview_collection.selected_index;"));
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
    assert!(panes.contains("text: root.source_detail.block_label != \"\" ? root.source_detail.block_label : \"No source block\";"));
    assert!(panes.contains(
        "text: root.source_detail.selected_line >= 0 ? \"line \" + root.source_detail.selected_line : \"\";"
    ));
    assert!(panes.contains("text: root.source_detail.roundtrip_status;"));
    assert!(panes.contains("text: root.source_detail.selected_excerpt;"));
    assert!(panes.contains("desired_cursor_byte_offset: root.source_detail.cursor_byte_offset;"));
    assert!(panes.contains(
        "changed desired_cursor_byte_offset => {\n        root.set-selection-offsets(root.desired_cursor_byte_offset, root.desired_cursor_byte_offset);\n    }"
    ));
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <bool> can_convert_to_reference: root.pane.can_convert_to_reference;"));
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <bool> can_extract_component: root.pane.can_extract_component;"));
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <bool> can_promote_to_external_widget: root.pane.can_promote_to_external_widget;"));
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains(
        "property <UiAssetInspectorWidgetData> inspector_widget: root.pane.inspector_widget;"
    ));

    assert!(panes.contains("text: \"Promote Draft\";"));
    assert!(panes.contains("text: \"Asset\";"));
    assert!(panes.contains("text: \"Comp\";"));
    assert!(panes.contains("text: \"Doc\";"));
    assert!(panes.contains("text: root.inspector_widget.promote_asset_id;"));
    assert!(panes.contains("text: root.inspector_widget.promote_component_name;"));
    assert!(panes.contains("text: root.inspector_widget.promote_document_id;"));
    assert!(panes.contains("enabled: root.inspector_widget.can_edit_promote_draft;"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"promote.asset_id.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"promote.component_name.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"promote.document_id.set\", -1, value, \"\");"));
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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(panes.contains("callback item_activated(item_index: int);"));
    assert!(panes.contains("double-clicked => {"));
    assert!(panes.contains("root.item_activated(index);"));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(panes.contains(
        "item_activated(item_index) => { root.collection_event(\"hierarchy\", \"activated\", item_index); }"
    ));
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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(panes.contains(
        "item_activated(item_index) => { root.collection_event(\"preview\", \"activated\", item_index); }"
    ));
    assert!(panes.contains("UiAssetCanvasSurface {"));
}

#[test]
fn ui_asset_editor_pane_declares_source_cursor_roundtrip_callback() {
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
        "callback ui_asset_source_cursor_changed(instance_id: string, byte_offset: int);"
    ));
    assert!(pane_surface.contains(
        "source_cursor_changed(byte_offset) => { root.ui_asset_source_cursor_changed(root.pane.id, byte_offset); }"
    ));
    assert!(pane_block.contains("callback source_cursor_changed(byte_offset: int);"));
    assert!(pane_block.contains(
        "property <UiAssetSourceDetailData> source_detail: root.pane.source_detail;"
    ));
    assert!(panes.contains("callback source_cursor_moved(byte_offset: int);"));
    assert!(panes.contains(
        "source_cursor_moved(byte_offset) => { root.source_cursor_changed(byte_offset); }"
    ));
    assert!(panes.contains(
        "init => {\n        root.set-selection-offsets(root.desired_cursor_byte_offset, root.desired_cursor_byte_offset);\n    }"
    ));
    assert!(panes.contains(
        "cursor-position-changed(_cursor_position) => {\n        root.source_cursor_moved(root.cursor-position-byte-offset);\n    }"
    ));
}

#[test]
fn ui_asset_editor_canvas_declares_selected_frame_authoring_overlay_controls() {
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

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <bool> can_insert_child: root.pane.can_insert_child;"));
    assert!(pane_block.contains("property <bool> can_insert_after: root.pane.can_insert_after;"));
    assert!(pane_block.contains("property <bool> can_move_up: root.pane.can_move_up;"));
    assert!(pane_block.contains("property <bool> can_move_down: root.pane.can_move_down;"));
    assert!(pane_block.contains("property <bool> can_reparent_into_previous: root.pane.can_reparent_into_previous;"));
    assert!(pane_block.contains("property <bool> can_reparent_into_next: root.pane.can_reparent_into_next;"));
    assert!(pane_block.contains("property <bool> can_reparent_outdent: root.pane.can_reparent_outdent;"));
    assert!(pane_block.contains("property <bool> can_wrap_in_vertical_box: root.pane.can_wrap_in_vertical_box;"));
    assert!(pane_block.contains("property <bool> can_unwrap: root.pane.can_unwrap;"));

    assert!(panes.contains("palette_has_selection: root.palette_collection.selected_index >= 0;"));
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
    assert!(pane_block.contains("property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"));
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
    assert!(pane_block.contains("if (root.palette_drag_projection.target_action != \"\") {"));
    assert!(pane_block.contains("root.palette_drag_dropped();"));
    assert!(pane_block.contains("root.palette_drag_cancelled();"));
    assert!(pane_block.contains("root.palette_drag_active = false;"));
    assert!(pane_block.contains("root.palette_drag_source_index = -1;"));

    assert!(pane_block.contains("drag_enabled: true;"));
    assert!(pane_block.contains("item_drag_started(item_index, x, y) => {"));
    assert!(pane_block.contains("root.collection_event(\"palette\", \"selected\", item_index);"));
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
        pane_block.contains("external_drag_target_index: root.palette_drag_projection.target_preview_index;")
    );
    assert!(pane_block.contains("external_drag_target_action: root.palette_drag_projection.target_action;"));
    assert!(pane_block.contains("external_drag_target_label: root.palette_drag_projection.target_label;"));
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

    assert!(pane_block.contains("property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"));
    assert!(pane_block.contains("title: \"Target Cycle\";"));
    assert!(pane_block.contains("items: root.palette_drag_projection.candidate_items;"));
    assert!(pane_block.contains("selected_index: root.palette_drag_projection.candidate_selected_index;"));

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

    assert!(source.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(source.contains("callback ui_asset_palette_target_confirm(instance_id: string);"));
    assert!(source.contains("callback ui_asset_palette_target_cancel(instance_id: string);"));
    assert!(pane_block.contains("property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { root.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_surface.contains(
        "palette_target_confirm() => { root.ui_asset_palette_target_confirm(root.pane.id); }"
    ));
    assert!(pane_surface.contains(
        "palette_target_cancel() => { root.ui_asset_palette_target_cancel(root.pane.id); }"
    ));

    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_block.contains("callback palette_target_confirm();"));
    assert!(pane_block.contains("callback palette_target_cancel();"));
    assert!(panes.contains(
        "if root.palette_drag_projection.candidate_items.length > 1 && (root.palette_drag_active || root.palette_drag_projection.target_chooser_active): Rectangle {"
    ));
    assert!(panes.contains(
        "root.collection_event(\"palette_target_candidate\", \"selected\", item_index);"
    ));
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
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_block.contains(
        "property <UiAssetInspectorSlotData> inspector_slot: root.pane.inspector_slot;"
    ));
    assert!(pane_block.contains(
        "property <UiAssetInspectorLayoutData> inspector_layout: root.pane.inspector_layout;"
    ));
    assert!(pane_block.contains(
        "property <UiAssetInspectorBindingData> inspector_binding: root.pane.inspector_binding;"
    ));

    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"layout.box.gap.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.overlay.anchor_x.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.grid.row.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"slot.flow.alignment.set\", -1, value, \"\");"));
    assert!(
        panes.contains("root.detail_event(\"inspector_widget\", \"slot.linear.width_weight.set\", -1, value, \"\");")
    );
    assert!(
        panes.contains("root.detail_event(\"inspector_widget\", \"slot.linear.width_stretch.set\", -1, value, \"\");")
    );
    assert!(
        panes.contains("root.detail_event(\"inspector_widget\", \"slot.linear.height_weight.set\", -1, value, \"\");")
    );
    assert!(
        panes.contains("root.detail_event(\"inspector_widget\", \"slot.linear.height_stretch.set\", -1, value, \"\");")
    );
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"layout.scroll.axis.set\", -1, value, \"\");"));
    assert!(panes.contains(
        "root.detail_event(\"inspector_widget\", \"layout.scroll.scrollbar_visibility.set\", -1, value, \"\");"
    ));
    assert!(pane_block.contains(
        "property <UiAssetInspectorSlotData> inspector_slot: root.pane.inspector_slot;"
    ));
    assert!(pane_block.contains(
        "property <UiAssetInspectorLayoutData> inspector_layout: root.pane.inspector_layout;"
    ));
    assert!(pane_block.contains(
        "property <UiAssetInspectorBindingData> inspector_binding: root.pane.inspector_binding;"
    ));
    assert!(panes.contains("text: root.inspector_layout.box_gap;"));
    assert!(panes.contains("text: root.inspector_slot.linear_main_weight;"));
    assert!(panes.contains("text: root.inspector_binding.binding_route_target;"));
    assert!(panes.contains("text: root.inspector_binding.binding_action_target;"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"binding.route_target.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"inspector_widget\", \"binding.action_target.set\", -1, value, \"\");"));
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

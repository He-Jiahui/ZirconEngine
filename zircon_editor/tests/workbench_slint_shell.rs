use std::fs;
use std::path::PathBuf;

fn root_shell_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench.slint");
    fs::read_to_string(path).expect("workbench.slint should be readable")
}

fn host_scaffold_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/host_scaffold.slint");
    fs::read_to_string(path).expect("host_scaffold.slint should be readable")
}

fn host_surface_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/host_surface.slint");
    fs::read_to_string(path).expect("host_surface.slint should be readable")
}

fn host_components_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/host_components.slint");
    fs::read_to_string(path).expect("host_components.slint should be readable")
}

fn host_interaction_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/host_interaction.slint");
    fs::read_to_string(path).expect("host_interaction.slint should be readable")
}

fn host_context_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/host_context.slint");
    fs::read_to_string(path).expect("host_context.slint should be readable")
}

fn shell_source() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        root_shell_source(),
        host_scaffold_source(),
        host_surface_source(),
        host_components_source(),
        host_interaction_source(),
        host_context_source()
    )
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
    assert!(center_band.contains("x: root.host_layout.center_band_frame.x * 1px;"));
    assert!(center_band.contains("y: root.host_layout.center_band_frame.y * 1px;"));
    assert!(center_band.contains("width: root.host_layout.center_band_frame.width * 1px;"));
    assert!(center_band.contains("height: root.host_layout.center_band_frame.height * 1px;"));

    let right_region = block_after(
        &source,
        "if root.surface_orchestration_data.right_stack_width_px > 0.0: HostSideDockSurface {",
    );
    assert!(right_region.contains("surface_data: root.right_dock_data;"));

    let side_dock_surface = scoped_block_after(
        &source,
        "export component HostSideDockSurface inherits Rectangle {",
    );
    assert!(side_dock_surface.contains("x: root.surface_data.region_frame.x * 1px;"));
    assert!(side_dock_surface.contains("y: root.surface_data.region_frame.y * 1px;"));
    assert!(side_dock_surface.contains("width: root.surface_data.region_frame.width * 1px;"));
    assert!(side_dock_surface.contains("height: root.surface_data.region_frame.height * 1px;"));

    let bottom_region = block_after(
        &source,
        "if root.surface_orchestration_data.bottom_panel_height_px > 0.0: HostBottomDockSurface {",
    );
    assert!(bottom_region.contains("surface_data: root.bottom_dock_data;"));

    let bottom_dock_surface = scoped_block_after(
        &source,
        "export component HostBottomDockSurface inherits Rectangle {",
    );
    assert!(bottom_dock_surface.contains("x: root.surface_data.region_frame.x * 1px;"));
    assert!(bottom_dock_surface.contains("y: root.surface_data.region_frame.y * 1px;"));
    assert!(bottom_dock_surface.contains("width: root.surface_data.region_frame.width * 1px;"));
    assert!(bottom_dock_surface.contains("height: root.surface_data.region_frame.height * 1px;"));

    let resize_layer = block_after(&source, "HostResizeLayer {");
    assert!(resize_layer.contains("resize_data: root.resize_layer_data;"));
    assert!(!resize_layer.contains("resize_state <=> WorkbenchHostContext.resize_state;"));

    let resize_layer_component = scoped_block_after(
        &source,
        "export component HostResizeLayer inherits Rectangle {",
    );
    assert!(resize_layer_component.contains("x: root.resize_data.left_splitter_frame.x * 1px;"));
    assert!(resize_layer_component.contains("y: root.resize_data.left_splitter_frame.y * 1px;"));
    assert!(resize_layer_component.contains("width: root.resize_data.left_splitter_frame.width * 1px;"));
    assert!(resize_layer_component.contains("height: root.resize_data.left_splitter_frame.height * 1px;"));
    assert!(resize_layer_component.contains("x: root.resize_data.right_splitter_frame.x * 1px;"));
    assert!(resize_layer_component.contains("y: root.resize_data.right_splitter_frame.y * 1px;"));
    assert!(resize_layer_component.contains("width: root.resize_data.right_splitter_frame.width * 1px;"));
    assert!(resize_layer_component.contains("height: root.resize_data.right_splitter_frame.height * 1px;"));
    assert!(resize_layer_component.contains("x: root.resize_data.bottom_splitter_frame.x * 1px;"));
    assert!(resize_layer_component.contains("y: root.resize_data.bottom_splitter_frame.y * 1px;"));
    assert!(resize_layer_component.contains("width: root.resize_data.bottom_splitter_frame.width * 1px;"));
    assert!(resize_layer_component.contains("height: root.resize_data.bottom_splitter_frame.height * 1px;"));
    assert!(resize_layer_component.contains("WorkbenchHostContext.resize_state.resize_active"));
    assert!(resize_layer_component.contains("WorkbenchHostContext.resize_state.resize_group"));

    let status_bar = block_after(&source, "status_bar_zone := Rectangle {");
    assert!(status_bar.contains("x: root.status_data.status_bar_frame.x * 1px;"));
    assert!(status_bar.contains("y: root.status_data.status_bar_frame.y * 1px;"));
    assert!(status_bar.contains("width: root.status_data.status_bar_frame.width * 1px;"));
    assert!(status_bar.contains("height: root.status_data.status_bar_frame.height * 1px;"));
}

#[test]
fn workbench_shell_declares_native_resize_and_maximize_bounds() {
    let source = root_shell_source();
    let shell_block =
        scoped_block_after(&source, "export component UiHostWindow inherits Window {");

    assert!(shell_block.contains("no-frame: false;"));
    assert!(shell_block.contains("resize-border-width: 8px;"));
    assert!(shell_block.contains("max-width:"));
    assert!(shell_block.contains("max-height:"));
}

#[test]
fn ui_host_window_root_delegates_to_internal_scaffold_only() {
    let source = root_shell_source();
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
    let source = root_shell_source();
    let scaffold = host_scaffold_source();
    let host_surface = host_surface_source();
    let host_components = host_components_source();
    let host_context = host_context_source();
    let pane_surface = pane_surface_source();
    let callback_wiring = slint_host_source("src/ui/slint_host/app/callback_wiring.rs");
    let apply_presentation = apply_presentation_source();
    let pointer_layout = slint_host_source("src/ui/slint_host/app/pointer_layout.rs");
    let host_lifecycle = slint_host_source("src/ui/slint_host/app/host_lifecycle.rs");

    assert!(
        source.contains("import { WorkbenchHostContext } from \"workbench/host_context.slint\";")
    );
    assert!(
        source.contains("import { WorkbenchHostScaffold } from \"workbench/host_scaffold.slint\";")
    );
    assert!(source.contains(
        "import { HostWindowShellData, HostWindowLayoutData, HostWindowSurfaceData } from \"workbench/host_components.slint\";"
    ));
    assert!(
        source.contains("import { PaneSurfaceHostContext } from \"workbench/pane_surface.slint\";")
    );
    assert!(
        source.contains("export { WorkbenchHostContext } from \"workbench/host_context.slint\";")
    );
    assert!(
        source.contains("export { PaneSurfaceHostContext } from \"workbench/pane_surface.slint\";")
    );
    assert!(!source.contains("import { AssetBrowserPane } from \"workbench/assets.slint\";"));
    assert!(!source.contains(
        "import { RecentProjectData, WelcomePane, WelcomePaneData } from \"workbench/welcome.slint\";"
    ));
    assert!(!source.contains(
        "import { DockTabButton, TabChip, ToolbarButton } from \"workbench/chrome.slint\";"
    ));
    assert!(!source.contains("import { PaneSurface } from \"workbench/pane_surface.slint\";"));
    assert!(!source.contains("component PaneSurface inherits Rectangle {"));
    assert!(!source.contains("export global PaneSurfaceHostContext {"));
    assert!(!source.contains("export component WorkbenchHostScaffold inherits Rectangle {"));
    assert!(!source.contains("callback pane_surface_control_clicked <=>"));
    assert!(!source.contains("callback ui_asset_palette_drag_cancel <=>"));
    assert!(!source.contains(
        "pane_surface_control_clicked(control_id, action_id) => { root.pane_surface_control_clicked(control_id, action_id); }"
    ));
    assert!(!source.contains(
        "hierarchy_pointer_clicked(x, y, width, height) => { root.hierarchy_pointer_clicked(x, y, width, height); }"
    ));
    assert!(!source.contains(
        "in property <WelcomePaneData> welcome_pane <=> PaneSurfaceHostContext.welcome_pane;"
    ));
    assert!(!source.contains(
        "in-out property <string> mesh_import_path <=> PaneSurfaceHostContext.mesh_import_path;"
    ));
    assert!(!source
        .contains("in property <string> status_primary <=> PaneSurfaceHostContext.status_text;"));
    assert!(!source
        .contains("in property <bool> delete_enabled <=> PaneSurfaceHostContext.delete_enabled;"));
    assert!(!source.contains(
        "import { AssetFolderData, AssetItemData, AssetReferenceData, AssetSelectionData, ProjectOverviewData } from \"workbench/assets.slint\";"
    ));
    assert!(!source.contains("import { SceneNodeData } from \"workbench/panes.slint\";"));
    assert!(!source.contains(
        "import { RecentProjectData, WelcomePaneData } from \"workbench/welcome.slint\";"
    ));
    assert!(!source.contains("in property <[BreadcrumbData]> breadcrumbs <=> host.breadcrumbs;"));
    assert!(source.contains(
        "in property <HostWindowSurfaceData> host_surface_data <=> host.host_surface_data;"
    ));
    assert!(source.contains("in property <HostWindowShellData> host_shell <=> host.host_shell;"));
    assert!(source.contains("in property <HostWindowLayoutData> host_layout <=> host.host_layout;"));
    for removed_root_host_proxy in [
        "in property <[TabData]> host_tabs <=> host.host_tabs;",
        "in property <[TabData]> left_tabs <=> host.left_tabs;",
        "in property <[TabData]> right_tabs <=> host.right_tabs;",
        "in property <[TabData]> bottom_tabs <=> host.bottom_tabs;",
        "in property <[TabData]> document_tabs <=> host.document_tabs;",
        "in property <[FloatingWindowData]> floating_windows <=> host.floating_windows;",
        "in property <PaneData> left_pane <=> host.left_pane;",
        "in property <PaneData> right_pane <=> host.right_pane;",
        "in property <PaneData> bottom_pane <=> host.bottom_pane;",
        "in property <PaneData> document_pane <=> host.document_pane;",
        "in property <string> project_path <=> host.project_path;",
        "in property <string> status_secondary <=> host.status_secondary;",
        "in property <string> viewport_label <=> host.viewport_label;",
        "in property <bool> drawers_visible <=> host.drawers_visible;",
        "in property <bool> left_expanded <=> host.left_expanded;",
        "in property <bool> right_expanded <=> host.right_expanded;",
        "in property <bool> bottom_expanded <=> host.bottom_expanded;",
        "in property <bool> save_project_enabled <=> host.save_project_enabled;",
        "in property <bool> undo_enabled <=> host.undo_enabled;",
        "in property <bool> redo_enabled <=> host.redo_enabled;",
        "in property <[string]> preset_names <=> host.preset_names;",
        "in property <string> active_preset_name <=> host.active_preset_name;",
        "in property <float> shell_min_width_px <=> host.shell_min_width_px;",
        "in property <float> shell_min_height_px <=> host.shell_min_height_px;",
        "in property <bool> native_floating_window_mode <=> host.native_floating_window_mode;",
        "in property <string> native_floating_window_id <=> host.native_floating_window_id;",
        "in property <string> native_window_title <=> host.native_window_title;",
        "in property <FrameRect> native_window_bounds <=> host.native_window_bounds;",
        "in property <FrameRect> center_band_frame <=> host.center_band_frame;",
        "in property <FrameRect> status_bar_frame <=> host.status_bar_frame;",
        "in property <FrameRect> left_region_frame <=> host.left_region_frame;",
        "in property <FrameRect> document_region_frame <=> host.document_region_frame;",
        "in property <FrameRect> right_region_frame <=> host.right_region_frame;",
        "in property <FrameRect> bottom_region_frame <=> host.bottom_region_frame;",
        "in property <FrameRect> left_splitter_frame <=> host.left_splitter_frame;",
        "in property <FrameRect> right_splitter_frame <=> host.right_splitter_frame;",
        "in property <FrameRect> bottom_splitter_frame <=> host.bottom_splitter_frame;",
        "in property <FrameRect> viewport_content_frame <=> host.viewport_content_frame;",
    ] {
        assert!(
            !source.contains(removed_root_host_proxy),
            "UiHostWindow root should not keep scattered host-shell proxy `{removed_root_host_proxy}`"
        );
    }
    for removed_root_abi in [
        "in-out property <int> open_menu_index",
        "in-out property <int> hovered_menu_index",
        "in-out property <int> hovered_menu_item_index",
        "in-out property <float> window_menu_scroll_px",
        "in-out property <float> window_menu_popup_height_px",
        "in-out property <string> active_drag_target_group",
        "in-out property <bool> drag_active",
        "in-out property <string> drag_tab_id",
        "in-out property <string> drag_tab_title",
        "in-out property <string> drag_tab_icon_key",
        "in-out property <string> drag_source_group",
        "in-out property <float> drag_pointer_x",
        "in-out property <float> drag_pointer_y",
        "callback menu_pointer_clicked(",
        "callback menu_pointer_moved(",
        "callback menu_pointer_scrolled(",
        "callback activity_rail_pointer_clicked(",
        "callback host_page_pointer_clicked(",
        "callback drawer_header_pointer_clicked(",
        "callback document_tab_pointer_clicked(",
        "callback document_tab_close_pointer_clicked(",
        "callback floating_window_header_pointer_clicked(",
        "callback workbench_drag_pointer_event(",
        "callback workbench_resize_pointer_event(",
        "callback asset_control_changed(",
        "callback asset_control_clicked(",
        "callback welcome_control_changed(",
        "callback welcome_control_clicked(",
        "callback viewport_toolbar_pointer_clicked(",
    ] {
        assert!(
            !source.contains(removed_root_abi),
            "UiHostWindow root should not keep workbench host interaction ABI `{removed_root_abi}`"
        );
    }
    for removed_root_probe in [
        "private property <FrameRect> file_menu_button_frame:",
        "private property <FrameRect> edit_menu_button_frame:",
        "private property <FrameRect> selection_menu_button_frame:",
        "private property <FrameRect> view_menu_button_frame:",
        "private property <FrameRect> window_menu_button_frame:",
        "private property <FrameRect> help_menu_button_frame:",
        "shared_pointer_hook_probes := Rectangle {",
        "shared_menu_anchor_probes := Rectangle {",
        "shared_welcome_control_probe := WelcomePane {",
    ] {
        assert!(
            !source.contains(removed_root_probe),
            "UiHostWindow root should not keep probe-only baggage `{removed_root_probe}`"
        );
    }

    assert!(scaffold.contains(
        "import { HostWindowShellData, HostWindowLayoutData, HostWindowSurfaceData } from \"host_components.slint\";"
    ));
    assert!(scaffold.contains(
        "import { HostWorkbenchWindowSurface, HostNativeWorkbenchWindowSurface } from \"host_surface.slint\";"
    ));
    assert!(scaffold.contains("export component WorkbenchHostScaffold inherits Rectangle {"));
    assert!(!scaffold.contains("import { WorkbenchHostContext } from \"host_context.slint\";"));
    assert!(!scaffold.contains(
        "in property <WelcomePaneData> welcome_pane <=> PaneSurfaceHostContext.welcome_pane;"
    ));
    assert!(!scaffold.contains(
        "in-out property <string> mesh_import_path <=> PaneSurfaceHostContext.mesh_import_path;"
    ));
    assert!(!scaffold.contains("in property <[BreadcrumbData]> breadcrumbs;"));
    assert!(!scaffold
        .contains("in property <string> status_primary <=> PaneSurfaceHostContext.status_text;"));
    assert!(!scaffold
        .contains("in property <bool> delete_enabled <=> PaneSurfaceHostContext.delete_enabled;"));
    assert!(scaffold.contains("in property <HostWindowSurfaceData> host_surface_data;"));
    assert!(scaffold.contains("in property <HostWindowShellData> host_shell: {"));
    assert!(scaffold.contains("in property <HostWindowLayoutData> host_layout: {"));
    for removed_scaffold_abi in [
        "in-out property <int> open_menu_index",
        "in-out property <int> hovered_menu_index",
        "in-out property <int> hovered_menu_item_index",
        "in-out property <float> window_menu_scroll_px",
        "in-out property <float> window_menu_popup_height_px",
        "in-out property <string> active_drag_target_group",
        "in-out property <bool> drag_active",
        "in-out property <string> drag_tab_id",
        "in-out property <string> drag_tab_title",
        "in-out property <string> drag_tab_icon_key",
        "in-out property <string> drag_source_group",
        "in-out property <float> drag_pointer_x",
        "in-out property <float> drag_pointer_y",
        "callback menu_pointer_clicked(",
        "callback menu_pointer_moved(",
        "callback menu_pointer_scrolled(",
        "callback activity_rail_pointer_clicked(",
        "callback host_page_pointer_clicked(",
        "callback drawer_header_pointer_clicked(",
        "callback document_tab_pointer_clicked(",
        "callback document_tab_close_pointer_clicked(",
        "callback floating_window_header_pointer_clicked(",
        "callback workbench_drag_pointer_event(",
        "callback workbench_resize_pointer_event(",
    ] {
        assert!(
            !scaffold.contains(removed_scaffold_abi),
            "WorkbenchHostScaffold should not keep host interaction ABI `{removed_scaffold_abi}`"
        );
    }
    for removed_scaffold_proxy in [
        "in-out property <FrameRect> file_menu_button_frame",
        "in-out property <FrameRect> edit_menu_button_frame",
        "in-out property <FrameRect> selection_menu_button_frame",
        "in-out property <FrameRect> view_menu_button_frame",
        "in-out property <FrameRect> window_menu_button_frame",
        "in-out property <FrameRect> help_menu_button_frame",
    ] {
        assert!(
            !scaffold.contains(removed_scaffold_proxy),
            "WorkbenchHostScaffold should not proxy host menu anchor frame `{removed_scaffold_proxy}`"
        );
    }
    assert!(!scaffold.contains("menu_button_row := HorizontalLayout {"));
    assert!(!scaffold.contains("for page[index] in root.host_tabs: DockTabButton {"));
    assert!(!scaffold.contains("document_zone := Rectangle {"));
    assert!(!scaffold.contains("status_bar_zone := Rectangle {"));
    assert!(
        !scaffold.contains("if root.left_splitter_frame.width > 0.0: left_splitter := Rectangle {")
    );
    assert!(!scaffold
        .contains("if root.right_splitter_frame.width > 0.0: right_splitter := Rectangle {"));
    assert!(!scaffold
        .contains("if root.bottom_splitter_frame.height > 0.0: bottom_splitter := Rectangle {"));
    assert!(!scaffold.contains(
        "for window[index] in root.floating_windows: floating_window_card := Rectangle {"
    ));
    assert!(!scaffold.contains("if root.drag_active: Rectangle {"));
    assert!(!scaffold.contains("if root.resize_active: TouchArea {"));
    assert!(!scaffold.contains("PaneSurface {"));
    assert!(!scaffold.contains("HostMenuChrome {"));
    assert!(!scaffold.contains("HostPageChrome {"));
    assert!(!scaffold.contains("HostDocumentDockSurface {"));
    assert!(!scaffold.contains("HostSideDockSurface {"));
    assert!(!scaffold.contains("HostBottomDockSurface {"));
    assert!(!scaffold.contains("HostStatusBar {"));
    assert!(!scaffold.contains("HostResizeLayer {"));
    assert!(!scaffold.contains("HostFloatingWindowLayer {"));
    assert!(!scaffold.contains("HostNativeFloatingWindowSurface {"));
    assert!(!scaffold.contains("WorkbenchHostContext.menu_pointer_clicked("));
    assert!(!scaffold.contains("WorkbenchHostContext.document_tab_pointer_clicked("));
    assert!(!scaffold.contains("if WorkbenchHostContext.drag_active: HostTabDragOverlay {"));
    assert!(scaffold
        .contains("if !root.host_shell.native_floating_window_mode: HostWorkbenchWindowSurface {"));
    assert!(scaffold.contains(
        "if root.host_shell.native_floating_window_mode: HostNativeWorkbenchWindowSurface {"
    ));
    assert!(scaffold.contains("host_surface_data: root.host_surface_data;"));
    for removed_scaffold_surface_proxy in [
        "host_tabs: root.host_surface_data.host_tabs;",
        "left_tabs: root.host_surface_data.left_tabs;",
        "right_tabs: root.host_surface_data.right_tabs;",
        "bottom_tabs: root.host_surface_data.bottom_tabs;",
        "document_tabs: root.host_surface_data.document_tabs;",
        "floating_windows: root.host_surface_data.floating_windows;",
        "left_pane: root.host_surface_data.left_pane;",
        "right_pane: root.host_surface_data.right_pane;",
        "bottom_pane: root.host_surface_data.bottom_pane;",
        "document_pane: root.host_surface_data.document_pane;",
    ] {
        assert!(
            !scaffold.contains(removed_scaffold_surface_proxy),
            "WorkbenchHostScaffold should not keep grouped surface fan-out `{removed_scaffold_surface_proxy}`"
        );
    }
    assert!(!scaffold.contains("welcome_pane: root.welcome_pane;"));
    assert!(!scaffold.contains("mesh_import_path: root.mesh_import_path;"));

    assert!(host_surface.contains("import { PaneSurfaceHostContext } from \"pane_surface.slint\";"));
    assert!(
        host_surface.contains("export component HostWorkbenchWindowSurface inherits Rectangle {")
    );
    assert!(host_surface
        .contains("export component HostNativeWorkbenchWindowSurface inherits Rectangle {"));
    assert!(host_surface.contains("in property <HostWindowSurfaceData> host_surface_data;"));
    assert!(host_surface.contains("in property <HostWindowShellData> host_shell: {"));
    assert!(host_surface.contains("in property <HostWindowLayoutData> host_layout: {"));
    for removed_host_surface_passthrough in [
        "in property <[TabData]> host_tabs:",
        "in property <[TabData]> left_tabs:",
        "in property <[TabData]> right_tabs:",
        "in property <[TabData]> bottom_tabs:",
        "in property <[TabData]> document_tabs:",
        "in property <[FloatingWindowData]> floating_windows:",
        "in property <PaneData> left_pane:",
        "in property <PaneData> right_pane:",
        "in property <PaneData> bottom_pane:",
        "in property <PaneData> document_pane:",
    ] {
        assert!(
            !host_surface.contains(removed_host_surface_passthrough),
            "host_surface should not reopen grouped surface payload `{removed_host_surface_passthrough}`"
        );
    }
    assert!(host_surface.contains("page_data: root.page_chrome_data;"));
    assert!(host_surface.contains("surface_data: root.document_dock_data;"));
    assert!(host_surface.contains("layer_data: root.floating_layer_data;"));
    assert!(host_surface.contains("status_data: root.status_bar_data;"));
    assert!(host_surface.contains("resize_data: root.resize_layer_data;"));
    assert!(host_surface.contains("HostMenuChrome {"));
    assert!(host_surface.contains("HostPageChrome {"));
    assert!(host_surface.contains("HostDocumentDockSurface {"));
    assert!(host_surface.contains("HostSideDockSurface {"));
    assert!(host_surface.contains("HostBottomDockSurface {"));
    assert!(host_surface.contains("HostStatusBar {"));
    assert!(host_surface.contains("HostResizeLayer {"));
    assert!(host_surface.contains("HostFloatingWindowLayer {"));
    assert!(host_surface.contains("HostNativeFloatingWindowSurface {"));
    assert!(!host_surface.contains("import { WorkbenchHostContext } from \"host_context.slint\";"));
    assert!(!host_surface.contains("WorkbenchHostContext.menu_pointer_clicked("));
    assert!(!host_surface.contains("WorkbenchHostContext.document_tab_pointer_clicked("));
    assert!(!host_surface.contains("if WorkbenchHostContext.drag_state.drag_active: HostTabDragOverlay {"));

    assert!(
        host_components.contains("import { PaneSurface, PaneData } from \"pane_surface.slint\";")
    );
    assert!(host_components.contains(
        "import { WorkbenchHostContext } from \"host_context.slint\";"
    ));
    assert!(host_components.contains("export struct HostWindowShellData {"));
    assert!(host_components.contains("export struct HostWindowLayoutData {"));
    assert!(host_components.contains("export struct HostWindowSurfaceData {"));
    assert!(host_components.contains("export component HostMenuChrome inherits Rectangle {"));
    assert!(host_components.contains("export component HostPageChrome inherits Rectangle {"));
    assert!(
        host_components.contains("export component HostDocumentDockSurface inherits Rectangle {")
    );
    assert!(host_components.contains("export component HostSideDockSurface inherits Rectangle {"));
    assert!(host_components.contains("export component HostBottomDockSurface inherits Rectangle {"));
    assert!(host_components.contains("export component HostStatusBar inherits Rectangle {"));
    assert!(host_components.contains("export component HostResizeLayer inherits Rectangle {"));
    assert!(
        host_components.contains("export component HostFloatingWindowLayer inherits Rectangle {")
    );
    assert!(host_components
        .contains("export component HostNativeFloatingWindowSurface inherits Rectangle {"));
    assert!(host_components.contains("export component HostTabDragOverlay inherits Rectangle {"));
    assert!(host_components.matches("PaneSurface {").count() >= 5);

    assert!(host_context.contains("export global WorkbenchHostContext {"));
    assert!(host_context.contains("in-out property <HostMenuStateData> menu_state: {"));
    assert!(host_context.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(host_context.contains("in-out property <HostResizeStateData> resize_state: {"));
    assert!(host_context.contains("callback menu_pointer_clicked(x: float, y: float);"));
    assert!(host_context
        .contains("callback workbench_drag_pointer_event(kind: int, x: float, y: float);"));
    assert!(host_context
        .contains("callback workbench_resize_pointer_event(kind: int, x: float, y: float);"));

    assert!(pane_surface.contains("export global PaneSurfaceHostContext {"));
    assert!(pane_surface.contains("component PaneSurface inherits Rectangle {"));
    assert!(pane_surface.contains(
        "in property <WelcomePaneData> welcome_pane <=> PaneSurfaceHostContext.welcome_pane;"
    ));
    for removed_root_registration in [
        "ui.on_asset_control_changed(",
        "ui.on_asset_control_clicked(",
        "ui.on_welcome_control_changed(",
        "ui.on_welcome_control_clicked(",
        "ui.on_viewport_toolbar_pointer_clicked(",
    ] {
        assert!(
            !callback_wiring.contains(removed_root_registration),
            "callback wiring should not keep root registration `{removed_root_registration}`"
        );
    }
    for pane_surface_registration in [
        "pane_surface_host.on_asset_control_changed(",
        "pane_surface_host.on_asset_control_clicked(",
        "pane_surface_host.on_welcome_control_changed(",
        "pane_surface_host.on_welcome_control_clicked(",
        "pane_surface_host.on_viewport_toolbar_pointer_clicked(",
    ] {
        assert!(
            callback_wiring.contains(pane_surface_registration),
            "callback wiring is missing pane-surface registration `{pane_surface_registration}`"
        );
    }
    assert!(apply_presentation.contains(
        "let pane_surface_host = ui.global::<crate::ui::slint_host::PaneSurfaceHostContext>();"
    ));
    for removed_root_pane_setter in [
        "ui.set_breadcrumbs(",
        "ui.set_welcome_pane(",
        "ui.set_recent_projects(",
        "ui.set_hierarchy_nodes(",
        "ui.set_project_overview(",
        "ui.set_status_primary(",
        "ui.set_delete_enabled(",
        "ui.set_inspector_name(",
        "ui.set_inspector_parent(",
        "ui.set_inspector_x(",
        "ui.set_inspector_y(",
        "ui.set_inspector_z(",
        "ui.set_mesh_import_path(",
    ] {
        assert!(
            !apply_presentation.contains(removed_root_pane_setter),
            "apply_presentation should not keep root pane setter `{removed_root_pane_setter}`"
        );
    }
    assert!(pointer_layout.contains("fn pane_surface_host(&self) -> PaneSurfaceHostContext<'_> {"));
    for removed_pointer_layout_setter in [
        "self.ui.set_welcome_recent_scroll_px(",
        "self.ui.set_hovered_welcome_recent_index(",
        "self.ui.set_hierarchy_scroll_px(",
        "self.ui.set_console_scroll_px(",
        "self.ui.set_inspector_scroll_px(",
        "self.ui.set_browser_asset_details_scroll_px(",
        "self.ui.set_activity_asset_tree_hovered_index(",
        "self.ui.set_browser_asset_tree_hovered_index(",
    ] {
        assert!(
            !pointer_layout.contains(removed_pointer_layout_setter),
            "pointer_layout should not keep root pane setter `{removed_pointer_layout_setter}`"
        );
    }
    assert!(host_lifecycle.contains(".global::<crate::ui::slint_host::PaneSurfaceHostContext>()"));
    assert!(!host_lifecycle.contains("self.ui.set_viewport_image("));
    assert!(pane_surface.contains("if root.pane.kind == \"Welcome\": WelcomePane {"));
    assert!(pane_surface.contains(
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {"
    ));
}

#[test]
fn shell_drag_targets_allow_empty_tool_regions() {
    let source = shell_source();
    let root = root_shell_source();
    let host_context = host_context_source();

    assert!(host_context
        .contains("callback workbench_drag_pointer_event(kind: int, x: float, y: float);"));
    assert!(host_context
        .contains("callback workbench_resize_pointer_event(kind: int, x: float, y: float);"));
    assert!(!root.contains("callback workbench_drag_pointer_event(kind: int, x: float, y: float);"));
    assert!(
        !root.contains("callback workbench_resize_pointer_event(kind: int, x: float, y: float);")
    );
    assert!(!source.contains("callback drop_tab(tab_id: string, target_group: string, pointer_x: float, pointer_y: float);"));
    assert!(!source.contains("callback update_drag_target(x: float, y: float);"));
    assert!(!source.contains("callback begin_drawer_resize(x: float, y: float);"));
    assert!(!source.contains("callback update_drawer_resize(x: float, y: float);"));
    assert!(!source.contains("callback finish_drawer_resize(x: float, y: float);"));

    assert!(host_context.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(!root.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(!source.contains("property <string> drag_target_group:"));
    let host_surface = host_surface_source();
    assert!(!host_surface.contains(
        "if WorkbenchHostContext.drag_state.drag_active: HostTabDragOverlay {"
    ));
    let host_components = host_components_source();
    assert!(host_components.contains("visible: WorkbenchHostContext.drag_state.drag_active;"));

    let drag_overlay = scoped_block_after(
        &source,
        "export component HostTabDragOverlay inherits Rectangle {",
    );
    assert!(drag_overlay.contains("if root.overlay_data.left_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains("if root.overlay_data.right_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains(
        "x: parent.width - root.overlay_data.right_drop_width_px * 1px + 8px;"
    ));
    assert!(drag_overlay.contains("if root.overlay_data.bottom_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains("WorkbenchHostContext.workbench_drag_pointer_event("));
    assert!(drag_overlay.contains("WorkbenchHostContext.drag_state.drag_pointer_x,"));
    assert!(drag_overlay.contains("WorkbenchHostContext.drag_state.drag_pointer_y,"));
    assert!(!drag_overlay
        .contains("WorkbenchHostContext.update_drag_target(WorkbenchHostContext.drag_state.drag_pointer_x, WorkbenchHostContext.drag_state.drag_pointer_y);"));
    assert!(!drag_overlay.contains("root.drop_tab("));
    assert!(drag_overlay.contains("active_drag_target_group: \"\","));

    let resize_overlay = block_after(
        &source,
        "if WorkbenchHostContext.resize_state.resize_active: TouchArea {",
    );
    assert!(resize_overlay.contains(
        "WorkbenchHostContext.workbench_resize_pointer_event(1, self.mouse-x / 1px, self.mouse-y / 1px);"
    ));
    assert!(resize_overlay.contains(
        "WorkbenchHostContext.workbench_resize_pointer_event(2, self.mouse-x / 1px, self.mouse-y / 1px);"
    ));
    assert!(resize_overlay.contains("mouse-cursor: WorkbenchHostContext.resize_state.resize_group == \"bottom\" ? ns-resize : ew-resize;"));
    assert!(!resize_overlay.contains("root.update_drawer_resize("));
    assert!(!resize_overlay.contains("root.finish_drawer_resize("));
}

#[test]
fn drag_overlay_uses_pointer_following_ghost_preview_instead_of_centered_banner() {
    let source = shell_source();

    assert!(source.contains("drag_preview := Rectangle {"));
    assert!(source.contains("x: clamp(WorkbenchHostContext.drag_state.drag_pointer_x * 1px"));
    assert!(source.contains("y: clamp("));
    assert!(source.contains("WorkbenchHostContext.drag_state.drag_pointer_y * 1px - self.height - 14px"));
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

    assert!(source.contains("out property <FrameRect> file_menu_button_frame: {"));
    assert!(source.contains("out property <FrameRect> edit_menu_button_frame: {"));
    assert!(source.contains("out property <FrameRect> selection_menu_button_frame: {"));
    assert!(source.contains("out property <FrameRect> view_menu_button_frame: {"));
    assert!(source.contains("out property <FrameRect> window_menu_button_frame: {"));
    assert!(source.contains("out property <FrameRect> help_menu_button_frame: {"));

    assert!(
        source.contains("x: top_bar.x / 1px + menu_button_row.x / 1px + file_menu_button.x / 1px,")
    );
    assert!(
        source.contains("x: top_bar.x / 1px + menu_button_row.x / 1px + edit_menu_button.x / 1px,")
    );
    assert!(source
        .contains("x: top_bar.x / 1px + menu_button_row.x / 1px + selection_menu_button.x / 1px,"));
    assert!(
        source.contains("x: top_bar.x / 1px + menu_button_row.x / 1px + view_menu_button.x / 1px,")
    );
    assert!(source
        .contains("x: top_bar.x / 1px + menu_button_row.x / 1px + window_menu_button.x / 1px,"));
    assert!(
        source.contains("x: top_bar.x / 1px + menu_button_row.x / 1px + help_menu_button.x / 1px,")
    );

    assert!(source.contains("if WorkbenchHostContext.menu_state.open_menu_index == 0: Rectangle {"));
    assert!(source.contains("x: root.file_menu_button_frame.x * 1px;"));
    assert!(source.contains("if WorkbenchHostContext.menu_state.open_menu_index == 1: Rectangle {"));
    assert!(source.contains("x: root.edit_menu_button_frame.x * 1px;"));
    assert!(source.contains("if WorkbenchHostContext.menu_state.open_menu_index == 2: Rectangle {"));
    assert!(source.contains("x: root.selection_menu_button_frame.x * 1px;"));
    assert!(source.contains("if WorkbenchHostContext.menu_state.open_menu_index == 3: Rectangle {"));
    assert!(source.contains("x: root.view_menu_button_frame.x * 1px;"));
    assert!(source.contains("if WorkbenchHostContext.menu_state.open_menu_index == 4: Rectangle {"));
    assert!(source.contains("x: root.window_menu_button_frame.x * 1px;"));
    assert!(source.contains("if WorkbenchHostContext.menu_state.open_menu_index == 5: Rectangle {"));
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

    assert!(source.contains("top_bar_height_px: root.surface_metrics.top_bar_height_px,"));
    assert!(!source.contains("property <length> popup_y:"));
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
    assert!(preview.contains("text: root.drag_target_label != \"\" ? root.drag_target_label : \"Move Tab\";"));
}

#[test]
fn drag_overlay_declares_document_edge_target_keys_and_highlights() {
    let source = shell_source();

    assert!(source.contains(
        "WorkbenchHostContext.drag_state.active_drag_target_group == \"document-left\" ? \"Split Left\" :"
    ));
    assert!(source.contains(
        "WorkbenchHostContext.drag_state.active_drag_target_group == \"document-right\" ? \"Split Right\" :"
    ));
    assert!(source.contains(
        "WorkbenchHostContext.drag_state.active_drag_target_group == \"document-top\" ? \"Split Top\" :"
    ));
    assert!(source.contains(
        "WorkbenchHostContext.drag_state.active_drag_target_group == \"document-bottom\" ? \"Split Bottom\" :"
    ));

    let drag_overlay = scoped_block_after(
        &source,
        "export component HostTabDragOverlay inherits Rectangle {",
    );
    assert!(drag_overlay.contains("WorkbenchHostContext.drag_state.active_drag_target_group == \"document-left\""));
    assert!(drag_overlay.contains("WorkbenchHostContext.drag_state.active_drag_target_group == \"document-right\""));
    assert!(drag_overlay.contains("WorkbenchHostContext.drag_state.active_drag_target_group == \"document-top\""));
    assert!(drag_overlay.contains("WorkbenchHostContext.drag_state.active_drag_target_group == \"document-bottom\""));
}

#[test]
fn host_surface_groups_host_surface_interaction_and_layout_payloads() {
    let host_surface = host_surface_source();
    let host_components = host_components_source();
    let host_interaction = host_interaction_source();
    let host_context = host_context_source();
    let pointer_layout = slint_host_source("src/ui/slint_host/app/pointer_layout.rs");
    let workspace_docking = slint_host_source("src/ui/slint_host/app/workspace_docking.rs");

    for grouped_struct in [
        "export struct HostMenuChromeData {",
        "export struct HostPageChromeData {",
        "export struct HostStatusBarData {",
        "export struct HostResizeLayerData {",
        "export struct HostTabDragOverlayData {",
        "export struct HostSideDockSurfaceData {",
        "export struct HostDocumentDockSurfaceData {",
        "export struct HostBottomDockSurfaceData {",
        "export struct HostFloatingWindowLayerData {",
    ] {
        assert!(
            host_components.contains(grouped_struct),
            "host components should declare grouped interaction/layout payload `{grouped_struct}`"
        );
    }
    for interaction_struct in [
        "export struct HostMenuStateData {",
        "export struct HostDragStateData {",
        "export struct HostResizeStateData {",
    ] {
        assert!(
            host_interaction.contains(interaction_struct),
            "host interaction contract should declare grouped interaction state `{interaction_struct}`"
        );
    }

    assert!(host_context.contains(
        "in-out property <HostMenuStateData> menu_state: {"
    ));
    assert!(host_context.contains(
        "in-out property <HostDragStateData> drag_state: {"
    ));
    assert!(host_context.contains(
        "in-out property <HostResizeStateData> resize_state: {"
    ));
    assert!(host_context.contains(
        "import { HostDragStateData, HostMenuStateData, HostResizeStateData } from \"host_interaction.slint\";"
    ));
    assert!(!host_context.contains(
        "import { HostDragStateData, HostMenuStateData, HostResizeStateData } from \"host_components.slint\";"
    ));
    for removed_context_scalar in [
        "in-out property <int> open_menu_index: -1;",
        "in-out property <int> hovered_menu_index: -1;",
        "in-out property <int> hovered_menu_item_index: -1;",
        "in-out property <float> window_menu_scroll_px: 0.0;",
        "in-out property <float> window_menu_popup_height_px: 72.0;",
        "in-out property <string> active_drag_target_group: \"\";",
        "in-out property <bool> drag_active: false;",
        "in-out property <string> drag_tab_id: \"\";",
        "in-out property <string> drag_tab_title: \"\";",
        "in-out property <string> drag_tab_icon_key: \"\";",
        "in-out property <string> drag_source_group: \"\";",
        "in-out property <float> drag_pointer_x: 0.0;",
        "in-out property <float> drag_pointer_y: 0.0;",
    ] {
        assert!(
            !host_context.contains(removed_context_scalar),
            "host context should drop scalar interaction property `{removed_context_scalar}` after DTO grouping"
        );
    }

    assert!(host_surface.contains("menu_data: root.menu_chrome_data;"));
    assert!(host_surface.contains("page_data: root.page_chrome_data;"));
    assert!(host_surface.contains("status_data: root.status_bar_data;"));
    assert!(host_surface.contains("resize_data: root.resize_layer_data;"));
    assert!(host_surface.contains("overlay_data: root.drag_overlay_data;"));
    assert!(host_surface.contains("property <HostSideDockSurfaceData> left_dock_data: {"));
    assert!(host_surface.contains("property <HostDocumentDockSurfaceData> document_dock_data: {"));
    assert!(host_surface.contains("property <HostSideDockSurfaceData> right_dock_data: {"));
    assert!(host_surface.contains("property <HostBottomDockSurfaceData> bottom_dock_data: {"));
    assert!(host_surface.contains("property <HostFloatingWindowLayerData> floating_layer_data: {"));
    assert!(!host_surface.contains("import { WorkbenchHostContext } from \"host_context.slint\";"));
    for removed_host_surface_forwarding in [
        "menu_state: WorkbenchHostContext.menu_state;",
        "resize_state <=> WorkbenchHostContext.resize_state;",
        "drag_state <=> WorkbenchHostContext.drag_state;",
        "WorkbenchHostContext.menu_pointer_clicked(",
        "WorkbenchHostContext.host_page_pointer_clicked(",
        "WorkbenchHostContext.activity_rail_pointer_clicked(",
        "WorkbenchHostContext.drawer_header_pointer_clicked(",
        "WorkbenchHostContext.document_tab_pointer_clicked(",
        "WorkbenchHostContext.document_tab_close_pointer_clicked(",
        "WorkbenchHostContext.floating_window_header_pointer_clicked(",
        "WorkbenchHostContext.workbench_drag_pointer_event(",
        "WorkbenchHostContext.workbench_resize_pointer_event(",
        "if WorkbenchHostContext.drag_state.drag_active: HostTabDragOverlay {",
    ] {
        assert!(
            !host_surface.contains(removed_host_surface_forwarding),
            "host_surface should stop acting as interaction forwarder `{removed_host_surface_forwarding}`"
        );
    }

    let host_menu = scoped_block_after(&host_surface, "host_menu := HostMenuChrome {");
    for removed_host_menu_scalar in [
        "outer_margin: root.outer_margin;",
        "top_bar_height: root.top_bar_height;",
        "save_project_enabled: root.host_shell.save_project_enabled;",
        "undo_enabled: root.host_shell.undo_enabled;",
        "redo_enabled: root.host_shell.redo_enabled;",
        "delete_enabled: PaneSurfaceHostContext.delete_enabled;",
        "preset_names: root.host_shell.preset_names;",
        "active_preset_name: root.host_shell.active_preset_name;",
        "resolved_preset_name: root.resolved_preset_name;",
        "open_menu_index: WorkbenchHostContext.open_menu_index;",
        "hovered_menu_index: WorkbenchHostContext.hovered_menu_index;",
        "hovered_menu_item_index: WorkbenchHostContext.hovered_menu_item_index;",
        "window_menu_scroll_px: WorkbenchHostContext.window_menu_scroll_px;",
        "window_menu_popup_height_px: WorkbenchHostContext.window_menu_popup_height_px;",
    ] {
        assert!(
            !host_menu.contains(removed_host_menu_scalar),
            "host menu callsite should consume grouped DTO instead of `{removed_host_menu_scalar}`"
        );
    }

    let host_page = scoped_block_after(&host_surface, "HostPageChrome {");
    for removed_host_page_scalar in [
        "top_bar_height: root.top_bar_height;",
        "host_bar_height: root.host_bar_height;",
        "tabs: root.host_surface_data.host_tabs;",
        "project_path: root.host_shell.project_path;",
    ] {
        assert!(
            !host_page.contains(removed_host_page_scalar),
            "host page chrome callsite should consume grouped DTO instead of `{removed_host_page_scalar}`"
        );
    }

    let host_status_bar = scoped_block_after(&host_surface, "HostStatusBar {");
    for removed_status_scalar in [
        "status_bar_frame: root.host_layout.status_bar_frame;",
        "status_primary: PaneSurfaceHostContext.status_text;",
        "status_secondary: root.host_shell.status_secondary;",
        "viewport_label: root.host_shell.viewport_label;",
    ] {
        assert!(
            !host_status_bar.contains(removed_status_scalar),
            "status bar callsite should consume grouped DTO instead of `{removed_status_scalar}`"
        );
    }

    let host_resize_layer = scoped_block_after(&host_surface, "HostResizeLayer {");
    for removed_resize_scalar in [
        "left_splitter_frame: root.host_layout.left_splitter_frame;",
        "right_splitter_frame: root.host_layout.right_splitter_frame;",
        "bottom_splitter_frame: root.host_layout.bottom_splitter_frame;",
        "resize_active <=> root.resize_active;",
        "resize_group <=> root.resize_group;",
    ] {
        assert!(
            !host_resize_layer.contains(removed_resize_scalar),
            "resize layer callsite should consume grouped DTO instead of `{removed_resize_scalar}`"
        );
    }

    let drag_overlay = scoped_block_after(
        &host_surface,
        "HostTabDragOverlay {",
    );
    for removed_drag_overlay_scalar in [
        "drag_active <=> WorkbenchHostContext.drag_active;",
        "drag_tab_id <=> WorkbenchHostContext.drag_tab_id;",
        "drag_tab_title <=> WorkbenchHostContext.drag_tab_title;",
        "drag_tab_icon_key <=> WorkbenchHostContext.drag_tab_icon_key;",
        "drag_source_group <=> WorkbenchHostContext.drag_source_group;",
        "drag_pointer_x <=> WorkbenchHostContext.drag_pointer_x;",
        "drag_pointer_y <=> WorkbenchHostContext.drag_pointer_y;",
        "left_drop_enabled: root.left_drop_enabled;",
        "right_drop_enabled: root.right_drop_enabled;",
        "bottom_drop_enabled: root.bottom_drop_enabled;",
        "left_drop_width: root.left_drop_width;",
        "right_drop_width: root.right_drop_width;",
        "bottom_drop_height: root.bottom_drop_height;",
        "main_content_y: root.main_content_y;",
        "main_content_height: root.main_content_height;",
        "document_zone_x: root.document_zone_x;",
        "document_zone_width: root.document_zone_width;",
        "bottom_drop_top_px: root.bottom_drop_top_px;",
        "drag_overlay_bottom_px: root.drag_overlay_bottom_px;",
        "drag_target_label: root.drag_target_label;",
    ] {
        assert!(
            !drag_overlay.contains(removed_drag_overlay_scalar),
            "drag overlay callsite should consume grouped DTO instead of `{removed_drag_overlay_scalar}`"
        );
    }

    let left_dock_surface = scoped_block_after(
        &host_surface,
        "if root.surface_orchestration_data.left_stack_width_px > 0.0: HostSideDockSurface {",
    );
    assert!(left_dock_surface.contains("surface_data: root.left_dock_data;"));
    for removed_left_surface_forwarder in [
        "drag_state <=> WorkbenchHostContext.drag_state;",
        "activity_rail_pointer_clicked(side, x, y) => {",
        "drawer_header_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "workbench_drag_pointer_event(kind, x, y) => {",
    ] {
        assert!(
            !left_dock_surface.contains(removed_left_surface_forwarder),
            "left side dock callsite should stop forwarding interaction `{removed_left_surface_forwarder}`"
        );
    }
    for removed_left_surface_scalar in [
        "region_frame: root.host_layout.left_region_frame;",
        "surface_key: \"left\";",
        "rail_before_panel: true;",
        "tabs: root.host_surface_data.left_tabs;",
        "pane: root.host_surface_data.left_pane;",
        "rail_width: root.left_rail_width;",
        "panel_width: root.left_panel_width;",
        "panel_header_height: root.panel_header_height;",
        "tab_origin_x: root.left_tab_origin_x;",
        "tab_origin_y: root.left_tab_origin_y;",
    ] {
        assert!(
            !left_dock_surface.contains(removed_left_surface_scalar),
            "left side dock surface should consume grouped DTO instead of `{removed_left_surface_scalar}`"
        );
    }

    let document_dock_surface = scoped_block_after(&host_surface, "HostDocumentDockSurface {");
    assert!(document_dock_surface.contains("surface_data: root.document_dock_data;"));
    for removed_document_surface_forwarder in [
        "drag_state <=> WorkbenchHostContext.drag_state;",
        "document_tab_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "document_tab_close_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "workbench_drag_pointer_event(kind, x, y) => {",
    ] {
        assert!(
            !document_dock_surface.contains(removed_document_surface_forwarder),
            "document dock callsite should stop forwarding interaction `{removed_document_surface_forwarder}`"
        );
    }
    for removed_document_surface_scalar in [
        "region_frame: root.host_layout.document_region_frame;",
        "tabs: root.host_surface_data.document_tabs;",
        "pane: root.host_surface_data.document_pane;",
        "header_height: root.document_header_height;",
        "tab_origin_x: root.document_tab_origin_x;",
        "tab_origin_y: root.document_tab_origin_y;",
    ] {
        assert!(
            !document_dock_surface.contains(removed_document_surface_scalar),
            "document dock surface should consume grouped DTO instead of `{removed_document_surface_scalar}`"
        );
    }

    let right_dock_surface = scoped_block_after(
        &host_surface,
        "if root.surface_orchestration_data.right_stack_width_px > 0.0: HostSideDockSurface {",
    );
    assert!(right_dock_surface.contains("surface_data: root.right_dock_data;"));
    for removed_right_surface_forwarder in [
        "drag_state <=> WorkbenchHostContext.drag_state;",
        "activity_rail_pointer_clicked(side, x, y) => {",
        "drawer_header_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "workbench_drag_pointer_event(kind, x, y) => {",
    ] {
        assert!(
            !right_dock_surface.contains(removed_right_surface_forwarder),
            "right side dock callsite should stop forwarding interaction `{removed_right_surface_forwarder}`"
        );
    }
    for removed_right_surface_scalar in [
        "region_frame: root.host_layout.right_region_frame;",
        "surface_key: \"right\";",
        "rail_before_panel: false;",
        "tabs: root.host_surface_data.right_tabs;",
        "pane: root.host_surface_data.right_pane;",
        "rail_width: root.right_rail_width;",
        "panel_width: root.right_panel_width;",
        "panel_header_height: root.panel_header_height;",
        "tab_origin_x: root.right_tab_origin_x;",
        "tab_origin_y: root.right_tab_origin_y;",
    ] {
        assert!(
            !right_dock_surface.contains(removed_right_surface_scalar),
            "right side dock surface should consume grouped DTO instead of `{removed_right_surface_scalar}`"
        );
    }

    let bottom_dock_surface = scoped_block_after(
        &host_surface,
        "if root.surface_orchestration_data.bottom_panel_height_px > 0.0: HostBottomDockSurface {",
    );
    assert!(bottom_dock_surface.contains("surface_data: root.bottom_dock_data;"));
    for removed_bottom_surface_forwarder in [
        "drag_state <=> WorkbenchHostContext.drag_state;",
        "drawer_header_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "workbench_drag_pointer_event(kind, x, y) => {",
    ] {
        assert!(
            !bottom_dock_surface.contains(removed_bottom_surface_forwarder),
            "bottom dock callsite should stop forwarding interaction `{removed_bottom_surface_forwarder}`"
        );
    }
    for removed_bottom_surface_scalar in [
        "region_frame: root.host_layout.bottom_region_frame;",
        "tabs: root.host_surface_data.bottom_tabs;",
        "pane: root.host_surface_data.bottom_pane;",
        "expanded: root.host_shell.bottom_expanded;",
        "header_height: root.panel_header_height;",
        "tab_origin_x: root.bottom_tab_origin_x;",
        "tab_origin_y: root.bottom_tab_origin_y;",
    ] {
        assert!(
            !bottom_dock_surface.contains(removed_bottom_surface_scalar),
            "bottom dock surface should consume grouped DTO instead of `{removed_bottom_surface_scalar}`"
        );
    }

    let floating_window_layer = scoped_block_after(&host_surface, "HostFloatingWindowLayer {");
    assert!(floating_window_layer.contains("layer_data: root.floating_layer_data;"));
    for removed_floating_layer_forwarder in [
        "drag_state <=> WorkbenchHostContext.drag_state;",
        "document_tab_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "document_tab_close_pointer_clicked(surface_key, tab_index, tab_x, tab_width, point_x, point_y) => {",
        "floating_window_header_pointer_clicked(x, y) => {",
        "workbench_drag_pointer_event(kind, x, y) => {",
    ] {
        assert!(
            !floating_window_layer.contains(removed_floating_layer_forwarder),
            "floating layer callsite should stop forwarding interaction `{removed_floating_layer_forwarder}`"
        );
    }
    for removed_floating_layer_scalar in [
        "floating_windows: root.host_surface_data.floating_windows;",
        "header_height: root.document_header_height;",
    ] {
        assert!(
            !floating_window_layer.contains(removed_floating_layer_scalar),
            "floating layer should consume grouped DTO instead of `{removed_floating_layer_scalar}`"
        );
    }

    assert!(host_components.contains(
        "import { WorkbenchHostContext } from \"host_context.slint\";"
    ));
    assert!(host_components.contains("in property <HostMenuChromeData> menu_data: {"));
    assert!(host_components.contains("in property <HostPageChromeData> page_data: {"));
    assert!(host_components.contains("in property <HostStatusBarData> status_data: {"));
    assert!(host_components.contains("in property <HostResizeLayerData> resize_data: {"));
    assert!(host_components.contains("in property <HostTabDragOverlayData> overlay_data: {"));
    let host_menu_chrome = scoped_block_after(
        &host_components,
        "export component HostMenuChrome inherits Rectangle {",
    );
    assert!(host_menu_chrome.contains("WorkbenchHostContext.menu_state.open_menu_index"));
    assert!(host_menu_chrome.contains("WorkbenchHostContext.menu_pointer_clicked("));
    let host_tab_drag_overlay = scoped_block_after(
        &host_components,
        "export component HostTabDragOverlay inherits Rectangle {",
    );
    assert!(host_tab_drag_overlay.contains("WorkbenchHostContext.drag_state.active_drag_target_group"));
    assert!(host_tab_drag_overlay.contains("WorkbenchHostContext.workbench_drag_pointer_event("));
    let host_resize_layer = scoped_block_after(
        &host_components,
        "export component HostResizeLayer inherits Rectangle {",
    );
    assert!(host_resize_layer.contains("WorkbenchHostContext.resize_state.resize_group"));
    assert!(host_resize_layer.contains("WorkbenchHostContext.workbench_resize_pointer_event("));
    let host_page_chrome = scoped_block_after(
        &host_components,
        "export component HostPageChrome inherits Rectangle {",
    );
    assert!(!host_page_chrome.contains("in property <[TabData]> tabs;"));
    assert!(!host_page_chrome.contains("in property <string> project_path;"));
    assert!(!host_page_chrome.contains("callback host_page_pointer_clicked("));
    assert!(host_page_chrome.contains("WorkbenchHostContext.host_page_pointer_clicked("));
    let host_side_dock_surface = scoped_block_after(
        &host_components,
        "export component HostSideDockSurface inherits Rectangle {",
    );
    assert!(host_side_dock_surface.contains("in property <HostSideDockSurfaceData> surface_data;"));
    assert!(!host_side_dock_surface.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(!host_side_dock_surface.contains("callback activity_rail_pointer_clicked("));
    assert!(!host_side_dock_surface.contains("callback drawer_header_pointer_clicked("));
    assert!(!host_side_dock_surface.contains("callback workbench_drag_pointer_event("));
    assert!(host_side_dock_surface.contains("WorkbenchHostContext.activity_rail_pointer_clicked("));
    assert!(host_side_dock_surface.contains("WorkbenchHostContext.drawer_header_pointer_clicked("));
    assert!(host_side_dock_surface.contains("WorkbenchHostContext.workbench_drag_pointer_event("));
    assert!(host_side_dock_surface.contains("WorkbenchHostContext.drag_state.drag_tab_id == tab.id"));
    for removed_side_dock_scalar in [
        "in property <FrameRect> region_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <string> surface_key;",
        "in property <bool> rail_before_panel: true;",
        "in property <[TabData]> tabs;",
        "in property <PaneData> pane;",
        "in property <length> rail_width: 0px;",
        "in property <length> panel_width: 0px;",
        "in property <length> panel_header_height: 25px;",
        "in property <length> tab_origin_x: 0px;",
        "in property <length> tab_origin_y: 0px;",
    ] {
        assert!(
            !host_side_dock_surface.contains(removed_side_dock_scalar),
            "HostSideDockSurface should drop scalar ABI `{removed_side_dock_scalar}` after DTO grouping"
        );
    }

    let host_document_dock_surface = scoped_block_after(
        &host_components,
        "export component HostDocumentDockSurface inherits Rectangle {",
    );
    assert!(host_document_dock_surface.contains(
        "in property <HostDocumentDockSurfaceData> surface_data;"
    ));
    assert!(!host_document_dock_surface.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(!host_document_dock_surface.contains("callback document_tab_pointer_clicked("));
    assert!(!host_document_dock_surface.contains("callback document_tab_close_pointer_clicked("));
    assert!(!host_document_dock_surface.contains("callback workbench_drag_pointer_event("));
    assert!(host_document_dock_surface.contains("WorkbenchHostContext.document_tab_pointer_clicked("));
    assert!(host_document_dock_surface.contains("WorkbenchHostContext.document_tab_close_pointer_clicked("));
    assert!(host_document_dock_surface.contains("WorkbenchHostContext.workbench_drag_pointer_event("));
    assert!(host_document_dock_surface.contains("WorkbenchHostContext.drag_state.drag_tab_id == tab.id"));
    for removed_document_dock_scalar in [
        "in property <FrameRect> region_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <[TabData]> tabs;",
        "in property <PaneData> pane;",
        "in property <length> header_height: 31px;",
        "in property <length> tab_origin_x: 0px;",
        "in property <length> tab_origin_y: 0px;",
    ] {
        assert!(
            !host_document_dock_surface.contains(removed_document_dock_scalar),
            "HostDocumentDockSurface should drop scalar ABI `{removed_document_dock_scalar}` after DTO grouping"
        );
    }

    let host_bottom_dock_surface = scoped_block_after(
        &host_components,
        "export component HostBottomDockSurface inherits Rectangle {",
    );
    assert!(host_bottom_dock_surface.contains("in property <HostBottomDockSurfaceData> surface_data;"));
    assert!(!host_bottom_dock_surface.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(!host_bottom_dock_surface.contains("callback drawer_header_pointer_clicked("));
    assert!(!host_bottom_dock_surface.contains("callback workbench_drag_pointer_event("));
    assert!(host_bottom_dock_surface.contains("WorkbenchHostContext.drawer_header_pointer_clicked("));
    assert!(host_bottom_dock_surface.contains("WorkbenchHostContext.workbench_drag_pointer_event("));
    assert!(host_bottom_dock_surface.contains("WorkbenchHostContext.drag_state.drag_tab_id == tab.id"));
    for removed_bottom_dock_scalar in [
        "in property <FrameRect> region_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <[TabData]> tabs;",
        "in property <PaneData> pane;",
        "in property <bool> expanded: false;",
        "in property <length> header_height: 25px;",
        "in property <length> tab_origin_x: 0px;",
        "in property <length> tab_origin_y: 0px;",
    ] {
        assert!(
            !host_bottom_dock_surface.contains(removed_bottom_dock_scalar),
            "HostBottomDockSurface should drop scalar ABI `{removed_bottom_dock_scalar}` after DTO grouping"
        );
    }

    let host_floating_window_layer = scoped_block_after(
        &host_components,
        "export component HostFloatingWindowLayer inherits Rectangle {",
    );
    assert!(host_floating_window_layer.contains(
        "in property <HostFloatingWindowLayerData> layer_data;"
    ));
    assert!(!host_floating_window_layer.contains("in-out property <HostDragStateData> drag_state: {"));
    assert!(!host_floating_window_layer.contains("callback document_tab_pointer_clicked("));
    assert!(!host_floating_window_layer.contains("callback document_tab_close_pointer_clicked("));
    assert!(!host_floating_window_layer.contains("callback floating_window_header_pointer_clicked("));
    assert!(!host_floating_window_layer.contains("callback workbench_drag_pointer_event("));
    assert!(host_floating_window_layer.contains("WorkbenchHostContext.document_tab_pointer_clicked("));
    assert!(host_floating_window_layer.contains("WorkbenchHostContext.document_tab_close_pointer_clicked("));
    assert!(host_floating_window_layer.contains("WorkbenchHostContext.floating_window_header_pointer_clicked("));
    assert!(host_floating_window_layer.contains("WorkbenchHostContext.workbench_drag_pointer_event("));
    assert!(host_floating_window_layer.contains("WorkbenchHostContext.drag_state.active_drag_target_group == window.target_group"));
    for removed_floating_window_scalar in [
        "in property <[FloatingWindowData]> floating_windows;",
        "in property <length> header_height: 31px;",
    ] {
        assert!(
            !host_floating_window_layer.contains(removed_floating_window_scalar),
            "HostFloatingWindowLayer should drop scalar ABI `{removed_floating_window_scalar}` after DTO grouping"
        );
    }
    for removed_component_scalar in [
        "in property <length> outer_margin: 0px;",
        "in property <length> top_bar_height: 25px;",
        "in property <bool> save_project_enabled: false;",
        "in property <bool> undo_enabled: false;",
        "in property <bool> redo_enabled: false;",
        "in property <bool> delete_enabled: false;",
        "in property <[string]> preset_names;",
        "in property <string> active_preset_name;",
        "in property <string> resolved_preset_name: root.active_preset_name != \"\" ? root.active_preset_name : \"rider\";",
        "in property <length> host_bar_height: 24px;",
        "in property <int> open_menu_index: -1;",
        "in property <int> hovered_menu_index: -1;",
        "in property <int> hovered_menu_item_index: -1;",
        "in property <float> window_menu_scroll_px: 0.0;",
        "in property <float> window_menu_popup_height_px: 72.0;",
        "in property <FrameRect> status_bar_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <string> status_primary;",
        "in property <string> status_secondary;",
        "in property <string> viewport_label;",
        "in property <FrameRect> left_splitter_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <FrameRect> right_splitter_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <FrameRect> bottom_splitter_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <string> active_drag_target_group: \"\";",
        "in-out property <bool> drag_active: false;",
        "in-out property <string> drag_tab_id: \"\";",
        "in-out property <string> drag_tab_title: \"\";",
        "in-out property <string> drag_tab_icon_key: \"\";",
        "in-out property <string> drag_source_group: \"\";",
        "in-out property <float> drag_pointer_x: 0.0;",
        "in-out property <float> drag_pointer_y: 0.0;",
        "in-out property <bool> resize_active: false;",
        "in-out property <string> resize_group: \"\";",
        "in property <bool> left_drop_enabled: false;",
        "in property <bool> right_drop_enabled: false;",
        "in property <bool> bottom_drop_enabled: false;",
        "in property <length> left_drop_width: 0px;",
        "in property <length> right_drop_width: 0px;",
        "in property <length> bottom_drop_height: 0px;",
        "in property <length> main_content_y: 0px;",
        "in property <length> main_content_height: 0px;",
        "in property <length> document_zone_x: 0px;",
        "in property <length> document_zone_width: 0px;",
        "in property <float> bottom_drop_top_px: 0.0;",
        "in property <float> drag_overlay_bottom_px: 0.0;",
        "in property <string> drag_target_label;",
        "in property <HostMenuStateData> menu_state: {",
        "in-out property <HostDragStateData> drag_state: {",
        "in-out property <HostResizeStateData> resize_state: {",
        "callback menu_pointer_clicked(",
        "callback menu_pointer_moved(",
        "callback menu_pointer_scrolled(",
        "callback host_page_pointer_clicked(",
        "callback activity_rail_pointer_clicked(",
        "callback drawer_header_pointer_clicked(",
        "callback document_tab_pointer_clicked(",
        "callback document_tab_close_pointer_clicked(",
        "callback floating_window_header_pointer_clicked(",
        "callback workbench_drag_pointer_event(",
        "callback workbench_resize_pointer_event(",
    ] {
        assert!(
            !host_components.contains(removed_component_scalar),
            "host components should drop scalar interaction/layout ABI `{removed_component_scalar}` after DTO grouping"
        );
    }

    assert!(pointer_layout.contains("host_shell.set_menu_state(HostMenuStateData {"));
    for removed_pointer_layout_setter in [
        "host_shell.set_open_menu_index(",
        "host_shell.set_hovered_menu_index(",
        "host_shell.set_hovered_menu_item_index(",
        "host_shell.set_window_menu_scroll_px(",
        "host_shell.set_window_menu_popup_height_px(",
    ] {
        assert!(
            !pointer_layout.contains(removed_pointer_layout_setter),
            "pointer_layout should stop using scalar menu setters `{removed_pointer_layout_setter}`"
        );
    }

    assert!(workspace_docking.contains("let mut drag_state = host_shell.get_drag_state();"));
    assert!(workspace_docking.contains("host_shell.set_drag_state(drag_state);"));
    for removed_workspace_docking_scalar in [
        ".set_active_drag_target_group(",
        "host_shell.get_drag_tab_id().to_string()",
        "host_shell.get_active_drag_target_group().to_string()",
    ] {
        assert!(
            !workspace_docking.contains(removed_workspace_docking_scalar),
            "workspace docking should stop using scalar drag state access `{removed_workspace_docking_scalar}`"
        );
    }
}

#[test]
fn host_surface_groups_orchestration_metrics_and_native_floating_payloads() {
    let host_surface = host_surface_source();
    let host_components = host_components_source();

    for grouped_struct in [
        "export struct HostWorkbenchSurfaceMetricsData {",
        "export struct HostWorkbenchSurfaceOrchestrationData {",
        "export struct HostNativeFloatingWindowSurfaceData {",
    ] {
        assert!(
            host_components.contains(grouped_struct),
            "host components should declare orchestration/native grouped payload `{grouped_struct}`"
        );
    }

    assert!(host_surface.contains(
        "property <HostWorkbenchSurfaceMetricsData> surface_metrics: {"
    ));
    assert!(host_surface.contains(
        "property <HostWorkbenchSurfaceOrchestrationData> surface_orchestration_data: {"
    ));
    assert!(host_surface.contains(
        "property <HostNativeFloatingWindowSurfaceData> native_floating_surface_data: {"
    ));
    for removed_host_surface_length_property in [
        "property <length> outer_margin: 0px;",
        "property <length> rail_width: 34px;",
        "property <length> top_bar_height: 25px;",
        "property <length> host_bar_height: 24px;",
        "property <length> panel_header_height: 25px;",
        "property <length> document_header_height: 31px;",
        "property <length> left_rail_width:",
        "property <length> right_rail_width:",
        "property <length> left_stack_width:",
        "property <length> right_stack_width:",
        "property <length> left_panel_width:",
        "property <length> right_panel_width:",
        "property <length> bottom_panel_height:",
        "property <length> main_content_y:",
        "property <length> document_zone_x:",
        "property <length> right_stack_x:",
        "property <length> bottom_panel_y:",
        "property <length> left_tab_origin_x:",
        "property <length> left_tab_origin_y:",
        "property <length> document_tab_origin_x:",
        "property <length> document_tab_origin_y:",
        "property <length> right_tab_origin_x:",
        "property <length> right_tab_origin_y:",
        "property <length> bottom_tab_origin_x:",
        "property <length> bottom_tab_origin_y:",
    ] {
        assert!(
            !host_surface.contains(removed_host_surface_length_property),
            "host_surface should drop root orchestration scalar `{removed_host_surface_length_property}`"
        );
    }
    for removed_host_surface_scalar_binding in [
        "outer_margin_px: root.outer_margin / 1px,",
        "top_bar_height_px: root.top_bar_height / 1px,",
        "host_bar_height_px: root.host_bar_height / 1px,",
        "rail_width_px: root.left_rail_width / 1px,",
        "panel_width_px: root.left_panel_width / 1px,",
        "panel_header_height_px: root.panel_header_height / 1px,",
        "header_height_px: root.document_header_height / 1px,",
        "rail_width_px: root.right_rail_width / 1px,",
        "panel_width_px: root.right_panel_width / 1px,",
        "header_height_px: root.panel_header_height / 1px,",
        "tab_origin_x_px: root.bottom_tab_origin_x / 1px,",
        "floating_windows: root.host_surface_data.floating_windows;",
        "native_floating_window_id: root.host_shell.native_floating_window_id;",
        "native_window_bounds: root.host_shell.native_window_bounds;",
        "header_height: root.header_height;",
    ] {
        assert!(
            !host_surface.contains(removed_host_surface_scalar_binding),
            "host_surface should stop binding native/orchestration scalar `{removed_host_surface_scalar_binding}`"
        );
    }
    for expected_grouped_usage in [
        "outer_margin_px: root.surface_metrics.outer_margin_px,",
        "top_bar_height_px: root.surface_metrics.top_bar_height_px,",
        "host_bar_height_px: root.surface_metrics.host_bar_height_px,",
        "rail_width_px: root.surface_orchestration_data.left_rail_width_px,",
        "panel_width_px: root.surface_orchestration_data.left_panel_width_px,",
        "header_height_px: root.surface_metrics.document_header_height_px,",
        "x: root.surface_metrics.outer_margin_px * 1px;",
        "surface_data: root.native_floating_surface_data;",
    ] {
        assert!(
            host_surface.contains(expected_grouped_usage),
            "host_surface should use grouped orchestration/native data via `{expected_grouped_usage}`"
        );
    }

    let native_host_surface = scoped_block_after(
        &host_surface,
        "HostNativeFloatingWindowSurface {",
    );
    assert!(native_host_surface.contains("surface_data: root.native_floating_surface_data;"));
    for removed_native_host_scalar in [
        "floating_windows: root.host_surface_data.floating_windows;",
        "native_floating_window_id: root.host_shell.native_floating_window_id;",
        "native_window_bounds: root.host_shell.native_window_bounds;",
        "header_height: root.header_height;",
    ] {
        assert!(
            !native_host_surface.contains(removed_native_host_scalar),
            "native host surface should consume grouped DTO instead of `{removed_native_host_scalar}`"
        );
    }

    let host_native_floating_window_surface = scoped_block_after(
        &host_components,
        "export component HostNativeFloatingWindowSurface inherits Rectangle {",
    );
    assert!(host_native_floating_window_surface.contains(
        "in property <HostNativeFloatingWindowSurfaceData> surface_data;"
    ));
    for removed_native_component_scalar in [
        "in property <[FloatingWindowData]> floating_windows;",
        "in property <string> native_floating_window_id;",
        "in property <FrameRect> native_window_bounds: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };",
        "in property <length> header_height: 31px;",
    ] {
        assert!(
            !host_native_floating_window_surface.contains(removed_native_component_scalar),
            "HostNativeFloatingWindowSurface should drop scalar ABI `{removed_native_component_scalar}`"
        );
    }
    for expected_native_component_usage in [
        "for window[index] in root.surface_data.floating_windows: Rectangle {",
        "visible: window.window_id == root.surface_data.native_floating_window_id;",
        "height: root.surface_data.header_height_px * 1px;",
        "root.surface_data.native_window_bounds.x + self.x / 1px + self.mouse-x / 1px,",
        "root.surface_data.native_window_bounds.y + self.mouse-y / 1px,",
    ] {
        assert!(
            host_native_floating_window_surface.contains(expected_native_component_usage),
            "HostNativeFloatingWindowSurface should read grouped payload via `{expected_native_component_usage}`"
        );
    }
}

#[test]
fn floating_window_overlay_declares_projection_input_and_pane_surface_host() {
    let source = shell_source();

    assert!(source.contains("export struct FloatingWindowData {"));
    assert!(source.contains("in property <HostFloatingWindowLayerData> layer_data;"));
    assert!(source.contains("callback floating_window_header_pointer_clicked(x: float, y: float);"));

    let floating_overlay = block_after(
        &source,
        "for window[index] in root.layer_data.floating_windows: floating_window_card := Rectangle {",
    );
    assert!(floating_overlay.contains("for tab[index] in window.tabs: TabChip {"));
    assert!(floating_overlay.contains("pointer_clicked(x, y) => {"));
    assert!(floating_overlay.contains("WorkbenchHostContext.document_tab_pointer_clicked("));
    assert!(floating_overlay.contains("close_pointer_clicked(x, y) => {"));
    assert!(floating_overlay.contains("WorkbenchHostContext.document_tab_close_pointer_clicked("));
    assert!(floating_overlay.contains("header_touch := TouchArea {"));
    assert!(floating_overlay.contains("WorkbenchHostContext.floating_window_header_pointer_clicked("));
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
        "for window[index] in root.layer_data.floating_windows: floating_window_card := Rectangle {",
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
    let pane_surface = pane_surface_source();
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

    assert!(
        pane_surface.contains("import { AssetBrowserPane"),
        "asset browser pane catalog should live under pane_surface rather than the root host shell"
    );
    assert!(!source.contains("import { AssetBrowserPane"));
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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(
        pane_catalog.contains("callback ui_asset_action(instance_id: string, action_id: string);")
    );
    assert!(pane_catalog
        .contains("callback ui_asset_source_edited(instance_id: string, value: string);"));
    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));

    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "action(action_id) => { PaneSurfaceHostContext.ui_asset_action(root.pane.id, action_id); }"
    ));
    assert!(pane_surface.contains(
        "source_edited(value) => { PaneSurfaceHostContext.ui_asset_source_edited(root.pane.id, value); }"
    ));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
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
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
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
        "root.collection_event(\"preview_mock_subject\", \"selected\", item_index);",
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
        "property <UiAssetCollectionPanelData> collections: root.pane.collections;",
        "property <UiAssetCollectionPanelData> collections: root.pane.collections;",
        "property <UiAssetCollectionPanelData> collections: root.pane.collections;",
        "property <UiAssetSourcePanelData> source_panel: root.pane.source;",
        "property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
    ] {
        assert!(
            pane_block.contains(pane_property),
            "UiAssetEditorPane should declare grouped selection property `{pane_property}`"
        );
    }

    for nested_selection_usage in [
        "items: root.source_panel.detail.outline.items;",
        "selected_index: root.source_panel.detail.outline.selected_index;",
        "items: root.preview_panel.mock.subject_collection.items;",
        "selected_index: root.preview_panel.mock.subject_collection.selected_index;",
        "items: root.style_panel.theme_source.collection.items;",
        "selected_index: root.style_panel.theme_source.collection.selected_index;",
        "items: root.style_panel.matched_rule.collection.items;",
        "selected_index: root.style_panel.matched_rule.collection.selected_index;",
        "items: root.inspector_panel.slot.semantic.collection.items;",
        "selected_index: root.inspector_panel.slot.semantic.collection.selected_index;",
        "items: root.inspector_panel.layout.semantic.collection.items;",
        "selected_index: root.inspector_panel.layout.semantic.collection.selected_index;",
        "items: root.inspector_panel.binding.collection.items;",
        "selected_index: root.inspector_panel.binding.collection.selected_index;",
        "items: root.inspector_panel.binding.event_collection.items;",
        "selected_index: root.inspector_panel.binding.event_collection.selected_index;",
        "items: root.inspector_panel.binding.action_kind_collection.items;",
        "selected_index: root.inspector_panel.binding.action_kind_collection.selected_index;",
        "items: root.inspector_panel.binding.payload_collection.items;",
        "selected_index: root.inspector_panel.binding.payload_collection.selected_index;",
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
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));

    assert!(pane_block.contains("label: \"Open Ref\";"));
    assert!(pane_block.contains("enabled: root.actions.can_open_reference;"));
    assert!(pane_block.contains("active: root.actions.can_open_reference;"));
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
    assert!(
        pane_block.contains("property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;")
    );

    assert!(pane_block.contains("label: \"Docked\";"));
    assert!(pane_block.contains("active: root.preview_panel.preset == \"Editor Docked\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.editor_docked\");"));
    assert!(pane_block.contains("label: \"Float\";"));
    assert!(pane_block.contains("active: root.preview_panel.preset == \"Editor Floating\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.editor_floating\");"));
    assert!(pane_block.contains("label: \"HUD\";"));
    assert!(pane_block.contains("active: root.preview_panel.preset == \"Game HUD\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.game_hud\");"));
    assert!(pane_block.contains("label: \"Dialog\";"));
    assert!(pane_block.contains("active: root.preview_panel.preset == \"Dialog\";"));
    assert!(pane_block.contains("root.action(\"preview.preset.dialog\");"));
}

#[test]
fn ui_asset_editor_pane_declares_slot_aware_external_palette_drop_overlays() {
    let panes = panes_source();
    let canvas_block = block_after(
        &panes,
        "component UiAssetCanvasSurface inherits Rectangle {",
    );

    assert!(canvas_block.contains("root.external_drag.target_action == \"palette.insert.child\""));
    assert!(canvas_block.contains("root.external_drag.target_action == \"palette.insert.after\""));
    assert!(canvas_block.contains("text: root.external_drag.target_label;"));
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
    assert!(
        pane_block.contains("slot_target_items: root.palette_drag_projection.slot_target_items,")
    );

    assert!(canvas_block.contains("in property <UiAssetCanvasDragProjectionData> external_drag;"));
    assert!(canvas_block.contains(
        "for target[index] in root.external_drag.slot_target_items: external_slot_target := Rectangle {"
    ));
    assert!(canvas_block.contains("target.selected ? 2px : 1px;"));
    assert!(canvas_block.contains("target.label"));
    assert!(canvas_block.contains("root.external_drag.slot_target_items.length == 0"));
}

#[test]
fn ui_asset_editor_pane_declares_mock_preview_controls_and_callbacks() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_catalog.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(
        pane_block.contains("property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;")
    );
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(panes.contains("title: \"Mock Subjects\";"));
    assert!(panes.contains("items: root.preview_panel.mock.subject_collection.items;"));
    assert!(panes
        .contains("selected_index: root.preview_panel.mock.subject_collection.selected_index;"));
    assert!(panes
        .contains("root.collection_event(\"preview_mock_subject\", \"selected\", item_index);"));
    assert!(panes.contains("text: root.preview_panel.mock.property;"));
    assert!(panes.contains("text: root.preview_panel.mock.kind;"));
    assert!(panes.contains("root.preview_panel.mock.value;"));
    assert!(panes.contains("root.detail_event(\"preview_mock\", \"preview.mock.value.set\""));
    assert!(panes.contains("root.detail_event(\"preview_mock\", \"preview.mock.clear\", root.preview_panel.mock.collection.selected_index, \"\", \"\");"));
    assert!(panes.contains("title: \"Preview State Graph\";"));
    assert!(panes.contains("items: root.preview_panel.mock.state_graph_items;"));
}

#[test]
fn ui_asset_editor_pane_declares_nested_mock_preview_controls_and_callbacks() {
    let panes = panes_source();
    let preview_struct_block = block_after(&panes, "export struct UiAssetPreviewMockData {");

    assert!(preview_struct_block.contains("nested_collection: UiAssetStringSelectionData,"));
    assert!(preview_struct_block.contains("nested_key: string,"));
    assert!(preview_struct_block.contains("nested_kind: string,"));
    assert!(preview_struct_block.contains("nested_value: string,"));
    assert!(preview_struct_block.contains("nested_can_edit: bool,"));
    assert!(preview_struct_block.contains("nested_can_add: bool,"));
    assert!(preview_struct_block.contains("nested_can_delete: bool,"));
    assert!(panes.contains("title: \"Nested Mock Entries\";"));
    assert!(panes.contains("items: root.preview_panel.mock.nested_collection.items;"));
    assert!(
        panes.contains("selected_index: root.preview_panel.mock.nested_collection.selected_index;")
    );
    assert!(
        panes.contains("root.collection_event(\"preview_mock_nested\", \"selected\", item_index);")
    );
    assert!(panes.contains("text: root.preview_panel.mock.nested_key;"));
    assert!(panes.contains("text: root.preview_panel.mock.nested_kind;"));
    assert!(panes.contains("root.preview_panel.mock.nested_value;"));
    assert!(panes
        .contains("root.detail_event(\"preview_mock_nested\", \"preview.mock.nested.value.set\""));
    assert!(
        panes.contains("root.detail_event(\"preview_mock_nested\", \"preview.mock.nested.upsert\"")
    );
    assert!(panes.contains(
        "root.detail_event(\"preview_mock_nested\", \"preview.mock.nested.delete\", root.preview_panel.mock.nested_collection.selected_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_preview_mock_suggestion_controls_and_callback() {
    let panes = panes_source();
    let preview_struct_block = block_after(&panes, "export struct UiAssetPreviewMockData {");

    assert!(preview_struct_block.contains("suggestion_collection: UiAssetStringSelectionData,"));
    assert!(panes.contains("title: \"Mock Suggestions\";"));
    assert!(panes.contains("items: root.preview_panel.mock.suggestion_collection.items;"));
    assert!(panes.contains(
        "selected_index: root.preview_panel.mock.suggestion_collection.selected_index;"
    ));
    assert!(panes.contains(
        "root.detail_event(\"preview_mock_suggestion\", \"preview.mock.suggestion.apply\", item_index, \"\", \"\");"
    ));
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

    assert!(pane_catalog.contains(
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
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
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
        "property <UiAssetSourcePanelData> source_panel: root.pane.source;",
        "property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
        "property <UiAssetStylePanelData> style_panel: root.pane.style;",
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
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_style_class_action(instance_id: string, action_id: string, class_name: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "style_class_action(action_id, class_name) => { PaneSurfaceHostContext.ui_asset_style_class_action(root.pane.id, action_id, class_name); }"
    ));

    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(
        pane_block.contains("callback style_class_action(action_id: string, class_name: string);")
    );
    assert!(panes.contains("property <string> style_class_draft: \"\";"));
    assert!(panes.contains("for class_name in root.style_panel.class_items: Text {"));
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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("property <string> style_rule_selector_draft: \"\";"));
    assert!(panes.contains("title: \"Rules\";"));
    assert!(panes.contains("items: root.style_panel.rule.items;"));
    assert!(panes.contains("selected_index: root.style_panel.rule.selected_index;"));
    assert!(
        panes.contains("root.style_rule_selector_draft = root.style_panel.rule.items[item_index];")
    );
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.select\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("placeholder: \"selector\";"));
    assert!(panes.contains("label: \"Apply\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.rename\", root.style_panel.rule.selected_index, root.style_rule_selector_draft != \"\" ? root.style_rule_selector_draft : root.style_panel.rule.selected_selector, \"\");"
    ));
    assert!(panes.contains("label: \"Delete\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.delete\", root.style_panel.rule.selected_index, \"\", \"\");"
    ));
    assert!(panes.contains("label: \"Up\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.move_up\", root.style_panel.rule.selected_index, \"\", \"\");"
    ));
    assert!(panes.contains("label: \"Down\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.move_down\", root.style_panel.rule.selected_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_style_token_editing_controls_and_callback() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("property <string> style_token_name_draft: \"\";"));
    assert!(panes.contains("property <string> style_token_value_draft: \"\";"));
    assert!(panes.contains("title: \"Tokens\";"));
    assert!(panes.contains("items: root.style_panel.token.items;"));
    assert!(panes.contains("selected_index: root.style_panel.token.selected_index;"));
    assert!(panes.contains("root.style_token_name_draft = root.style_panel.token.selected_name;"));
    assert!(panes.contains("root.style_token_value_draft = root.style_panel.token.selected_value;"));
    assert!(panes.contains("placeholder: \"token-name\";"));
    assert!(panes.contains("placeholder: \"token-value\";"));
    assert!(panes.contains("label: \"Apply\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_token\", \"style.token.upsert\", root.style_panel.token.selected_index, root.style_token_name_draft != \"\" ? root.style_token_name_draft : root.style_panel.token.selected_name, root.style_token_value_draft != \"\" ? root.style_token_value_draft : root.style_panel.token.selected_value);"
    ));
    assert!(panes.contains("label: \"Delete\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_token\", \"style.token.delete\", root.style_panel.token.selected_index, \"\", \"\");"
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
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));

    assert!(panes.contains("title: \"Theme Sources\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.collection.items;"));
    assert!(
        panes.contains("selected_index: root.style_panel.theme_source.collection.selected_index;")
    );
    assert!(panes.contains("root.action(\"theme.source.select.\" + item_index);"));
    assert!(panes.contains("text: root.style_panel.theme_source.selected_source_kind != \"\" ? root.style_panel.theme_source.selected_source_kind + \" Theme\" : \"No theme source selected\";"));
    assert!(panes.contains("title: \"Theme Tokens\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.selected_source_token_items;"));
    assert!(panes.contains("title: \"Theme Rules\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.selected_source_rule_items;"));
}

#[test]
fn ui_asset_editor_pane_declares_style_rule_declaration_editing_controls_and_callback() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("property <string> style_rule_declaration_path_draft: \"\";"));
    assert!(panes.contains("property <string> style_rule_declaration_value_draft: \"\";"));
    assert!(panes.contains("title: \"Declarations\";"));
    assert!(panes.contains("items: root.style_panel.rule_declaration.items;"));
    assert!(panes.contains("selected_index: root.style_panel.rule_declaration.selected_index;"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule_declaration\", \"style.rule.declaration.select\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains(
        "root.style_rule_declaration_path_draft = root.style_panel.rule_declaration.selected_path;"
    ));
    assert!(panes.contains(
        "root.style_rule_declaration_value_draft = root.style_panel.rule_declaration.selected_value;"
    ));
    assert!(panes.contains("placeholder: \"self.background.color\";"));
    assert!(panes.contains("placeholder: \"value\";"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule_declaration\", \"style.rule.declaration.upsert\", root.style_panel.rule_declaration.selected_index, root.style_rule_declaration_path_draft != \"\" ? root.style_rule_declaration_path_draft : root.style_panel.rule_declaration.selected_path, root.style_rule_declaration_value_draft != \"\" ? root.style_rule_declaration_value_draft : root.style_panel.rule_declaration.selected_value);"
    ));
    assert!(panes.contains(
        "root.detail_event(\"style_rule_declaration\", \"style.rule.declaration.delete\", root.style_panel.rule_declaration.selected_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_matched_rule_inspection_controls_and_callback() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));

    assert!(panes.contains("title: \"Matched Rules\";"));
    assert!(panes.contains("items: root.style_panel.matched_rule.collection.items;"));
    assert!(
        panes.contains("selected_index: root.style_panel.matched_rule.collection.selected_index;")
    );
    assert!(
        panes.contains("root.collection_event(\"matched_style_rule\", \"selected\", item_index);")
    );
    assert!(panes.contains("text: root.style_panel.matched_rule.selected_origin != \"\" ? root.style_panel.matched_rule.selected_origin : \"No matched rule selected\";"));
    assert!(panes.contains("text: root.style_panel.matched_rule.selected_selector;"));
    assert!(panes.contains("text: root.style_panel.matched_rule.selected_specificity >= 0 ? \"specificity \" + root.style_panel.matched_rule.selected_specificity + \" • order \" + root.style_panel.matched_rule.selected_source_order : \"\";"));
    assert!(panes
        .contains("for item in root.style_panel.matched_rule.selected_declaration_items: Text {"));
}

#[test]
fn ui_asset_editor_pane_declares_widget_inspector_editing_controls_and_callback() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("text: \"Widget\";"));
    assert!(panes.contains("text: \"Node\";"));
    assert!(panes.contains("text: \"Control Id\";"));
    assert!(panes.contains("text: \"Text\";"));
    assert!(panes.contains(
        "text: root.inspector_panel.widget.selected_node_id != \"\" ? root.inspector_panel.widget.selected_node_id : \"No selection\";"
    ));
    assert!(panes.contains("text: root.inspector_panel.widget.control_id;"));
    assert!(panes.contains("text: root.inspector_panel.widget.text_prop;"));
    assert!(panes
        .contains("root.detail_event(\"widget\", \"widget.control_id.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"widget\", \"widget.text.set\", -1, value, \"\");"));
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
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));

    assert!(panes.contains("text: \"Slot\";"));
    assert!(panes.contains("text: \"Mount\";"));
    assert!(panes.contains("text: \"Padding\";"));
    assert!(panes.contains("text: \"Width\";"));
    assert!(panes.contains("text: \"Height\";"));
    assert!(panes.contains("text: root.inspector_panel.slot.padding;"));
    assert!(panes.contains("text: root.inspector_panel.slot.width_preferred;"));
    assert!(panes.contains("text: root.inspector_panel.slot.height_preferred;"));
    assert!(panes.contains("root.detail_event(\"slot\", \"slot.mount.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"slot\", \"slot.padding.set\", -1, value, \"\");"));
    assert!(panes.contains(
        "root.detail_event(\"slot\", \"slot.layout.width.preferred.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"slot\", \"slot.layout.height.preferred.set\", -1, value, \"\");"
    ));
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
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));

    assert!(panes.contains("text: \"Layout\";"));
    assert!(panes.contains("text: root.inspector_panel.layout.width_preferred;"));
    assert!(panes.contains("text: root.inspector_panel.layout.height_preferred;"));
    assert!(panes.contains(
        "root.detail_event(\"layout\", \"layout.width.preferred.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"layout\", \"layout.height.preferred.set\", -1, value, \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_parent_specific_semantic_inspector_controls() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));

    assert!(panes.contains("text: root.inspector_panel.slot.semantic.title;"));
    assert!(panes.contains("text: root.inspector_panel.layout.semantic.title;"));
    assert!(panes.contains("items: root.inspector_panel.slot.semantic.collection.items;"));
    assert!(panes.contains("items: root.inspector_panel.layout.semantic.collection.items;"));
    assert!(panes.contains("root.collection_event(\"slot_semantic\", \"selected\", item_index);"));
    assert!(panes.contains("root.collection_event(\"layout_semantic\", \"selected\", item_index);"));
    assert!(panes
        .contains("root.detail_event(\"slot\", \"slot.semantic.value.set\", -1, value, \"\");"));
    assert!(
        panes.contains("root.detail_event(\"slot\", \"slot.semantic.delete\", -1, \"\", \"\");")
    );
    assert!(panes.contains(
        "root.detail_event(\"layout\", \"layout.semantic.value.set\", -1, value, \"\");"
    ));
    assert!(panes
        .contains("root.detail_event(\"layout\", \"layout.semantic.delete\", -1, \"\", \"\");"));
}

#[test]
fn ui_asset_editor_pane_declares_binding_inspector_editing_controls() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_catalog.contains(
        "callback ui_asset_detail_event(instance_id: string, detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_surface.contains(
        "detail_event(detail_id, action_id, item_index, primary, secondary) => { PaneSurfaceHostContext.ui_asset_detail_event(root.pane.id, detail_id, action_id, item_index, primary, secondary); }"
    ));

    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_block.contains(
        "callback detail_event(detail_id: string, action_id: string, item_index: int, primary: string, secondary: string);"
    ));

    assert!(panes.contains("text: \"Bindings\";"));
    assert!(panes.contains("root.detail_event(\"binding\", \"binding.add\", -1, \"\", \"\");"));
    assert!(panes.contains("root.detail_event(\"binding\", \"binding.delete\", -1, \"\", \"\");"));
    assert!(panes.contains("root.detail_event(\"binding\", \"binding.id.set\", -1, value, \"\");"));
    assert!(
        panes.contains("root.detail_event(\"binding\", \"binding.route.set\", -1, value, \"\");")
    );
    assert!(panes.contains("title: \"Event\";"));
    assert!(panes.contains("items: root.inspector_panel.binding.event_collection.items;"));
    assert!(panes
        .contains("selected_index: root.inspector_panel.binding.event_collection.selected_index;"));
    assert!(panes.contains("root.collection_event(\"binding_event\", \"selected\", item_index);"));
    assert!(panes.contains("title: \"Action Kind\";"));
    assert!(panes.contains("items: root.inspector_panel.binding.action_kind_collection.items;"));
    assert!(panes.contains(
        "selected_index: root.inspector_panel.binding.action_kind_collection.selected_index;"
    ));
    assert!(
        panes.contains("root.collection_event(\"binding_action_kind\", \"selected\", item_index);")
    );
    assert!(panes.contains("title: \"Payload\";"));
    assert!(panes.contains("items: root.inspector_panel.binding.payload_collection.items;"));
    assert!(panes.contains(
        "selected_index: root.inspector_panel.binding.payload_collection.selected_index;"
    ));
    assert!(panes.contains("root.collection_event(\"binding_payload\", \"selected\", item_index);"));
    assert!(panes.contains("root.detail_event(\"binding_payload\", \"binding.payload.upsert\""));
    assert!(panes.contains("root.detail_event(\"binding_payload\", \"binding.payload.delete\", root.inspector_panel.binding.payload_collection.selected_index, \"\", \"\");"));
}

#[test]
fn ui_asset_editor_pane_declares_binding_payload_suggestion_controls_and_callback() {
    let panes = panes_source();
    let binding_struct_block = block_after(&panes, "export struct UiAssetInspectorBindingData {");

    assert!(
        binding_struct_block.contains("payload_suggestion_collection: UiAssetStringSelectionData,")
    );
    assert!(panes.contains("title: \"Payload Suggestions\";"));
    assert!(
        panes.contains("items: root.inspector_panel.binding.payload_suggestion_collection.items;")
    );
    assert!(panes.contains(
        "selected_index: root.inspector_panel.binding.payload_suggestion_collection.selected_index;"
    ));
    assert!(panes.contains(
        "root.detail_event(\"binding_payload_suggestion\", \"binding.payload.suggestion.apply\", item_index, \"\", \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_palette_tree_authoring_and_selection_sync_controls() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));

    assert!(pane_block
        .contains("property <UiAssetCollectionPanelData> collections: root.pane.collections;"));
    assert!(pane_block
        .contains("property <UiAssetCollectionPanelData> collections: root.pane.collections;"));
    assert!(pane_block
        .contains("property <UiAssetCollectionPanelData> collections: root.pane.collections;"));
    assert!(
        pane_block.contains("property <UiAssetSourcePanelData> source_panel: root.pane.source;")
    );
    assert!(
        pane_block.contains("property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;")
    );
    assert!(
        pane_block.contains("property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;")
    );
    assert!(
        pane_block.contains("property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;")
    );
    assert!(pane_block.contains(
        "callback collection_event(collection_id: string, event_kind: string, item_index: int);"
    ));

    assert!(panes.contains("export struct UiAssetCanvasNodeData {"));
    assert!(panes.contains("export struct UiAssetCanvasDragProjectionData {"));
    assert!(panes.contains("component UiAssetCanvasSurface inherits Rectangle {"));
    assert!(panes.contains("text: \"Designer Canvas\";"));
    assert!(panes.contains("canvas: root.preview_panel.canvas;"));
    assert!(panes.contains("selection: root.collections.preview;"));
    assert!(panes.contains("action_state: root.actions;"));
    assert!(panes.contains("title: \"Render Stack\";"));
    assert!(panes.contains("title: \"Source Outline\";"));
    assert!(panes.contains("selected_index: root.source_panel.detail.outline.selected_index;"));
    assert!(panes.contains("root.collection_event(\"source_outline\", \"selected\", item_index);"));
    assert!(panes.contains("title: \"Palette\";"));
    assert!(panes.contains("selected_index: root.collections.palette.selected_index;"));
    assert!(panes.contains("root.collection_event(\"palette\", \"selected\", item_index);"));
    assert!(panes.contains("selected_index: root.collections.hierarchy.selected_index;"));
    assert!(panes.contains("selected_index: root.collections.preview.selected_index;"));
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
    assert!(panes.contains("text: root.source_panel.detail.block_label != \"\" ? root.source_panel.detail.block_label : \"No source block\";"));
    assert!(panes.contains(
        "text: root.source_panel.detail.selected_line >= 0 ? \"line \" + root.source_panel.detail.selected_line : \"\";"
    ));
    assert!(panes.contains("text: root.source_panel.detail.roundtrip_status;"));
    assert!(panes.contains("text: root.source_panel.detail.selected_excerpt;"));
    assert!(
        panes.contains("desired_cursor_byte_offset: root.source_panel.detail.cursor_byte_offset;")
    );
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
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(panes.contains("label: \"To Ref\";"));
    assert!(panes.contains("enabled: root.actions.can_convert_to_reference;"));
    assert!(panes.contains("active: root.actions.can_convert_to_reference;"));
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
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(panes.contains("label: \"Extract\";"));
    assert!(panes.contains("enabled: root.actions.can_extract_component;"));
    assert!(panes.contains("active: root.actions.can_extract_component;"));
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
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(panes.contains("label: \"Promote\";"));
    assert!(panes.contains("enabled: root.actions.can_promote_to_external_widget;"));
    assert!(panes.contains("active: root.actions.can_promote_to_external_widget;"));
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
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));

    assert!(panes.contains("text: \"Promote Draft\";"));
    assert!(panes.contains("text: \"Asset\";"));
    assert!(panes.contains("text: \"Comp\";"));
    assert!(panes.contains("text: \"Doc\";"));
    assert!(panes.contains("text: root.inspector_panel.widget.promote_asset_id;"));
    assert!(panes.contains("text: root.inspector_panel.widget.promote_component_name;"));
    assert!(panes.contains("text: root.inspector_panel.widget.promote_document_id;"));
    assert!(panes.contains("enabled: root.inspector_panel.widget.can_edit_promote_draft;"));
    assert!(panes.contains(
        "root.detail_event(\"widget_promote\", \"promote.asset_id.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"widget_promote\", \"promote.component_name.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"widget_promote\", \"promote.document_id.set\", -1, value, \"\");"
    ));
}

#[test]
fn ui_asset_editor_pane_declares_hierarchy_activation_callback_and_double_click_binding() {
    let source = shell_source();
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_source_cursor_changed(instance_id: string, byte_offset: int);"
    ));
    assert!(pane_surface.contains(
        "source_cursor_changed(byte_offset) => { PaneSurfaceHostContext.ui_asset_source_cursor_changed(root.pane.id, byte_offset); }"
    ));
    assert!(pane_block.contains("callback source_cursor_changed(byte_offset: int);"));
    assert!(
        pane_block.contains("property <UiAssetSourcePanelData> source_panel: root.pane.source;")
    );
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

    assert!(canvas_block.contains("in property <UiAssetPreviewCanvasData> canvas;"));
    assert!(canvas_block.contains("in property <UiAssetStringSelectionData> selection;"));
    assert!(canvas_block.contains("in property <bool> palette_has_selection: false;"));
    assert!(canvas_block.contains("in property <UiAssetActionStateData> action_state;"));
    assert!(canvas_block.contains("in property <UiAssetCanvasDragProjectionData> external_drag;"));
    assert!(canvas_block.contains("callback action_requested(action_id: string);"));
    assert!(canvas_block.contains("property <int> overlay_index:"));
    assert!(canvas_block.contains("if root.overlay_index >= 0: overlay := Rectangle {"));
    assert!(canvas_block.contains(
        "text: root.canvas.items[root.overlay_index].label + \" • \" + root.canvas.items[root.overlay_index].kind;"
    ));
    assert!(canvas_block.contains("label: \"Add In\";"));
    assert!(canvas_block.contains("enabled: root.action_state.can_insert_child;"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.child\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_insert_after;"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.after\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_move_up;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.move.up\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_move_down;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.move.down\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_reparent_into_previous;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_previous\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_reparent_into_next;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_next\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_reparent_outdent;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.outdent\");"));
    assert!(canvas_block.contains("root.action_requested(\"reference.open\");"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.convert.reference\");"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.extract.component\");"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.promote.widget\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_wrap_in_vertical_box;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.wrap.vertical_box\");"));
    assert!(canvas_block.contains("enabled: root.action_state.can_unwrap;"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.unwrap\");"));

    assert!(pane_surface.contains("pane: root.pane.ui_asset;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));
    assert!(pane_block.contains("property <UiAssetActionStateData> actions: root.pane.actions;"));

    assert!(panes.contains("palette_has_selection: root.collections.palette.selected_index >= 0;"));
    assert!(panes.contains("canvas: root.preview_panel.canvas;"));
    assert!(panes.contains("selection: root.collections.preview;"));
    assert!(panes.contains("action_state: root.actions;"));
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
        "if root.overlay_index >= 0 && root.action_state.can_insert_child: insert_child_target := Rectangle {"
    ));
    assert!(canvas_block.contains("text: \"Insert In\";"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.child\");"));
    assert!(canvas_block.contains(
        "if root.overlay_index >= 0 && root.action_state.can_insert_after: insert_after_target := Rectangle {"
    ));
    assert!(canvas_block.contains("text: \"Insert After\";"));
    assert!(canvas_block.contains("root.action_requested(\"palette.insert.after\");"));
    assert!(
        canvas_block.contains("if root.overlay_index >= 0 && root.action_state.can_reparent_into_previous: reparent_prev_target := Rectangle {")
    );
    assert!(canvas_block.contains("text: \"Into Prev\";"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_previous\");"));
    assert!(
        canvas_block.contains("if root.overlay_index >= 0 && root.action_state.can_reparent_into_next: reparent_next_target := Rectangle {")
    );
    assert!(canvas_block.contains("text: \"Into Next\";"));
    assert!(canvas_block.contains("root.action_requested(\"canvas.reparent.into_next\");"));
    assert!(canvas_block.contains(
        "if root.overlay_index >= 0 && root.action_state.can_reparent_outdent: outdent_target := Rectangle {"
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
    let pane_catalog = pane_surface_source();
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

    assert!(pane_catalog.contains(
        "callback ui_asset_palette_drag_hover(instance_id: string, surface_x: float, surface_y: float);"
    ));
    assert!(pane_catalog.contains("callback ui_asset_palette_drag_drop(instance_id: string);"));
    assert!(pane_catalog.contains("callback ui_asset_palette_drag_cancel(instance_id: string);"));
    assert!(pane_block.contains(
        "property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"
    ));
    assert!(pane_surface.contains(
        "palette_drag_hovered(surface_x, surface_y) => { PaneSurfaceHostContext.ui_asset_palette_drag_hover(root.pane.id, surface_x, surface_y); }"
    ));
    assert!(pane_surface.contains(
        "palette_drag_dropped() => { PaneSurfaceHostContext.ui_asset_palette_drag_drop(root.pane.id); }"
    ));
    assert!(pane_surface.contains(
        "palette_drag_cancelled() => { PaneSurfaceHostContext.ui_asset_palette_drag_cancel(root.pane.id); }"
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

    assert!(canvas_block.contains("in property <UiAssetCanvasDragProjectionData> external_drag;"));
    assert!(canvas_block.contains("out property <float> surface_scale:"));
    assert!(canvas_block.contains("out property <float> surface_origin_x:"));
    assert!(canvas_block.contains("out property <float> surface_origin_y:"));
    assert!(canvas_block.contains("property <bool> external_target_active:"));
    assert!(canvas_block.contains("self.external_target_active ?"));
    assert!(
        canvas_block.contains("if root.external_drag.active: external_drag_overlay := Rectangle {")
    );
    assert!(canvas_block.contains(
        "text: root.external_drag.target_label != \"\" ? root.external_drag.target_label : \"Drop On Canvas\";"
    ));
    assert!(canvas_block.contains("drop_inside_overlay := Rectangle {"));
    assert!(canvas_block.contains("drop_after_overlay := Rectangle {"));
    assert!(pane_block.contains("external_drag: {"));
    assert!(pane_block.contains("active: root.palette_drag_active,"));
    assert!(pane_block.contains("pointer_x: root.palette_drag_pointer_x - preview_canvas.x / 1px,"));
    assert!(pane_block.contains("pointer_y: root.palette_drag_pointer_y - preview_canvas.y / 1px,"));
    assert!(pane_block.contains("target_index: root.palette_drag_projection.target_preview_index,"));
    assert!(pane_block.contains("target_action: root.palette_drag_projection.target_action,"));
    assert!(pane_block.contains("target_label: root.palette_drag_projection.target_label,"));
}

#[test]
fn ui_asset_editor_pane_declares_palette_target_cycle_panel_and_keyboard_controls() {
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
    let drag_overlay = block_after(&panes, "if root.palette_drag_active: TouchArea {");

    assert!(pane_block.contains(
        "property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"
    ));
    assert!(pane_block.contains("title: \"Target Cycle\";"));
    assert!(pane_block.contains("items: root.palette_drag_projection.candidate_items;"));
    assert!(pane_block
        .contains("selected_index: root.palette_drag_projection.candidate_selected_index;"));

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
    let pane_catalog = pane_surface_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_catalog.contains(
        "callback ui_asset_collection_event(instance_id: string, collection_id: string, event_kind: string, item_index: int);"
    ));
    assert!(pane_catalog.contains("callback ui_asset_palette_target_confirm(instance_id: string);"));
    assert!(pane_catalog.contains("callback ui_asset_palette_target_cancel(instance_id: string);"));
    assert!(pane_block.contains(
        "property <UiAssetPaletteDragData> palette_drag_projection: root.pane.palette_drag;"
    ));
    assert!(pane_surface.contains(
        "collection_event(collection_id, event_kind, item_index) => { PaneSurfaceHostContext.ui_asset_collection_event(root.pane.id, collection_id, event_kind, item_index); }"
    ));
    assert!(pane_surface.contains(
        "palette_target_confirm() => { PaneSurfaceHostContext.ui_asset_palette_target_confirm(root.pane.id); }"
    ));
    assert!(pane_surface.contains(
        "palette_target_cancel() => { PaneSurfaceHostContext.ui_asset_palette_target_cancel(root.pane.id); }"
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
    let _pane_surface = block_after(
        &source,
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
    );

    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));

    assert!(
        panes.contains("root.detail_event(\"layout\", \"layout.box.gap.set\", -1, value, \"\");")
    );
    assert!(panes
        .contains("root.detail_event(\"slot\", \"slot.overlay.anchor_x.set\", -1, value, \"\");"));
    assert!(panes.contains("root.detail_event(\"slot\", \"slot.grid.row.set\", -1, value, \"\");"));
    assert!(panes
        .contains("root.detail_event(\"slot\", \"slot.flow.alignment.set\", -1, value, \"\");"));
    assert!(panes.contains(
        "root.detail_event(\"slot\", \"slot.linear.width_weight.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"slot\", \"slot.linear.width_stretch.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"slot\", \"slot.linear.height_weight.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"slot\", \"slot.linear.height_stretch.set\", -1, value, \"\");"
    ));
    assert!(panes
        .contains("root.detail_event(\"layout\", \"layout.scroll.axis.set\", -1, value, \"\");"));
    assert!(panes.contains(
        "root.detail_event(\"layout\", \"layout.scroll.scrollbar_visibility.set\", -1, value, \"\");"
    ));
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(pane_block
        .contains("property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"));
    assert!(panes.contains("text: root.inspector_panel.layout.box_gap;"));
    assert!(panes.contains("text: root.inspector_panel.slot.linear_main_weight;"));
    assert!(panes.contains("text: root.inspector_panel.binding.binding_route_target;"));
    assert!(panes.contains("text: root.inspector_panel.binding.binding_action_target;"));
    assert!(panes.contains(
        "root.detail_event(\"binding\", \"binding.route_target.set\", -1, value, \"\");"
    ));
    assert!(panes.contains(
        "root.detail_event(\"binding\", \"binding.action_target.set\", -1, value, \"\");"
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

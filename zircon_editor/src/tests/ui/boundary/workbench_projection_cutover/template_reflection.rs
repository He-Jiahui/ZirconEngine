use super::*;

#[test]
fn workbench_main_interface_entries_are_template_backed_and_reflected() {
    let shared_chrome_assets = [
        "/assets/ui/editor/workbench_menu_chrome.zui",
        "/assets/ui/editor/workbench_menu_popup.zui",
        "/assets/ui/editor/workbench_page_chrome.zui",
        "/assets/ui/editor/workbench_dock_header.zui",
        "/assets/ui/editor/workbench_status_bar.zui",
        "/assets/ui/editor/workbench_activity_rail.zui",
    ];
    for asset in shared_chrome_assets {
        assert_asset_exists(asset);
    }

    for asset in [
        "assets/ui/editor/host/editor_main_frame.zui",
        "assets/ui/editor/host/workbench_shell.zui",
        "assets/ui/editor/host/floating_window_source.zui",
        "assets/ui/editor/host/scene_viewport_toolbar.zui",
        "assets/ui/editor/host/pane_surface_controls.zui",
    ] {
        assert_asset_exists(asset);
    }

    let chrome_projection = source_file(&[
        "src",
        "ui",
        "layouts",
        "windows",
        "workbench_host_window",
        "chrome_template_projection.rs",
    ]);
    for asset in shared_chrome_assets {
        assert_contains("chrome_template_projection.rs", &chrome_projection, asset);
    }
    for forbidden in [
        "workbench_document_dock_header.ui.toml",
        "workbench_side_dock_header.ui.toml",
        "workbench_bottom_dock_header.ui.toml",
    ] {
        assert_does_not_contain(
            "chrome_template_projection.rs",
            &chrome_projection,
            forbidden,
        );
    }
    for required in [
        "build_view_template_node_projection(",
        "compose_view_template_node_model(",
        "surface_metrics_from_chrome_assets(",
        "menu_chrome_nodes(",
        "menu_popup_nodes(",
        "page_chrome_nodes(",
        "document_dock_header_nodes(",
        "side_dock_header_nodes(",
        "bottom_dock_header_nodes(",
        "floating_window_header_nodes(",
        "activity_rail_nodes(",
    ] {
        assert_contains(
            "chrome_template_projection.rs",
            &chrome_projection,
            required,
        );
    }
    assert_does_not_contain(
        "chrome_template_projection.rs",
        &chrome_projection,
        "build_view_template_nodes(",
    );
    for forbidden in [
        "HostMenuHitTable",
        "ControlHitTable",
        "DrawerHitTable",
        "FloatingWindowHitTable",
        "DocumentPaneHitTable",
        "SceneToolbarHitTable",
    ] {
        assert_does_not_contain(
            "chrome_template_projection.rs",
            &chrome_projection,
            forbidden,
        );
    }

    let shell_presentation = source_file(&[
        "src",
        "ui",
        "layouts",
        "windows",
        "workbench_host_window",
        "shell_presentation.rs",
    ]);
    for required in [
        "HostWindowSurfaceData",
        "host_tabs:",
        "left_tabs:",
        "right_tabs:",
        "bottom_tabs:",
        "document_tabs:",
        "floating_windows:",
        "left_pane: side_pane(",
        "right_pane: side_pane(",
        "bottom_pane: side_pane(",
        "document_pane: document_pane(",
    ] {
        assert_contains("shell_presentation.rs", &shell_presentation, required);
    }

    let pane_projection = source_file(&[
        "src",
        "ui",
        "layouts",
        "windows",
        "workbench_host_window",
        "pane_projection.rs",
    ]);
    for required in [
        "side_pane(",
        "document_pane(",
        "pane_from_tab(",
        "build_pane_presentation(",
        "build_pane_body_presentation(",
        "PanePresentation::new(",
        "PaneShellPresentation::new(",
        "scene_viewport_chrome(",
    ] {
        assert_contains("pane_projection.rs", &pane_projection, required);
    }

    let reflection = source_tree(&["src", "ui", "reflection"]);
    for required in [
        "pub struct EditorWorkbenchReflectionModel",
        "menu_items: Vec<EditorMenuItemReflectionModel>",
        "pages: Vec<EditorHostPageReflectionModel>",
        "drawers: Vec<EditorDrawerReflectionModel>",
        "floating_windows: Vec<EditorFloatingWindowReflectionModel>",
        "\"editor/workbench/menu\"",
        "\"MenuBar\"",
        "\"editor/workbench/pages\"",
        "\"PageCollection\"",
        "\"editor/workbench/drawers\"",
        "\"DrawerCollection\"",
        "\"editor/workbench/floating\"",
        "\"FloatingWindows\"",
        "EditorActivityHost::Drawer(_) => \"drawer\"",
        "EditorActivityHost::DocumentPage(_) => \"document_page\"",
        "EditorActivityHost::FloatingWindow(_) => \"floating_window\"",
        "EditorActivityHost::ExclusivePage(_) => \"exclusive_page\"",
    ] {
        assert_contains("reflection/", &reflection, required);
    }
}

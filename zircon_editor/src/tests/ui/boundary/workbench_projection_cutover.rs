use super::support::collect_rust_files;

fn source_file(path: &[&str]) -> String {
    let mut file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for segment in path {
        file.push(segment);
    }
    std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("expected readable source {file:?}"))
}

fn assert_asset_exists(asset_path: &str) {
    let file =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(asset_path.trim_start_matches('/'));
    assert!(
        file.exists(),
        "expected workbench template asset to exist at {file:?}"
    );
}

fn assert_contains(source_name: &str, source: &str, pattern: &str) {
    assert!(
        source.contains(pattern),
        "expected {source_name} to contain `{pattern}`"
    );
}

fn assert_does_not_contain(source_name: &str, source: &str, pattern: &str) {
    assert!(
        !source.contains(pattern),
        "expected {source_name} to avoid host-only cutover path `{pattern}`"
    );
}

fn assert_no_active_slint_files(root: &std::path::Path) {
    if !root.exists() {
        return;
    }
    for entry in std::fs::read_dir(root).unwrap_or_else(|_| panic!("expected readable {root:?}")) {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            assert_no_active_slint_files(&path);
            continue;
        }
        assert_ne!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("slint"),
            "active editor host tree must not keep business Slint source `{}`",
            path.display()
        );
    }
}

fn slint_host_import_blocks(source: &str) -> Vec<String> {
    let normalized = source.split_whitespace().collect::<String>();
    let mut blocks = Vec::new();
    let mut rest = normalized.as_str();

    while let Some(start) = rest.find("usecrate::ui::slint_host::{") {
        let after_start = &rest[start..];
        let Some(end) = after_start.find("};") else {
            break;
        };
        blocks.push(after_start[..end + 2].to_string());
        rest = &after_start[end + 2..];
    }

    blocks
}

#[test]
fn workbench_main_interface_entries_are_template_backed_and_reflected() {
    let shared_chrome_assets = [
        "/assets/ui/editor/workbench_menu_chrome.ui.toml",
        "/assets/ui/editor/workbench_menu_popup.ui.toml",
        "/assets/ui/editor/workbench_page_chrome.ui.toml",
        "/assets/ui/editor/workbench_dock_header.ui.toml",
        "/assets/ui/editor/workbench_status_bar.ui.toml",
        "/assets/ui/editor/workbench_activity_rail.ui.toml",
    ];
    for asset in shared_chrome_assets {
        assert_asset_exists(asset);
    }

    for asset in [
        "assets/ui/editor/host/editor_main_frame.ui.toml",
        "assets/ui/editor/host/workbench_shell.ui.toml",
        "assets/ui/editor/host/workbench_drawer_source.ui.toml",
        "assets/ui/editor/host/floating_window_source.ui.toml",
        "assets/ui/editor/host/workbench_document_dock_header.ui.toml",
        "assets/ui/editor/host/workbench_side_dock_header.ui.toml",
        "assets/ui/editor/host/workbench_bottom_dock_header.ui.toml",
        "assets/ui/editor/host/scene_viewport_toolbar.ui.toml",
        "assets/ui/editor/host/pane_surface_controls.ui.toml",
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
    for required in [
        "build_view_template_nodes(",
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

    let reflection = source_file(&["src", "ui", "reflection.rs"]);
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
        assert_contains("reflection.rs", &reflection, required);
    }
}

#[test]
fn workbench_host_pointer_paths_are_shared_surface_bridges_not_host_hit_tables() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_no_active_slint_files(&manifest.join("ui"));

    let slint_host_root = manifest.join("src").join("ui").join("slint_host");
    for path in collect_rust_files(&slint_host_root) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        for forbidden in [
            "HitTable",
            "hit_table",
            "PointerTable",
            "pointer_table",
            "ControlHitTable",
            "control_hit_table",
            "BusinessHitTable",
            "business_hit_table",
            "ManualHitTable",
            "manual_hit_table",
        ] {
            assert_does_not_contain(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("slint_host source"),
                &source,
                forbidden,
            );
        }
    }

    for (relative, required) in [
        (
            "src/ui/slint_host/menu_pointer/host_menu_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "targets: BTreeMap<UiNodeId, HostMenuPointerTarget>",
            ][..],
        ),
        (
            "src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "register_handled_pointer_node",
            ][..],
        ),
        (
            "src/ui/slint_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "targets: BTreeMap<UiNodeId, HostActivityRailPointerTarget>",
            ][..],
        ),
        (
            "src/ui/slint_host/activity_rail_pointer/rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "insert_strip(",
            ][..],
        ),
        (
            "src/ui/slint_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "targets: BTreeMap<UiNodeId, HostDrawerHeaderPointerTarget>",
            ][..],
        ),
        (
            "src/ui/slint_host/drawer_header_pointer/rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "register_handled_pointer_node",
            ][..],
        ),
        (
            "src/ui/slint_host/document_tab_pointer/host_document_tab_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "targets:",
                "HostDocumentTabPointerTarget",
            ][..],
        ),
        (
            "src/ui/slint_host/document_tab_pointer/host_document_tab_pointer_bridge_rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "register_handled_pointer_node",
            ][..],
        ),
        (
            "src/ui/slint_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "targets: BTreeMap<UiNodeId, ViewportToolbarPointerTarget>",
            ][..],
        ),
        (
            "src/ui/slint_host/viewport_toolbar_pointer/rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "register_handled_pointer_node",
            ][..],
        ),
        (
            "src/ui/slint_host/shell_pointer/bridge.rs",
            &[
                "drag_surface: UiSurface",
                "drag_dispatcher: UiPointerDispatcher",
                "resize_surface: UiSurface",
                "resize_dispatcher: UiPointerDispatcher",
                ".dispatch_pointer_event(",
            ][..],
        ),
        (
            "src/ui/slint_host/host_contract/surface_hit_test/template_node.rs",
            &[
                "UiSurfaceFrame",
                "hit_test_host_surface_frame",
                "template_nodes_surface_frame",
                "surface.surface_frame()",
            ][..],
        ),
    ] {
        let source = source_file(&[relative]);
        for marker in required {
            assert_contains(relative, &source, marker);
        }
    }
}

#[test]
fn workbench_root_shell_projection_uses_shared_frames_without_geometry_fallback() {
    let projection = source_file(&["src", "ui", "slint_host", "root_shell_projection.rs"]);

    for required in [
        "shared_root_shell_frame(",
        "shared_root_body_frame(",
        "shared_document_region_frame(",
        "shared_visible_drawer_shell_frame(",
        "frames.host_body_frame",
        "frames.status_bar_frame",
        "frames.pane_surface_frame",
    ] {
        assert_contains("root_shell_projection.rs", &projection, required);
    }

    for forbidden in [
        "derive_layout_frames_from_geometry_with_shared_root",
        "root_geometry_region_frame",
        "root_geometry_center_band_frame",
        "root_geometry_status_bar_frame",
        "WorkbenchShellGeometry {",
        ".region_frame(",
        ".splitter_frame(",
        "geometry.region_frame",
        "geometry.splitter_frame",
        "geometry.viewport_content_frame",
    ] {
        assert_does_not_contain("root_shell_projection.rs", &projection, forbidden);
    }
}

#[test]
fn shell_pointer_drag_surface_uses_shared_root_frames_without_geometry_fallback() {
    let drag_surface = source_file(&[
        "src",
        "ui",
        "slint_host",
        "shell_pointer",
        "drag_surface.rs",
    ]);

    for required in [
        "resolve_root_center_band_frame(shared_root_frames)",
        "resolve_root_status_bar_frame(shared_root_frames)",
        "resolve_root_document_region_frame(shared_root_frames)",
        "resolve_root_left_region_frame(shared_root_frames)",
        "resolve_root_right_region_frame(shared_root_frames)",
        "resolve_root_bottom_region_frame(shared_root_frames)",
    ] {
        assert_contains("drag_surface.rs", &drag_surface, required);
    }

    for forbidden in [
        "WorkbenchShellGeometry",
        "ShellRegionId",
        "shared_or_fallback_frame",
        "shared_or_geometry_frame",
        "geometry.center_band_frame",
        "geometry.status_bar_frame",
        "geometry.region_frame",
    ] {
        assert_does_not_contain("drag_surface.rs", &drag_surface, forbidden);
    }
}

#[test]
fn shell_pointer_bridge_does_not_recreate_root_frames_from_geometry() {
    let bridge = source_file(&["src", "ui", "slint_host", "shell_pointer", "bridge.rs"]);

    for required in [
        "update_layout_with_root_shell_frames(",
        "build_drag_surface(",
        "update_resize_surface(",
        "shared_root_frames",
    ] {
        assert_contains("bridge.rs", &bridge, required);
    }

    for forbidden in [
        "root_frames_from_geometry",
        "geometry.center_band_frame",
        "geometry.status_bar_frame",
        "geometry.region_frame",
        "ShellRegionId",
    ] {
        assert_does_not_contain("bridge.rs", &bridge, forbidden);
    }
}

#[test]
fn tab_drag_strip_hitbox_uses_shared_root_frames_without_geometry_fallback() {
    let strip_hitbox = source_file(&["src", "ui", "slint_host", "tab_drag", "strip_hitbox.rs"]);

    for required in [
        "resolve_root_center_band_frame(shared_root_frames)",
        "resolve_root_document_region_frame(shared_root_frames)",
        "resolve_root_left_region_frame(shared_root_frames)",
        "resolve_root_right_region_frame(shared_root_frames)",
        "resolve_root_bottom_region_frame(shared_root_frames)",
    ] {
        assert_contains("strip_hitbox.rs", &strip_hitbox, required);
    }

    for forbidden in [
        "WorkbenchShellGeometry",
        "shared_or_geometry_frame",
        "geometry.region_frame",
        "geometry.center_band_frame",
    ] {
        assert_does_not_contain("strip_hitbox.rs", &strip_hitbox, forbidden);
    }
}

#[test]
fn floating_window_projection_uses_shared_source_without_geometry_fallback() {
    let projection = source_file(&["src", "ui", "slint_host", "floating_window_projection.rs"]);

    for required in [
        "resolve_floating_window_projection_shared_source(",
        "build_floating_window_projection_bundle_from_windows_with_shared_source(",
        "resolve_floating_window_outer_frame_from_shared_source(",
        "window.requested_frame",
        "resolve_native_floating_window_host_frame(",
    ] {
        assert_contains("floating_window_projection.rs", &projection, required);
    }

    for forbidden in [
        "floating_window_projection_shared_source_from_geometry",
        "build_floating_window_projection_bundle_from_windows_with_geometry",
        "resolve_floating_window_projected_outer_frame_with_fallback",
        "WorkbenchShellGeometry",
        ".floating_window_frame(",
        "geometry.region_frame",
        "geometry.center_band_frame",
    ] {
        assert_does_not_contain("floating_window_projection.rs", &projection, forbidden);
    }
}

fn block_imports_name(block: &str, name: &str) -> bool {
    [
        format!("{{{name},"),
        format!(",{name},"),
        format!(",{name}}}"),
        format!("{{{name}}}"),
    ]
    .into_iter()
    .any(|pattern| block.contains(&pattern))
}

#[test]
fn workbench_host_window_keeps_host_contract_shell_dtos_at_ui_boundary_only() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("layouts")
        .join("windows")
        .join("workbench_host_window");
    let mod_source = std::fs::read_to_string(root.join("mod.rs")).expect("workbench host mod");
    let apply_presentation = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("slint_host")
            .join("ui")
            .join("apply_presentation.rs"),
    )
    .expect("apply presentation");

    assert!(
        mod_source.contains("mod host_data;"),
        "expected workbench_host_window mod wiring to own Rust host DTOs under host_data.rs"
    );
    assert!(
        root.join("host_data.rs").exists(),
        "expected Rust-owned host DTO declaration file under {:?}",
        root
    );

    for path in collect_rust_files(&root) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        let import_blocks = slint_host_import_blocks(&source);
        if import_blocks.is_empty() && !source.contains("crate::ui::slint_host::") {
            continue;
        }

        for forbidden in [
            "FrameRect",
            "PaneData",
            "TabData",
            "FloatingWindowData",
            "AssetsActivityShellFrameData",
            "AssetsActivityShellLayoutData",
            "AssetsActivityPaneData",
            "HierarchyShellFrameData",
            "HierarchyShellLayoutData",
            "HierarchyPaneData",
            "InspectorShellFrameData",
            "InspectorShellLayoutData",
            "InspectorPaneData",
            "ConsoleShellFrameData",
            "ConsoleShellLayoutData",
            "ConsolePaneData",
            "ProjectOverviewShellFrameData",
            "ProjectOverviewShellLayoutData",
            "ProjectOverviewPaneData",
            "AnimationEditorShellFrameData",
            "AnimationEditorShellLayoutData",
            "AnimationEditorPaneData",
            "UiAssetActionStateData",
            "UiAssetCanvasNodeData",
            "UiAssetCanvasSlotTargetData",
            "UiAssetCollectionPanelData",
            "UiAssetEditorPaneData",
            "UiAssetInspectorBindingData",
            "UiAssetInspectorLayoutData",
            "UiAssetInspectorPanelData",
            "UiAssetInspectorSemanticData",
            "UiAssetInspectorSlotData",
            "UiAssetInspectorWidgetData",
            "UiAssetMatchedStyleRuleData",
            "UiAssetPaletteDragData",
            "UiAssetPaneHeaderData",
            "UiAssetPreviewCanvasData",
            "UiAssetPreviewMockData",
            "UiAssetPreviewPanelData",
            "UiAssetSourceDetailData",
            "UiAssetSourcePanelData",
            "UiAssetStringSelectionData",
            "UiAssetStylePanelData",
            "UiAssetStyleRuleData",
            "UiAssetStyleRuleDeclarationData",
            "UiAssetStyleStateData",
            "UiAssetStyleTokenData",
            "UiAssetThemeSourceData",
            "HostWindowShellData",
            "HostWindowSurfaceData",
            "HostWindowLayoutData",
            "HostWindowSurfaceMetricsData",
            "HostWindowSurfaceOrchestrationData",
            "HostWindowSceneData",
            "HostNativeFloatingWindowSurfaceData",
            "ProjectOverviewData",
            "SceneNodeData",
        ] {
            for block in &import_blocks {
                assert!(
                    !block_imports_name(block, forbidden),
                    "expected {:?} to stop importing host-contract DTO `{forbidden}` into workbench_host_window internals",
                    path.file_name().expect("file name")
                );
            }
            assert!(
                !source.contains(&format!("crate::ui::slint_host::{forbidden}")),
                "expected {:?} to stop importing host-contract DTO `{forbidden}` into workbench_host_window internals",
                path.file_name().expect("file name")
            );
        }
    }

    assert!(
        apply_presentation.contains("fn to_host_contract_hierarchy_pane("),
        "expected apply_presentation.rs to own hierarchy pane conversion at the host-contract boundary"
    );
    assert!(
        apply_presentation.contains("fn to_host_contract_inspector_pane("),
        "expected apply_presentation.rs to own inspector pane conversion at the host-contract boundary"
    );
    assert!(
        apply_presentation.contains("fn to_host_contract_console_pane("),
        "expected apply_presentation.rs to own console pane conversion at the host-contract boundary"
    );
    assert!(
        apply_presentation.contains("fn to_host_contract_assets_activity_pane("),
        "expected apply_presentation.rs to own assets-activity pane conversion at the host-contract boundary"
    );
    assert!(
        apply_presentation.contains("fn to_host_contract_project_overview_pane("),
        "expected apply_presentation.rs to own project overview pane conversion at the host-contract boundary"
    );
    assert!(
        apply_presentation.contains("fn to_host_contract_animation_editor_pane("),
        "expected apply_presentation.rs to own animation pane conversion at the host-contract boundary"
    );
    assert!(
        apply_presentation.contains("fn to_host_contract_ui_asset_pane("),
        "expected apply_presentation.rs to own ui asset pane conversion at the host-contract boundary"
    );
}

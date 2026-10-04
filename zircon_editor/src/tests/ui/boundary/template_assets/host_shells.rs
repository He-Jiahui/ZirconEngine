use std::fs;
use std::path::Path;

use super::support::{png_dimensions, source};

#[test]
fn host_template_assets_are_toml_authority_for_editor_shells() {
    let assets: &[(&str, &[&str])] = &[
        (
            "assets/ui/editor/components/workbench/shell/activity_drawer_window.zui",
            &[
                "ActivityDrawerWindowRoot",
                "ActivityDrawerWindowContentSlot",
            ],
        ),
        (
            "assets/ui/editor/host/workbench_shell.zui",
            &[
                "UiHostWindow",
                "activity_rail",
                "document_host",
                "menu_bar",
                "editor_workbench_strict.zui",
                "res://ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton",
                "res://ui/editor/components/workbench/primitives/chrome/workbench_rail_button.zui#WorkbenchRailButton",
                "res://ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui#WorkbenchStatusItem",
                "WorkbenchScaffold",
                "StatusBarRoot",
            ],
        ),
        (
            "assets/ui/editor/windows/workbench_window.zui",
            &[
                "editor_workbench_strict.zui",
                "res://ui/editor/components/workbench/shell/workbench_component_drawer.zui#WorkbenchComponentDrawer",
                "res://ui/editor/components/workbench/shell/workbench_main_band.zui#WorkbenchMainBand",
                "res://ui/editor/components/workbench/shell/workbench_status_bar.zui#WorkbenchStatusBar",
                "res://ui/editor/components/workbench/shell/workbench_top_toolbar.zui#WorkbenchTopToolbar",
                "WorkbenchWindowTopToolbarRegion",
                "WorkbenchWindowMainBandRegion",
                "WorkbenchWindowComponentDrawerRegion",
                "WorkbenchWindowStatusBarRegion",
                "BottomDrawerShellRoot",
            ],
        ),
        (
            "assets/ui/editor/host/floating_window_source.zui",
            &["FloatingWindowSourceRoot", "FloatingWindowTopBarRoot"],
        ),
        (
            "assets/ui/editor/host/scene_viewport_toolbar.zui",
            &["SceneViewportToolbarRoot", "FrameSelection"],
        ),
        (
            "assets/ui/editor/host/asset_surface_controls.zui",
            &["AssetSurfaceControls", "OpenAssetBrowser"],
        ),
        (
            "assets/ui/editor/host/inspector_surface_controls.zui",
            &["InspectorSurfaceControls", "DeleteSelected"],
        ),
        (
            "assets/ui/editor/host/startup_welcome_controls.zui",
            &["CreateProject", "OpenExistingProject"],
        ),
    ];

    for (relative, markers) in assets {
        let asset = source(relative);
        for &marker in *markers {
            assert!(asset.contains(marker), "{relative} missing `{marker}`");
        }
    }
}

#[test]
fn workbench_drawer_frame_owners_live_in_real_component_assets() {
    let workbench_window = source("assets/ui/editor/windows/workbench_window.zui");
    for forbidden in [
        "drawer_projection",
        "WorkbenchDrawerSourceRoot",
        "LeftDrawerPanelRoot",
        "RightDrawerPanelRoot",
        "BottomDrawerPanelRoot",
        "BottomDrawerOuterSeparatorRoot",
        "surface_variant = \"frame_only\"",
    ] {
        assert!(
            !workbench_window.contains(forbidden),
            "Workbench window should not retain embedded drawer projection marker `{forbidden}`"
        );
    }

    for (relative, required) in [
        (
            "assets/ui/editor/components/workbench/shell/workbench_main_band.zui",
            &["LeftDrawerShellRoot", "RightDrawerShellRoot"][..],
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui",
            &["LeftDrawerHeaderRoot", "LeftDrawerContentRoot"][..],
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui",
            &["RightDrawerHeaderRoot", "RightDrawerContentRoot"][..],
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui",
            &["BottomDrawerHeaderRoot", "BottomDrawerContentRoot"][..],
        ),
    ] {
        let asset = source(relative);
        for marker in required {
            assert!(
                asset.contains(marker),
                "drawer frame owner `{marker}` should live in real component asset `{relative}`"
            );
        }
    }
}

#[test]
fn workbench_reference_visual_asset_remains_design_baseline_not_runtime_overlay() {
    let editor_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = editor_root
        .parent()
        .expect("zircon_editor lives directly under workspace root");
    let docs_path = repo_root.join("docs/ui-and-layout/workbench.png");
    let asset_path = editor_root.join("assets/ui/editor/reference/workbench.png");

    let docs_bytes = fs::read(&docs_path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", docs_path.display()));
    let asset_bytes = fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", asset_path.display()));

    assert_eq!(
        asset_bytes.len(),
        docs_bytes.len(),
        "editor workbench reference asset should keep the docs baseline byte length"
    );
    assert!(
        asset_bytes == docs_bytes,
        "editor workbench reference asset should stay byte-identical to docs/ui-and-layout/workbench.png"
    );
    assert_eq!(png_dimensions(&asset_bytes), (1672, 941));

    let workbench = source("assets/ui/editor/windows/workbench_window.zui");
    for forbidden in [
        "component = \"IconButton\"",
        "component = \"Button\"",
        "component = \"Dropdown\"",
        "component = \"Checkbox\"",
        "component = \"Radio\"",
        "component = \"Toggle\"",
        "component = \"RangeField\"",
        "component = \"TreeRow\"",
        "component = \"ListRow\"",
        "component = \"ContextActionMenu\"",
    ] {
        assert!(
            !workbench.contains(forbidden),
            "workbench window must route interaction primitives through Workbench* .zui components instead of `{forbidden}`"
        );
    }
    for forbidden in [
        "WorkbenchReferenceFrame",
        "WorkbenchReferenceImage",
        "ui/editor/reference/workbench.png",
        "docs/ui-and-layout/workbench.png",
    ] {
        assert!(
            !workbench.contains(forbidden),
            "workbench window must not render full PNG reference `{forbidden}`"
        );
    }

    let shell = source("assets/ui/editor/host/workbench_shell.zui");
    for forbidden in [
        "WorkbenchShellReferenceImage",
        "ui/editor/reference/workbench.png",
        "docs/ui-and-layout/workbench.png",
        "input_policy = \"Ignore\"",
    ] {
        assert!(
            !shell.contains(forbidden),
            "workbench shell must not render full PNG reference `{forbidden}`"
        );
    }

    let builtin_documents = crate::ui::template_runtime::builtin::builtin_template_documents();
    for (document_id, expected_suffix) in [
        (
            "res://ui/editor/host/workbench_shell.zui",
            "assets/ui/editor/host/workbench_shell.zui",
        ),
        (
            "res://ui/editor/windows/workbench_window.zui",
            "assets/ui/editor/windows/workbench_window.zui",
        ),
    ] {
        let route_path = builtin_documents
            .iter()
            .find_map(|(candidate, path)| (*candidate == document_id).then_some(path))
            .unwrap_or_else(|| panic!("builtin template registry missing `{document_id}`"));
        let normalized = route_path.to_string_lossy().replace('\\', "/");
        assert!(
            normalized.ends_with(expected_suffix),
            "builtin template `{document_id}` should route to `{expected_suffix}`, got `{normalized}`"
        );
    }
}

#[test]
fn critical_editor_shells_are_hard_cut_to_zui_assets() {
    let registry = source("src/ui/template_runtime/builtin/template_documents.rs");
    let runtime_host = source("src/ui/template_runtime/runtime/runtime_host.rs");
    for required in [
        "editor_main_frame.zui",
        "workbench_window.zui",
        "asset_window.zui",
        "ui_layout_editor_window.zui",
        "component_showcase.zui",
        "material_demo_window.zui",
        "material_component_lab.zui",
        "workbench_shell.zui",
        "floating_window_source.zui",
        "scene_viewport_toolbar.zui",
        "console_body.zui",
        "inspector_body.zui",
        "hierarchy_body.zui",
        "animation_sequence_body.zui",
        "animation_graph_body.zui",
        "runtime_diagnostics_body.zui",
        "performance_timeline_body.zui",
        "module_plugins_body.zui",
        "build_export_desktop_body.zui",
        "asset_surface_controls.zui",
        "startup_welcome_controls.zui",
        "inspector_surface_controls.zui",
        "pane_surface_controls.zui",
    ] {
        assert!(
            registry.contains(required),
            "builtin template registry missing .zui asset `{required}`"
        );
    }
    for component_only in [
        "activity_drawer_window.v2.ui.toml",
        "activity_drawer_window.zui",
        "workbench_drawer_source.v2.ui.toml",
    ] {
        assert!(
            !registry.contains(component_only),
            "drawer source shell is embedded in Workbench window/component assets, not a directly registered builtin document `{component_only}`"
        );
    }

    for forbidden in [
        "editor_main_frame.ui.toml",
        "activity_drawer_window.ui.toml",
        "workbench_window.ui.toml",
        "asset_window.ui.toml",
        "ui_layout_editor_window.ui.toml",
        "component_showcase.ui.toml",
        "material_demo_window.ui.toml",
        "workbench_shell.ui.toml",
        "workbench_drawer_source.ui.toml",
        "floating_window_source.ui.toml",
        "scene_viewport_toolbar.ui.toml",
        "console_body.ui.toml",
        "inspector_body.ui.toml",
        "hierarchy_body.ui.toml",
        "animation_sequence_body.ui.toml",
        "animation_graph_body.ui.toml",
        "runtime_diagnostics_body.ui.toml",
        "module_plugins_body.ui.toml",
        "build_export_desktop_body.ui.toml",
        "asset_surface_controls.ui.toml",
        "startup_welcome_controls.ui.toml",
        "inspector_surface_controls.ui.toml",
        "pane_surface_controls.ui.toml",
    ] {
        assert!(
            !registry.contains(&format!("\"{forbidden}\"")),
            "builtin template registry should not route critical shell through old asset `{forbidden}`"
        );
    }

    for required in [
        "UiV2PrototypeStoreFileCache",
        "v2_template_file_cache()",
        "self.register_v2_document_files(document_id, std::iter::once(path))",
        ".load_store(paths)?",
        "Arc<UiV2AssetDocument>",
        "Arc<UiV2CompiledDocument>",
    ] {
        assert!(
            runtime_host.contains(required),
            "builtin v2 host runtime should use heap-resident file cache marker `{required}`"
        );
    }
    for forbidden in [
        "UiV2AssetLoader",
        "UiV2DocumentCompiler",
        "v2_prototype_store",
    ] {
        assert!(
            !runtime_host.contains(forbidden),
            "builtin v2 host runtime should not keep per-registration deserialize/compile marker `{forbidden}`"
        );
    }

    let view_projection = format!(
        "{}\n{}\n{}",
        source("src/ui/layouts/views/view_projection.rs"),
        source("src/ui/layouts/views/view_projection/store_cache.rs"),
        source("src/ui/layouts/views/view_projection/build.rs")
    );
    for required in [
        "NonV2AssetPath",
        "UiV2PrototypeStoreFileCache",
        "UiV2SurfaceBuilder",
    ] {
        assert!(
            view_projection.contains(required),
            "editor view projection should expose v2 hard-cut marker `{required}`"
        );
    }
    for forbidden in [
        "UiTemplateSurfaceBuilder",
        "UiPrototypeStoreFileCache",
        "UiDocumentCompiler",
        "build_view_template_nodes_from_prototype_store",
    ] {
        assert!(
            !view_projection.contains(forbidden),
            "editor view projection should not keep old schema fallback marker `{forbidden}`"
        );
    }

    let asset_browser = source("src/ui/layouts/views/asset_browser.rs");
    assert!(asset_browser.contains("asset_browser.zui"));
    assert!(!asset_browser.contains("\"/assets/ui/editor/asset_browser.ui.toml\""));

    for (relative, required, forbidden) in [
        (
            "src/ui/layouts/views/console.rs",
            "console.zui",
            "\"/assets/ui/editor/console.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/hierarchy.rs",
            "hierarchy.zui",
            "\"/assets/ui/editor/hierarchy.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/inspector.rs",
            "inspector.zui",
            "\"/assets/ui/editor/inspector.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/assets_activity.rs",
            "assets_activity.zui",
            "\"/assets/ui/editor/assets_activity.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/animation_editor.rs",
            "host/animation_sequence_body.zui",
            "\"/assets/ui/editor/animation_editor.ui.toml\"",
        ),
        (
            "src/ui/layouts/views/welcome.rs",
            "welcome.zui",
            "\"/assets/ui/editor/welcome.ui.toml\"",
        ),
    ] {
        let source = source(relative);
        assert!(
            source.contains(required),
            "{relative} should route projection through .zui asset `{required}`"
        );
        assert!(
            !source.contains(forbidden),
            "{relative} should not route projection through old asset `{forbidden}`"
        );
    }

    let animation_editor = source("src/ui/layouts/views/animation_editor.rs");
    assert!(
        animation_editor.contains("host/animation_graph_body.zui"),
        "animation editor projection must route graph panes through the canonical graph host asset"
    );

    let ui_asset_editor_projection = source("src/ui/asset_editor/node_projection.rs");
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/ui_asset_editor.ui.toml")
            .exists(),
        "UI Asset Editor bootstrap must stay on the .zui authoring asset"
    );
    for required in [
        "ui_asset_editor.zui",
        "UiV2PrototypeStoreFileCache",
        "node_projection_v2_store_file_cache()",
        "UiV2SurfaceBuilder::build_surface_from_compiled_document",
        "surface.compute_layout(size)?",
        "project_ui_asset_editor_nodes(",
    ] {
        assert!(
            ui_asset_editor_projection.contains(required),
            "UI Asset Editor node projection should route through .zui runtime marker `{required}`"
        );
    }
    for forbidden in [
        "\"/assets/ui/editor/ui_asset_editor.ui.toml\"",
        "EditorTemplateRuntimeService",
        "UiCompiledDocument",
        "load_document_file",
        "compile_document_with_import_maps",
    ] {
        assert!(
            !ui_asset_editor_projection.contains(forbidden),
            "UI Asset Editor node projection should not keep old recursive projection marker `{forbidden}`"
        );
    }
}

#[test]
fn welcome_startup_demo_routes_to_component_showcase_window() {
    let welcome_asset = source("assets/ui/editor/welcome.zui");
    assert!(welcome_asset.contains("text = \"Showcase\""));
    assert!(welcome_asset.contains("tooltip = \"Open UI Component Showcase\""));
    assert!(welcome_asset.contains("id = \"Welcome/OpenStartupDemo\""));
    assert!(welcome_asset.contains("route = \"workbench.welcome.open_startup_demo\""));

    let welcome_session = source("src/ui/retained_host/app/welcome_session/actions.rs");
    assert!(welcome_session.contains("OpenStartupDemo"));
    assert!(welcome_session.contains("editor.ui_component_showcase"));
    assert!(!welcome_session.contains("\"editor.material_demo_window\""));
}

use super::support::collect_rust_files;

const FLAT_EDITOR_TEST_LAYOUT_ASSET_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.flat_layout"
version = 1
display_name = "Flat Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }
"#;

#[test]
fn build_script_tracks_nested_slint_sources_recursively() {
    let build_rs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("build.rs");
    let source = std::fs::read_to_string(build_rs).expect("build.rs");

    assert!(
        source.contains("fn emit_rerun_if_changed_recursive(root: &str)"),
        "expected build.rs to define a recursive rerun-if-changed helper for nested ui sources"
    );
    assert!(
        source.contains("emit_rerun_if_changed_recursive(\"ui\")"),
        "expected build.rs to recursively track the entire ui tree instead of only top-level slint paths"
    );
}

#[test]
fn workbench_slint_entry_stays_on_generic_host_bootstrap_files() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");

    for relative in [
        "workbench.slint",
        "workbench/host_scaffold.slint",
        "workbench/host_surface.slint",
        "workbench/host_root.slint",
    ] {
        let source = std::fs::read_to_string(ui_root.join(relative)).expect("slint source");
        for forbidden in [
            "asset_panes.slint",
            "assets.slint",
            "panes.slint",
            "ui_asset_editor_pane.slint",
            "welcome.slint",
        ] {
            assert!(
                !source.contains(forbidden),
                "expected {relative} to stay on generic host bootstrap surfaces, not import {forbidden}"
            );
        }
    }

    let root_source = std::fs::read_to_string(ui_root.join("workbench.slint")).expect("slint source");
    for forbidden in [
        "export { WorkbenchHostContext } from \"workbench/host_context.slint\";",
        "export { PaneSurfaceHostContext } from \"workbench/pane_surface_host_context.slint\";",
        "export { PaneData, ProjectOverviewData, SceneNodeData } from \"workbench/pane_data.slint\";",
        "export { HostWorkbenchWindowSceneData } from \"workbench/host_scene.slint\";",
    ] {
        assert!(
            !root_source.contains(forbidden),
            "expected workbench.slint to stop re-exporting business host surfaces via `{forbidden}`"
        );
    }
}

#[test]
fn workbench_surface_components_own_pane_surface_seam_instead_of_host_components() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");

    for relative in [
        "workbench.slint",
        "workbench/host_scaffold.slint",
        "workbench/host_surface.slint",
        "workbench/host_root.slint",
        "workbench/host_components.slint",
    ] {
        let source = std::fs::read_to_string(ui_root.join(relative)).expect("slint source");
        for forbidden in [
            "pane_surface.slint",
            "animation_editor_pane.slint",
            "asset_panes.slint",
            "assets.slint",
            "panes.slint",
            "ui_asset_editor_pane.slint",
            "welcome.slint",
        ] {
            assert!(
                !source.contains(forbidden),
                "expected {relative} to stay out of pane/business slint ownership, not import {forbidden}"
            );
        }
    }

    let surface_owner =
        std::fs::read_to_string(ui_root.join("workbench/host_workbench_surfaces.slint"))
            .expect("workbench surface owner");
    for required in [
        "import { PaneSurface } from \"pane_surface.slint\";",
        "export component HostSideDockSurface inherits Rectangle {",
        "export component HostDocumentDockSurface inherits Rectangle {",
        "export component HostBottomDockSurface inherits Rectangle {",
        "export component HostFloatingWindowLayer inherits Rectangle {",
        "export component HostNativeFloatingWindowSurface inherits Rectangle {",
    ] {
        assert!(
            surface_owner.contains(required),
            "expected host_workbench_surfaces.slint to own the pane-surface-backed workbench surface components via `{required}`"
        );
    }
    for forbidden in ["PaneSurfaceHostContext", "pane_surface_host_context.slint"] {
        assert!(
            !surface_owner.contains(forbidden),
            "expected host_workbench_surfaces.slint to drop PaneSurfaceHostContext re-export plumbing `{forbidden}`"
        );
    }
}

#[test]
fn pane_surface_shell_extracts_business_pane_catalog_into_pane_content_owner() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_surface = std::fs::read_to_string(ui_root.join("workbench/pane_surface.slint"))
        .expect("pane surface");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");

    for forbidden in [
        "animation_editor_pane.slint",
        "asset_panes.slint",
        "assets.slint",
        "panes.slint",
        "welcome.slint",
    ] {
        assert!(
            !pane_surface.contains(forbidden),
            "expected pane_surface.slint to stop owning business pane imports, not import {forbidden}"
        );
    }

    for required in [
        "import { AnimationEditorPane } from \"animation_editor_pane.slint\";",
        "import { AssetBrowserPane, AssetsActivityPane, ProjectOverviewPane } from \"asset_panes.slint\";",
        "import { AssetFolderData, AssetItemData, AssetReferenceData, AssetSelectionData } from \"assets.slint\";",
        "import { ConsolePane, FallbackPane, HierarchyPane, InspectorPane, ToolWindowEmptyState } from \"panes.slint\";",
        "import { UiAssetEditorPane } from \"ui_asset_editor_pane.slint\";",
        "import { RecentProjectData, WelcomePane, WelcomePaneData } from \"welcome.slint\";",
        "export component PaneContent inherits Rectangle {",
    ] {
        assert!(
            pane_content.contains(required),
            "expected pane_content.slint to own business pane routing via `{required}`"
        );
    }

    let panes = std::fs::read_to_string(ui_root.join("workbench/panes.slint")).expect("panes");
    let ui_asset_editor =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_pane.slint"))
            .expect("ui asset editor pane");
    let ui_asset_editor_data =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_data.slint"))
            .expect("ui asset editor data");

    assert!(
        !panes.contains("export component UiAssetEditorPane inherits Rectangle {"),
        "expected panes.slint to drop UiAssetEditorPane ownership after the hard cutover"
    );
    assert!(
        !panes.contains("export struct UiAssetEditorPaneData {"),
        "expected panes.slint to drop UiAssetEditorPaneData ownership after the hard cutover"
    );
    assert!(ui_asset_editor.contains("export component UiAssetEditorPane inherits Rectangle {"));
    assert!(!ui_asset_editor.contains("export struct UiAssetEditorPaneData {"));
    assert!(ui_asset_editor_data.contains("export struct UiAssetEditorPaneData {"));
}

#[test]
fn ui_asset_editor_owner_extracts_data_helpers_and_panels_out_of_root_file() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let ui_asset_pane =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_pane.slint"))
            .expect("ui asset pane");
    let ui_asset_data =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_data.slint"))
            .expect("ui asset data");
    let ui_asset_components =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_components.slint"))
            .expect("ui asset components");
    let ui_asset_center =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_center_column.slint"))
            .expect("ui asset center column");
    let ui_asset_inspector =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_inspector_panel.slint"))
            .expect("ui asset inspector panel");
    let ui_asset_stylesheet =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_stylesheet_panel.slint"))
            .expect("ui asset stylesheet panel");

    assert!(
        pane_data.contains("import { UiAssetEditorPaneData } from \"ui_asset_editor_data.slint\";")
    );
    assert!(!pane_data
        .contains("import { UiAssetEditorPaneData } from \"ui_asset_editor_pane.slint\";"));

    for required in [
        "UiAssetEditorPaneData",
        "ui_asset_editor_data.slint",
        "import { UiAssetSelectableSection } from \"ui_asset_editor_components.slint\";",
        "import { UiAssetEditorCenterColumn } from \"ui_asset_editor_center_column.slint\";",
        "import { UiAssetEditorInspectorPanel } from \"ui_asset_editor_inspector_panel.slint\";",
        "import { UiAssetEditorStylesheetPanel } from \"ui_asset_editor_stylesheet_panel.slint\";",
        "export component UiAssetEditorPane inherits Rectangle {",
    ] {
        assert!(
            ui_asset_pane.contains(required),
            "expected ui_asset_editor_pane.slint to keep only root-pane orchestration via `{required}`"
        );
    }

    for forbidden in [
        "component UiAssetSelectableSection inherits Rectangle {",
        "component UiAssetCanvasSurface inherits Rectangle {",
        "component UiAssetSourceTextInput inherits TextInput {",
        "text: \"Designer Canvas\";",
        "text: \"Inspector\";",
        "text: \"Stylesheet\";",
    ] {
        assert!(
            !ui_asset_pane.contains(forbidden),
            "expected ui_asset_editor_pane.slint to drop extracted inline owner `{forbidden}`"
        );
    }

    for required in [
        "export struct UiAssetEditorPaneData {",
        "export struct UiAssetPreviewPanelData {",
        "export struct UiAssetInspectorPanelData {",
        "export struct UiAssetStylePanelData {",
    ] {
        assert!(
            ui_asset_data.contains(required),
            "expected ui_asset_editor_data.slint to own `{required}`"
        );
    }

    for required in [
        "component UiAssetSelectableSection inherits Rectangle {",
        "component UiAssetCanvasSurface inherits Rectangle {",
        "component UiAssetSourceTextInput inherits TextInput {",
    ] {
        assert!(
            ui_asset_components.contains(required),
            "expected ui_asset_editor_components.slint to own `{required}`"
        );
    }

    for required in [
        "export component UiAssetEditorCenterColumn inherits Rectangle {",
        "text: \"Designer Canvas\";",
        "text: \"Source\";",
    ] {
        assert!(
            ui_asset_center.contains(required),
            "expected ui_asset_editor_center_column.slint to own `{required}`"
        );
    }

    for required in [
        "export component UiAssetEditorInspectorPanel inherits Rectangle {",
        "text: \"Inspector\";",
    ] {
        assert!(
            ui_asset_inspector.contains(required),
            "expected ui_asset_editor_inspector_panel.slint to own `{required}`"
        );
    }

    for required in [
        "export component UiAssetEditorStylesheetPanel inherits Rectangle {",
        "text: \"Stylesheet\";",
    ] {
        assert!(
            ui_asset_stylesheet.contains(required),
            "expected ui_asset_editor_stylesheet_panel.slint to own `{required}`"
        );
    }
}

#[test]
fn editor_test_support_migrates_flat_ui_asset_documents_for_editor_consumers() {
    let document = crate::tests::support::load_test_ui_asset(FLAT_EDITOR_TEST_LAYOUT_ASSET_TOML)
        .expect("flat document");

    assert_eq!(document.asset.id, "editor.tests.flat_layout");
    let root = document.root.as_ref().expect("tree root");
    assert_eq!(root.node_id, "root");
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].node.node_id, "status");
    assert_eq!(
        root.children[0]
            .node
            .props
            .get("text")
            .and_then(|value| value.as_str()),
        Some("Ready")
    );
}

#[test]
fn editor_production_ui_modules_keep_flat_asset_migration_in_test_support_only() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let support_source =
        std::fs::read_to_string(crate_root.join("tests").join("support.rs")).expect("support");

    assert!(
        !support_source.contains("UiFlatAssetMigrationAdapter"),
        "editor test support should own its flat fixture migration locally instead of importing the runtime internal adapter"
    );
    assert!(
        support_source.contains("fn migrate_flat_ui_asset_toml_str(")
            && support_source.contains("struct FlatUiAssetDocument"),
        "editor test support should keep the local flat fixture migration helper"
    );

    for relative in ["ui", "core"] {
        let root = crate_root.join(relative);
        let offenders = collect_rust_files(&root)
            .into_iter()
            .filter_map(|path| {
                let source = std::fs::read_to_string(&path).ok()?;
                (source.contains("UiFlatAssetMigrationAdapter")
                    || source.contains("FlatUiAssetDocument")
                    || source.contains("migrate_flat_ui_asset_toml_str("))
                .then(|| {
                    path.strip_prefix(&crate_root)
                        .unwrap()
                        .display()
                        .to_string()
                })
            })
            .collect::<Vec<_>>();

        assert!(
            offenders.is_empty(),
            "production editor modules under src/{relative} should not carry flat asset migration helpers, found {:?}",
            offenders
        );
    }
}

#[test]
fn editor_template_runtime_production_path_stays_on_asset_documents_only() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_host = crate_root.join("src/ui/template_runtime/runtime/runtime_host.rs");
    let registry = crate_root.join("src/ui/template/registry.rs");
    let runtime_host_source = std::fs::read_to_string(runtime_host).expect("runtime host");
    let registry_source = std::fs::read_to_string(registry).expect("template registry");

    assert!(
        !runtime_host_source.contains("UiTemplateLoader"),
        "expected editor runtime host to stop importing UiTemplateLoader on the production document path"
    );
    assert!(
        !runtime_host_source.contains(".register_document("),
        "expected editor runtime host to stop registering legacy UiTemplateDocument sources"
    );
    assert!(
        !registry_source.contains("UiTemplateDocument")
            && !registry_source.contains("EditorTemplateSource::Template")
            && !registry_source.contains("pub fn register_document("),
        "expected editor template registry to keep compiled asset documents as the only production authority"
    );
}

#[test]
fn editor_builtin_template_files_migrate_to_asset_tree_authority() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_root = crate_root
        .join("assets")
        .join("ui")
        .join("editor")
        .join("host");
    let legacy_root = crate_root.join("ui").join("templates");
    let template_documents =
        crate_root.join("src/ui/template_runtime/builtin/template_documents.rs");
    let template_documents_source =
        std::fs::read_to_string(template_documents).expect("builtin template documents");

    assert!(
        !legacy_root.exists(),
        "expected legacy builtin template entry directory {:?} to be deleted after assets cutover",
        legacy_root
    );
    assert!(
        template_documents_source.contains("/assets/ui/editor/host/")
            && !template_documents_source.contains("/ui/templates/"),
        "expected builtin template documents to load entry assets from crate assets only"
    );

    for relative in [
        "asset_surface_controls.ui.toml",
        "floating_window_source.ui.toml",
        "inspector_surface_controls.ui.toml",
        "pane_surface_controls.ui.toml",
        "scene_viewport_toolbar.ui.toml",
        "startup_welcome_controls.ui.toml",
        "workbench_drawer_source.ui.toml",
        "workbench_shell.ui.toml",
    ] {
        let source =
            std::fs::read_to_string(template_root.join(relative)).expect("template source");
        assert!(
            source.contains("[asset]") && source.contains("node_id ="),
            "expected {relative} to be rewritten as tree asset authority"
        );
        for forbidden in [
            "version = 1\n\n[root]\ntemplate =",
            "\r\n[root]\r\ntemplate =",
        ] {
            assert!(
                !source.contains(forbidden),
                "expected {relative} to stop using legacy UiTemplateDocument root syntax"
            );
        }
    }
}

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

fn scoped_block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker `{marker}` in source"));
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

    panic!("missing closing brace for `{marker}` in source");
}

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
            "animation_editor_pane_view.slint",
            "asset_panes.slint",
            "assets.slint",
            "assets_activity_pane.slint",
            "console_pane_view.slint",
            "hierarchy_pane_view.slint",
            "inspector_pane_view.slint",
            "panes.slint",
            "project_overview_pane_view.slint",
            "ui_asset_editor_pane.slint",
            "welcome.slint",
        ] {
            assert!(
                !source.contains(forbidden),
                "expected {relative} to stay on generic host bootstrap surfaces, not import {forbidden}"
            );
        }
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
            "animation_editor_pane_view.slint",
            "asset_panes.slint",
            "assets.slint",
            "assets_activity_pane.slint",
            "console_pane_view.slint",
            "hierarchy_pane_view.slint",
            "inspector_pane_view.slint",
            "panes.slint",
            "project_overview_pane_view.slint",
            "ui_asset_editor_pane.slint",
            "welcome.slint",
        ] {
            assert!(
                !source.contains(forbidden),
                "expected {relative} to stay out of pane/business slint ownership, not import {forbidden}"
            );
        }
    }

    assert!(
        !ui_root
            .join("workbench/host_workbench_surfaces.slint")
            .exists(),
        "expected host_workbench_surfaces.slint to be deleted after surface owner split"
    );

    for (relative, required_component) in [
        (
            "workbench/host_side_dock_surface.slint",
            "export component HostSideDockSurface inherits Rectangle {",
        ),
        (
            "workbench/host_document_dock_surface.slint",
            "export component HostDocumentDockSurface inherits Rectangle {",
        ),
        (
            "workbench/host_bottom_dock_surface.slint",
            "export component HostBottomDockSurface inherits Rectangle {",
        ),
        (
            "workbench/host_floating_window_layer.slint",
            "export component HostFloatingWindowLayer inherits Rectangle {",
        ),
        (
            "workbench/host_native_floating_window_surface.slint",
            "export component HostNativeFloatingWindowSurface inherits Rectangle {",
        ),
    ] {
        let surface_owner = std::fs::read_to_string(ui_root.join(relative)).expect("surface owner");
        assert!(
            surface_owner.contains("import { PaneSurface } from \"pane_surface.slint\";"),
            "expected {relative} to own a pane-surface-backed workbench surface"
        );
        assert!(
            surface_owner.contains(required_component),
            "expected {relative} to own the workbench surface component via `{required_component}`"
        );
        for forbidden in ["PaneSurfaceHostContext", "pane_surface_host_context.slint"] {
            assert!(
                !surface_owner.contains(forbidden),
                "expected {relative} to drop PaneSurfaceHostContext re-export plumbing `{forbidden}`"
            );
        }
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
        "import { EmptyStateCard, Palette, ShellButton, ToolbarButton } from \"chrome.slint\";",
        "import { AssetFolderData, AssetItemData, AssetListView",
        "import { AnimationEditorPaneData, AssetBrowserPaneData, AssetsActivityPaneData, ConsolePaneData, HierarchyPaneData, InspectorPaneData, PaneData, ProjectOverviewData, ProjectOverviewPaneData } from \"pane_data.slint\";",
        "import { CompactField, AxisField } from \"pane_fields.slint\";",
        "import { FallbackPane } from \"fallback_pane.slint\";",
        "import { PaneSurfaceHostContext } from \"pane_surface_host_context.slint\";",
        "import { ToolWindowEmptyState } from \"tool_window_empty_state.slint\";",
        "import { UiAssetEditorPane } from \"ui_asset_editor_pane.slint\";",
        "import { WelcomePane } from \"welcome.slint\";",
        "export component PaneContent inherits Rectangle {",
    ] {
        assert!(
            pane_content.contains(required),
            "expected pane_content.slint to own business pane routing via `{required}`"
        );
    }

    for forbidden in [
        "import { AssetBrowserPane } from \"asset_panes.slint\";",
        "import { AssetsActivityPane } from \"assets_activity_pane.slint\";",
        "import { AnimationEditorPaneView } from \"animation_editor_pane_view.slint\";",
        "import { ConsolePaneView } from \"console_pane_view.slint\";",
        "import { HierarchyPaneView } from \"hierarchy_pane_view.slint\";",
        "import { InspectorPaneView } from \"inspector_pane_view.slint\";",
        "import { ProjectOverviewPaneView } from \"project_overview_pane_view.slint\";",
        "component AssetsActivityPane inherits Rectangle {",
        "component AssetBrowserPane inherits Rectangle {",
    ] {
        assert!(
            !pane_content.contains(forbidden),
            "expected pane_content.slint to delete split pane import/body `{forbidden}`"
        );
    }

    for required in [
        "component HierarchyPaneView inherits Rectangle {",
        "component InspectorPaneView inherits Rectangle {",
        "component ConsolePaneView inherits Rectangle {",
        "component AnimationEditorPaneView inherits Rectangle {",
        "component AssetsActivityPaneView inherits Rectangle {",
        "component AssetBrowserPaneView inherits Rectangle {",
        "import { TemplatePane } from \"template_pane.slint\";",
        "if !root.pane.show_empty && root.pane.kind == \"Project\": TemplatePane {",
        "if !root.pane.show_empty && root.pane.kind == \"Assets\": AssetsActivityPaneView {",
        "if !root.pane.show_empty && root.pane.kind == \"AssetBrowser\": AssetBrowserPaneView {",
        "if !root.pane.show_empty && root.pane.kind == \"Hierarchy\": HierarchyPaneView {",
        "if !root.pane.show_empty && root.pane.kind == \"Inspector\": InspectorPaneView {",
        "if root.pane.kind == \"Console\": ConsolePaneView {",
        "if !root.pane.show_empty && root.pane.kind == \"UiAssetEditor\": UiAssetEditorPane {",
        "if !root.pane.show_empty && (root.pane.kind == \"AnimationSequenceEditor\" || root.pane.kind == \"AnimationGraphEditor\"): AnimationEditorPaneView {",
    ] {
        assert!(
            pane_content.contains(required),
            "expected pane_content.slint to keep business route `{required}`"
        );
    }

    for removed in [
        "animation_editor_pane_view.slint",
        "asset_panes.slint",
        "assets_activity_pane.slint",
        "console_pane_view.slint",
        "hierarchy_pane_view.slint",
        "inspector_pane_view.slint",
        "project_overview_pane_view.slint",
    ] {
        assert!(
            !ui_root.join(format!("workbench/{removed}")).exists(),
            "expected split pane owner `{removed}` to be deleted after pane_content cutover"
        );
    }

    let ui_asset_editor =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_pane.slint"))
            .expect("ui asset editor pane");
    let ui_asset_editor_data =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_data.slint"))
            .expect("ui asset editor data");

    assert!(
        !ui_root.join("workbench/panes.slint").exists(),
        "expected panes.slint to be deleted after the remaining pane owners were split out"
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
    let ui_asset_palette_target_chooser = std::fs::read_to_string(
        ui_root.join("workbench/ui_asset_editor_palette_target_chooser.slint"),
    )
    .expect("ui asset palette target chooser");
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
        "import { UiAssetEditorPaletteTargetChooser } from \"ui_asset_editor_palette_target_chooser.slint\";",
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
        "title: \"Target Cycle\";",
        "Sticky target chooser  Tab/arrows cycle  Enter apply",
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
        "export component UiAssetEditorPaletteTargetChooser inherits Rectangle {",
        "title: \"Target Cycle\";",
        "label: \"Apply\";",
        "label: \"Cancel\";",
    ] {
        assert!(
            ui_asset_palette_target_chooser.contains(required),
            "expected ui_asset_editor_palette_target_chooser.slint to own `{required}`"
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
fn pane_data_routes_remaining_pane_data_through_dedicated_owner_files() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");

    for required in [
        "import { TemplateNodeFrameData, TemplatePaneNodeData } from \"template_node_data.slint\";",
        "export struct AnimationEditorPaneData {",
        "export struct InspectorPaneData {",
        "nodes: [TemplatePaneNodeData],",
        "export struct SceneNodeData {",
        "export struct HierarchyPaneData {",
        "export struct ConsolePaneData {",
        "hierarchy: HierarchyPaneData,",
        "inspector: InspectorPaneData,",
        "console: ConsolePaneData,",
        "animation: AnimationEditorPaneData,",
    ] {
        assert!(
            pane_data.contains(required),
            "expected pane_data.slint to own remaining pane DTOs via `{required}`"
        );
    }

    for forbidden in [
        "export struct TemplateNodeFrameData {",
        "export struct TemplatePaneNodeData {",
        "export struct AnimationEditorShellFrameData {",
        "export struct AnimationEditorShellLayoutData {",
        "animation_editor_pane.slint",
        "console_pane.slint",
        "hierarchy_pane.slint",
        "inspector_pane.slint",
    ] {
        assert!(
            !pane_data.contains(forbidden),
            "expected pane_data.slint to stop routing DTO ownership through `{forbidden}`"
        );
    }

    assert!(
        !pane_data.contains("from \"panes.slint\""),
        "expected pane_data.slint to drop the legacy panes.slint owner import after cutover"
    );
}

#[test]
fn ui_asset_editor_template_node_data_is_shared_between_pane_data_and_ui_asset_editor_data() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let ui_asset_data =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_data.slint"))
            .expect("ui asset data");
    let shared_data =
        std::fs::read_to_string(ui_root.join("workbench/template_node_data.slint"))
            .expect("shared template node data");
    let normalized = format!(
        "{}{}{}",
        pane_data.split_whitespace().collect::<String>(),
        ui_asset_data.split_whitespace().collect::<String>(),
        shared_data.split_whitespace().collect::<String>()
    );

    for required in [
        "import{TemplateNodeFrameData,TemplatePaneNodeData}from\"template_node_data.slint\";",
        "exportstructTemplateNodeFrameData{",
        "exportstructTemplatePaneNodeData{",
        "nodes:[TemplatePaneNodeData],",
    ] {
        assert!(
            normalized.contains(required),
            "expected shared template-node DTO contract to include `{required}`"
        );
    }

    for forbidden in [
        "exportstructUiAssetShellFrameData{",
        "exportstructUiAssetEditorShellLayoutData{",
        "shell_layout:UiAssetEditorShellLayoutData,",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected UiAssetEditor data contract to drop legacy shell-layout DTO `{forbidden}`"
        );
    }
}

#[test]
fn ui_asset_editor_root_pane_consumes_template_nodes_instead_of_shell_layout() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let ui_asset_pane =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_pane.slint"))
            .expect("ui asset pane");
    let ui_asset_data =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_data.slint"))
            .expect("ui asset data");
    let template_node_data =
        std::fs::read_to_string(ui_root.join("workbench/template_node_data.slint"))
            .expect("shared template node data");
    let normalized = format!(
        "{}{}{}",
        ui_asset_pane.split_whitespace().collect::<String>(),
        ui_asset_data.split_whitespace().collect::<String>(),
        template_node_data.split_whitespace().collect::<String>()
    );

    for required in [
        "import{TemplateNodeFrameData,TemplatePaneNodeData}from\"template_node_data.slint\";",
        "nodes:[TemplatePaneNodeData],",
        "root.pane.nodes",
        "HeaderPanel",
        "PalettePanel",
        "HierarchyPanel",
        "CenterColumn",
        "InspectorPanel",
        "StylesheetPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui asset editor cutover to consume template nodes via `{required}`"
        );
    }

    assert!(
        !normalized.contains("shell_layout"),
        "expected ui_asset_editor cutover to drop shell_layout references from the Slint/data path"
    );
    assert!(
        !normalized.contains(
            "HorizontalLayout{x:10px;y:74px;width:parent.width-20px;height:parent.height-84px;"
        ),
        "expected ui_asset_editor_pane.slint to drop the old hard-coded outer body geometry once template nodes are authoritative"
    );
    assert!(
        !normalized.contains("Rectangle{width:240px;height:parent.height;background:transparent;"),
        "expected ui_asset_editor_pane.slint to drop the old hard-coded left column width once template nodes are authoritative"
    );
    assert!(
        !normalized.contains("Rectangle{width:280px;height:parent.height;background:transparent;"),
        "expected ui_asset_editor_pane.slint to drop the old hard-coded right column width once template nodes are authoritative"
    );
}

#[test]
fn ui_asset_editor_root_pane_consumes_template_nodes_for_header_and_side_panels() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let ui_asset_pane =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_pane.slint"))
            .expect("ui asset pane");
    let normalized = ui_asset_pane.split_whitespace().collect::<String>();

    for required in [
        "HeaderAssetRow",
        "HeaderStatusRow",
        "HeaderActionRow",
        "PalettePanel",
        "HierarchyPanel",
        "InspectorPanel",
        "StylesheetPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui_asset_editor_pane.slint to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "height:parent.height*0.42;",
        "height:parent.height*0.58-8px;",
        "height:parent.height*0.5;",
        "Text{x:10px;y:6px;",
        "Text{x:10px;y:16px;",
        "HorizontalLayout{x:10px;y:28px;",
        "root.shell_layout.",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected ui_asset_editor_pane.slint to drop legacy shell/geometry dependency `{forbidden}`"
        );
    }
}

#[test]
fn ui_asset_editor_root_pane_extracts_palette_target_chooser_overlay_owner() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let ui_asset_pane =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_pane.slint"))
            .expect("ui asset pane");
    let chooser = std::fs::read_to_string(
        ui_root.join("workbench/ui_asset_editor_palette_target_chooser.slint"),
    )
    .expect("palette target chooser");
    let normalized = ui_asset_pane.split_whitespace().collect::<String>();

    for required in [
        "import{UiAssetEditorPaletteTargetChooser}from\"ui_asset_editor_palette_target_chooser.slint\";",
        "UiAssetEditorPaletteTargetChooser{",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui_asset_editor_pane.slint to delegate palette target chooser overlay via `{required}`"
        );
    }

    for forbidden in [
        "title:\"TargetCycle\";",
        "StickytargetchooserTab/arrowscycleEnterapply",
        "ShellButton{width:56px;label:\"Apply\";",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected ui_asset_editor_pane.slint to drop inline palette target chooser owner `{forbidden}`"
        );
    }

    for required in [
        "exportcomponentUiAssetEditorPaletteTargetChooserinheritsRectangle{",
        "title:\"TargetCycle\";",
        "ShellButton{width:56px;label:\"Apply\";",
    ] {
        assert!(
            chooser
                .split_whitespace()
                .collect::<String>()
                .contains(required),
            "expected ui_asset_editor_palette_target_chooser.slint to own `{required}`"
        );
    }
}

#[test]
fn ui_asset_editor_center_column_consumes_template_nodes_for_inner_panels() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let ui_asset_center =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_center_column.slint"))
            .expect("ui asset center column");
    let normalized = ui_asset_center.split_whitespace().collect::<String>();

    for required in [
        "inproperty<[TemplatePaneNodeData]>nodes;",
        "root.nodes",
        "DesignerPanel",
        "DesignerCanvasPanel",
        "RenderStackPanel",
        "ActionBarPanel",
        "ActionInsertRow",
        "ActionReparentRow",
        "ActionStructureRow",
        "SourcePanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui_asset_editor_center_column.slint to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "height:parent.height*0.38;",
        "height:54px;",
        "height:parent.height*0.62-62px;",
        "shell_layout",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected ui_asset_editor_center_column.slint to drop legacy percentage/fixed split `{forbidden}` once template nodes are authoritative"
        );
    }
}

#[test]
fn ui_asset_editor_center_column_consumes_template_nodes_for_action_and_source_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let ui_asset_center =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_center_column.slint"))
            .expect("ui asset center column");
    let normalized = ui_asset_center.split_whitespace().collect::<String>();

    for required in [
        "ActionInsertRow",
        "ActionReparentRow",
        "ActionStructureRow",
        "SourceInfoPanel",
        "SourceOutlinePanel",
        "MockWorkspacePanel",
        "MockSubjectsPanel",
        "MockEditorPanel",
        "MockStateGraphPanel",
        "SourceTextPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui_asset_editor_center_column.slint to consume inner template mount `{required}`"
        );
    }

    for forbidden in [
        "VerticalLayout{x:10px;y:8px;width:parent.width-20px;spacing:4px;",
        "y:28px;",
        "y:parent.height-90px;",
        "y:78px;",
        "y:94px;",
        "y:126px;",
        "y:138px;",
        "y:216px;",
        "y:294px;",
        "y:392px;",
        "y:408px;",
        "y:436px;",
        "y:466px;",
        "y:500px;",
        "shell_layout",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected ui_asset_editor_center_column.slint to drop legacy section geometry `{forbidden}`"
        );
    }
}

#[test]
fn ui_asset_editor_stylesheet_panel_consumes_template_nodes_for_header_and_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let stylesheet_panel =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_stylesheet_panel.slint"))
            .expect("ui asset stylesheet panel");
    let normalized = stylesheet_panel.split_whitespace().collect::<String>();

    for required in [
        "inproperty<[TemplatePaneNodeData]>nodes;",
        "root.nodes",
        "StylesheetActionRow",
        "StylesheetStatePrimaryRow",
        "StylesheetStateSecondaryRow",
        "StylesheetContentPanel",
        "StylesheetThemeSection",
        "StylesheetAuthoringSection",
        "StylesheetMatchedRuleSection",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui_asset_editor_stylesheet_panel.slint to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "y:26px;",
        "y:54px;",
        "y:82px;",
        "y:112px;",
        "y:118px;",
        "VerticalLayout{x:0px;y:0px;width:parent.width;spacing:4px;UiAssetSelectableSection{width:parent.width;height:104px;title:\"ThemeSources\";",
        "shell_layout",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected ui_asset_editor_stylesheet_panel.slint to drop legacy stylesheet section geometry `{forbidden}`"
        );
    }
}

#[test]
fn ui_asset_editor_inspector_panel_consumes_template_nodes_for_content_and_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let inspector_panel =
        std::fs::read_to_string(ui_root.join("workbench/ui_asset_editor_inspector_panel.slint"))
            .expect("ui asset inspector panel");
    let normalized = inspector_panel.split_whitespace().collect::<String>();

    for required in [
        "inproperty<[TemplatePaneNodeData]>nodes;",
        "root.nodes",
        "InspectorContentPanel",
        "InspectorWidgetSection",
        "InspectorPromoteSection",
        "InspectorSlotSection",
        "InspectorLayoutSection",
        "InspectorBindingSection",
    ] {
        assert!(
            normalized.contains(required),
            "expected ui_asset_editor_inspector_panel.slint to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "Rectangle{x:0px;y:26px;width:parent.width;height:parent.height-26px;clip:true;",
        "VerticalLayout{x:10px;y:8px;width:parent.width-20px;spacing:4px;",
        "VerticalLayout{x:0px;y:0px;width:parent.width;spacing:4px;Text{text:\"Widget\";",
        "Text{text:\"PromoteDraft\";color:palette.text;font-size:10px;font-weight:600;}",
        "Text{text:\"Slot\";color:palette.text;font-size:10px;font-weight:600;}",
        "Text{text:\"Layout\";color:palette.text;font-size:10px;font-weight:600;}",
        "Text{text:\"Bindings\";color:palette.text;font-size:10px;font-weight:600;}",
        "shell_layout",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected ui_asset_editor_inspector_panel.slint to drop legacy inspector section geometry `{forbidden}`"
        );
    }
}

#[test]
fn assets_activity_and_asset_browser_panes_route_through_extracted_owner_files() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");

    assert!(
        pane_content.contains("component AssetsActivityPaneView inherits Rectangle {"),
        "expected pane_content.slint to own AssetsActivityPaneView inline"
    );
    assert!(
        pane_content.contains("component AssetBrowserPaneView inherits Rectangle {"),
        "expected pane_content.slint to own AssetBrowserPaneView inline"
    );
    assert!(
        pane_content.contains(
            "if !root.pane.show_empty && root.pane.kind == \"Assets\": AssetsActivityPaneView {"
        ),
        "expected pane_content.slint to route AssetsActivityPaneView inline"
    );
    assert!(
        pane_content.contains(
            "if !root.pane.show_empty && root.pane.kind == \"AssetBrowser\": AssetBrowserPaneView {"
        ),
        "expected pane_content.slint to route AssetBrowserPaneView inline"
    );
    for removed in ["asset_panes.slint", "assets_activity_pane.slint"] {
        assert!(
            !ui_root.join(format!("workbench/{removed}")).exists(),
            "expected split asset pane owner `{removed}` to be deleted"
        );
    }
    assert!(
        !pane_content
            .contains("import { AssetsActivityPane } from \"assets_activity_pane.slint\";")
            && !pane_content.contains("import { AssetBrowserPane } from \"asset_panes.slint\";"),
        "expected pane_content.slint to drop split asset pane imports"
    );
}

#[test]
fn project_overview_pane_routes_through_extracted_owner_file() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");

    assert!(
        pane_content.contains("import { TemplatePane } from \"template_pane.slint\";"),
        "expected pane_content.slint to route project overview through the generic TemplatePane"
    );
    assert!(
        pane_content
            .contains("if !root.pane.show_empty && root.pane.kind == \"Project\": TemplatePane {"),
        "expected pane_content.slint to route project overview through TemplatePane"
    );
    assert!(
        !ui_root
            .join("workbench/project_overview_pane_view.slint")
            .exists(),
        "expected project_overview_pane_view.slint to be deleted"
    );
}

#[test]
fn project_overview_pane_consumes_shell_layout_for_top_level_sections() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor");
    let project_overview =
        std::fs::read_to_string(asset_root.join("project_overview.ui.toml")).expect("asset");
    let normalized = project_overview.split_whitespace().collect::<String>();

    for required in [
        "control_id=\"ProjectOverviewOuterPanel\"",
        "control_id=\"ProjectOverviewHeaderTitleRow\"",
        "control_id=\"ProjectOverviewHeaderPathRow\"",
        "control_id=\"ProjectOverviewDetailsPanel\"",
        "control_id=\"ProjectOverviewCatalogPanel\"",
        "control_id=\"ProjectOverviewDefaultSceneValue\"",
        "control_id=\"ProjectOverviewAssetsRootValue\"",
        "control_id=\"ProjectOverviewLibraryValue\"",
        "control_id=\"ProjectOverviewCatalogSummaryValue\"",
        "control_id=\"OpenAssetsView\"",
        "control_id=\"OpenAssetBrowser\"",
        "dispatch_kind=\"surface\"",
        "action_id=\"OpenView.editor.assets\"",
        "dispatch_kind=\"asset\"",
    ] {
        assert!(
            normalized.contains(required),
            "expected project_overview.ui.toml to own projected project overview authority `{required}`"
        );
    }

    for forbidden in [
        "componentProjectOverviewPaneViewinheritsRectangle{",
        "root.pane.shell_layout.outer_panel",
        "root.pane.shell_layout.header_title_row",
        "root.pane.shell_layout.catalog_panel",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected project overview authority to leave pane_content shell-layout body behind `{forbidden}`"
        );
    }
}

#[test]
fn welcome_pane_consumes_template_mount_nodes_for_top_level_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let welcome =
        std::fs::read_to_string(ui_root.join("workbench/welcome.slint")).expect("welcome pane");
    let normalized = welcome.split_whitespace().collect::<String>();

    for required in [
        "exportstructWelcomePaneData{",
        "nodes:[TemplatePaneNodeData],",
        "root.welcome.nodes",
        "WelcomeOuterPanel",
        "WelcomeRecentPanel",
        "WelcomeRecentHeaderPanel",
        "WelcomeRecentListPanel",
        "WelcomeMainPanel",
        "WelcomeHeroPanel",
        "WelcomeStatusPanel",
        "WelcomeNewProjectHeaderPanel",
        "WelcomeProjectNameField",
        "WelcomeLocationField",
        "WelcomePreviewPanel",
        "WelcomeValidationPanel",
        "WelcomeActionsRow",
    ] {
        assert!(
            normalized.contains(required),
            "expected welcome.slint to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "exportstructWelcomeShellFrameData{",
        "exportstructWelcomeShellLayoutData{",
        "root.welcome.shell_layout.outer_panel",
        "root.welcome.shell_layout.recent_panel",
        "root.welcome.shell_layout.recent_header_panel",
        "root.welcome.shell_layout.recent_list_panel",
        "root.welcome.shell_layout.main_panel",
        "root.welcome.shell_layout.hero_panel",
        "root.welcome.shell_layout.status_panel",
        "root.welcome.shell_layout.new_project_header_panel",
        "root.welcome.shell_layout.project_name_field",
        "root.welcome.shell_layout.location_field",
        "root.welcome.shell_layout.preview_panel",
        "root.welcome.shell_layout.validation_panel",
        "root.welcome.shell_layout.actions_row",
        "Rectangle{x:18px;y:18px;width:parent.width-36px;height:parent.height-36px;",
        "Rectangle{x:0px;y:0px;width:320px;height:parent.height;",
        "Rectangle{x:28px;y:28px;width:parent.width-56px;height:92px;",
        "Rectangle{x:28px;y:132px;width:parent.width-56px;height:34px;",
        "FieldCard{x:28px;y:258px;width:parent.width-56px;height:70px;",
        "FieldCard{x:28px;y:340px;width:parent.width-56px;height:70px;",
        "Rectangle{x:28px;y:424px;width:parent.width-56px;height:88px;",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected welcome.slint to drop legacy welcome shell authority `{forbidden}`"
        );
    }
}

#[test]
fn hierarchy_pane_consumes_template_mount_nodes_for_list_panel() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let hierarchy = scoped_block_after(
        &pane_content,
        "component HierarchyPaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        hierarchy.split_whitespace().collect::<String>()
    );

    for required in [
        "exportstructHierarchyPaneData{",
        "componentHierarchyPaneViewinheritsRectangle{",
        "inproperty<HierarchyPaneData>pane;",
        "root.pane.nodes",
        "HierarchyListPanel",
        "root.pane.hierarchy_nodes",
    ] {
        assert!(
            normalized.contains(required),
            "expected hierarchy_pane_view/pane_data hierarchy cutover to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "exportstructHierarchyShellFrameData{",
        "exportstructHierarchyShellLayoutData{",
        "root.pane.shell_layout.list_panel",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected hierarchy_pane_view/pane_data hierarchy cutover to drop shell-layout dependency `{forbidden}`"
        );
    }
}

#[test]
fn inspector_pane_cutover_consumes_template_mount_nodes_in_pane_data_and_content() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let inspector_pane_view = scoped_block_after(
        &pane_content,
        "component InspectorPaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        inspector_pane_view.split_whitespace().collect::<String>()
    );

    for required in [
        "exportstructInspectorPaneData{",
        "componentInspectorPaneViewinheritsRectangle{",
        "inproperty<InspectorPaneData>pane;",
        "root.pane.nodes",
        "InspectorContentPanel",
        "InspectorHeaderPanel",
        "InspectorNameRow",
        "InspectorParentRow",
        "InspectorPositionRow",
        "InspectorSeparatorRow",
        "InspectorActionsRow",
    ] {
        assert!(
            normalized.contains(required),
            "expected inspector cutover to consume template mount authority via pane_data/inspector_pane_view for `{required}`"
        );
    }

    for forbidden in [
        "exportstructInspectorShellFrameData{",
        "exportstructInspectorShellLayoutData{",
        "root.pane.shell_layout.content_panel",
        "root.pane.shell_layout.header_panel",
        "root.pane.shell_layout.name_row",
        "root.pane.shell_layout.parent_row",
        "root.pane.shell_layout.position_row",
        "root.pane.shell_layout.separator_row",
        "root.pane.shell_layout.actions_row",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected inspector cutover to drop inspector shell-layout dependency `{forbidden}`"
        );
    }

    assert!(
        !ui_root.join("workbench/inspector_pane.slint").exists(),
        "expected inspector_pane.slint to be deleted after the inspector cutover"
    );
    assert!(
        !ui_root.join("workbench/inspector_pane_view.slint").exists(),
        "expected inspector_pane_view.slint to stay deleted after the inline-owner cutover"
    );
}

#[test]
fn console_pane_consumes_template_mount_nodes_for_text_panel() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let console = scoped_block_after(
        &pane_content,
        "component ConsolePaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        console.split_whitespace().collect::<String>()
    );

    for required in [
        "exportstructConsolePaneData{",
        "componentConsolePaneViewinheritsRectangle{",
        "inproperty<ConsolePaneData>pane;",
        "root.pane.nodes",
        "ConsoleTextPanel",
        "root.pane.status_text",
    ] {
        assert!(
            normalized.contains(required),
            "expected console_pane_view/pane_data console cutover to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "root.pane.shell_layout.text_panel",
        "lines:=VerticalLayout{x:10px;y:8px-root.scroll_px*1px;width:parent.width-20px;spacing:6px;",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected console_pane_view/pane_data console cutover to drop shell-layout dependency `{forbidden}`"
        );
    }
}

#[test]
fn asset_browser_pane_consumes_template_mount_nodes_for_top_level_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let asset_browser = scoped_block_after(
        &pane_content,
        "component AssetBrowserPaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        asset_browser.split_whitespace().collect::<String>()
    );

    for required in [
        "exportstructAssetBrowserPaneData{",
        "componentAssetBrowserPaneViewinheritsRectangle{",
        "inproperty<AssetBrowserPaneData>pane;",
        "root.pane.nodes",
        "AssetBrowserToolbarPanel",
        "AssetBrowserImportPanel",
        "AssetBrowserMainPanel",
        "AssetBrowserSourcesPanel",
        "AssetBrowserContentPanel",
        "AssetBrowserDetailsPanel",
        "AssetBrowserUtilityPanel",
        "AssetBrowserUtilityTabsRow",
        "AssetBrowserUtilityContentPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected asset browser block to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "root.pane.shell_layout.toolbar_panel",
        "root.pane.shell_layout.import_panel",
        "root.pane.shell_layout.main_panel",
        "root.pane.shell_layout.sources_panel",
        "root.pane.shell_layout.content_panel",
        "root.pane.shell_layout.details_panel",
        "root.pane.shell_layout.utility_panel",
        "root.pane.shell_layout.utility_tabs_row",
        "root.pane.shell_layout.utility_content_panel",
        "privateproperty<length>outer_margin:10px;",
        "privateproperty<length>toolbar_height:110px;",
        "privateproperty<length>import_height:44px;",
        "privateproperty<length>sources_width:min(max(20%*root.width,220px),280px);",
        "privateproperty<length>details_width:min(max(30%*root.width,300px),360px);",
        "privateproperty<length>content_width:root.width-root.outer_margin*2-root.sources_width-root.details_width-root.section_gap*2;",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected asset browser block to drop legacy shell/geometry dependency `{forbidden}`"
        );
    }
}

#[test]
fn asset_browser_pane_consumes_template_mount_nodes_for_toolbar_and_utility_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let asset_browser = scoped_block_after(
        &pane_content,
        "component AssetBrowserPaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        asset_browser.split_whitespace().collect::<String>()
    );

    for required in [
        "AssetBrowserToolbarTitleRow",
        "AssetBrowserToolbarSearchRow",
        "AssetBrowserToolbarKindPrimaryRow",
        "AssetBrowserToolbarKindSecondaryRow",
        "AssetBrowserReferenceLeftPanel",
        "AssetBrowserReferenceRightPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected asset browser block to consume inner template mount `{required}`"
        );
    }

    for forbidden in [
        "root.pane.shell_layout.toolbar_title_row",
        "root.pane.shell_layout.toolbar_search_row",
        "root.pane.shell_layout.toolbar_kind_primary_row",
        "root.pane.shell_layout.toolbar_kind_secondary_row",
        "root.pane.shell_layout.reference_left_panel",
        "root.pane.shell_layout.reference_right_panel",
        "SearchField{x:12px;y:46px;width:236px;height:28px;",
        "KindChip{x:260px;y:48px;width:34px;label:\"All\";",
        "KindChip{x:260px;y:76px;width:56px;label:\"Physics\";",
        "ToolbarButton{x:parent.width-114px;y:48px;",
        "Text{x:parent.width-240px;y:15px;width:228px;",
        "ReferenceListView{x:0px;y:0px;width:(parent.width-12px)/2;",
        "Rectangle{x:12px;y:50px;width:parent.width-24px;height:parent.height-62px;background:transparent;",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected asset browser block to drop shell/absolute inner section geometry `{forbidden}`"
        );
    }
}

#[test]
fn assets_activity_pane_consumes_template_mount_nodes_for_top_level_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let assets_activity = scoped_block_after(
        &pane_content,
        "component AssetsActivityPaneView inherits Rectangle {",
    );
    let normalized = assets_activity.split_whitespace().collect::<String>();

    for required in [
        "componentAssetsActivityPaneViewinheritsRectangle{",
        "inproperty<AssetsActivityPaneData>pane;",
        "root.pane.nodes",
        "AssetsActivityToolbarPanel",
        "AssetsActivityMainPanel",
        "AssetsActivityTreePanel",
        "AssetsActivityContentPanel",
        "AssetsActivityUtilityPanel",
        "AssetsActivityUtilityTabsRow",
        "AssetsActivityUtilityContentPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected assets_activity_pane block to consume template mount `{required}`"
        );
    }

    for forbidden in [
        "root.pane.shell_layout.toolbar_panel",
        "root.pane.shell_layout.main_panel",
        "root.pane.shell_layout.tree_panel",
        "root.pane.shell_layout.content_panel",
        "root.pane.shell_layout.utility_panel",
        "root.pane.shell_layout.utility_tabs_row",
        "root.pane.shell_layout.utility_content_panel",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected assets_activity_pane block to drop legacy shell-layout dependency `{forbidden}`"
        );
    }
}

#[test]
fn assets_activity_pane_consumes_template_mount_nodes_for_toolbar_and_utility_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let assets_activity = scoped_block_after(
        &pane_content,
        "component AssetsActivityPaneView inherits Rectangle {",
    );
    let normalized = assets_activity.split_whitespace().collect::<String>();

    for required in [
        "AssetsActivityToolbarTitleRow",
        "AssetsActivityToolbarSearchRow",
        "AssetsActivityToolbarKindPrimaryRow",
        "AssetsActivityToolbarKindSecondaryRow",
        "AssetsActivityPreviewPanel",
        "AssetsActivityReferenceLeftPanel",
        "AssetsActivityReferenceRightPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected assets_activity_pane block to consume inner template mount `{required}`"
        );
    }

    for forbidden in [
        "root.pane.shell_layout.toolbar_title_row",
        "root.pane.shell_layout.toolbar_search_row",
        "root.pane.shell_layout.toolbar_kind_primary_row",
        "root.pane.shell_layout.toolbar_kind_secondary_row",
        "root.pane.shell_layout.preview_panel",
        "root.pane.shell_layout.reference_left_panel",
        "root.pane.shell_layout.reference_right_panel",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected assets_activity_pane block to drop inner shell-layout dependency `{forbidden}`"
        );
    }
}

#[test]
fn animation_editor_cutover_consumes_template_mount_nodes_for_top_level_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let animation_editor_pane_view = scoped_block_after(
        &pane_content,
        "component AnimationEditorPaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        animation_editor_pane_view
            .split_whitespace()
            .collect::<String>()
    );

    for required in [
        "exportstructAnimationEditorPaneData{",
        "componentAnimationEditorPaneViewinheritsRectangle{",
        "inproperty<AnimationEditorPaneData>pane;",
        "root.pane.nodes",
        "AnimationEditorHeaderPanel",
        "AnimationEditorBodyPanel",
        "AnimationSequenceContentPanel",
        "AnimationGraphContentPanel",
        "AnimationStateMachineContentPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected animation cutover to consume top-level template mount authority via pane_data/animation_editor_pane_view for `{required}`"
        );
    }

    for forbidden in [
        "exportstructAnimationEditorShellFrameData{",
        "exportstructAnimationEditorShellLayoutData{",
        "root.pane.shell_layout.header_panel",
        "root.pane.shell_layout.body_panel",
        "root.pane.shell_layout.sequence_content_panel",
        "root.pane.shell_layout.graph_content_panel",
        "root.pane.shell_layout.state_machine_content_panel",
        "height:64px;",
        "y:64px;",
        "width:parent.width-24px;",
        "height:parent.height-24px;",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected animation cutover to drop top-level shell/geometry authority `{forbidden}`"
        );
    }

    assert!(
        !ui_root
            .join("workbench/animation_editor_pane.slint")
            .exists(),
        "expected animation_editor_pane.slint to be deleted after the animation cutover"
    );
    assert!(
        !ui_root
            .join("workbench/animation_editor_pane_view.slint")
            .exists(),
        "expected animation_editor_pane_view.slint to stay deleted after the inline-owner cutover"
    );
}

#[test]
fn animation_editor_cutover_consumes_template_mount_nodes_for_mode_inner_sections() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let pane_data =
        std::fs::read_to_string(ui_root.join("workbench/pane_data.slint")).expect("pane data");
    let pane_content = std::fs::read_to_string(ui_root.join("workbench/pane_content.slint"))
        .expect("pane content");
    let animation_editor_pane_view = scoped_block_after(
        &pane_content,
        "component AnimationEditorPaneView inherits Rectangle {",
    );
    let normalized = format!(
        "{}{}",
        pane_data.split_whitespace().collect::<String>(),
        animation_editor_pane_view
            .split_whitespace()
            .collect::<String>()
    );

    for required in [
        "AnimationEditorHeaderModeRow",
        "AnimationEditorHeaderPathRow",
        "AnimationEditorHeaderStatusRow",
        "AnimationSequenceTimelineRow",
        "AnimationSequenceSelectionRow",
        "AnimationSequenceTracksPanel",
        "AnimationGraphParametersPanel",
        "AnimationGraphNodesPanel",
        "AnimationStateMachineEntryRow",
        "AnimationStateMachineStatesPanel",
        "AnimationStateMachineTransitionsPanel",
    ] {
        assert!(
            normalized.contains(required),
            "expected animation cutover to consume inner template mount authority via pane_data/animation_editor_pane_view for `{required}`"
        );
    }

    for forbidden in [
        "root.pane.shell_layout.header_mode_row",
        "root.pane.shell_layout.header_path_row",
        "root.pane.shell_layout.header_status_row",
        "root.pane.shell_layout.sequence_timeline_row",
        "root.pane.shell_layout.sequence_selection_row",
        "root.pane.shell_layout.sequence_tracks_panel",
        "root.pane.shell_layout.graph_parameters_panel",
        "root.pane.shell_layout.graph_nodes_panel",
        "root.pane.shell_layout.state_machine_entry_row",
        "root.pane.shell_layout.state_machine_states_panel",
        "root.pane.shell_layout.state_machine_transitions_panel",
        "x:12px;y:10px;",
        "x:12px;y:26px;",
        "x:12px;y:44px;",
        "y:18px;",
        "y:44px;",
        "y:140px;",
        "y:148px;",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "expected animation cutover to drop legacy animation shell offset `{forbidden}`"
        );
    }
    assert!(
        !ui_root
            .join("workbench/animation_editor_pane_view.slint")
            .exists(),
        "expected animation_editor_pane_view.slint to stay deleted after the inline-owner cutover"
    );
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
    let build_session = crate_root.join("src/ui/template_runtime/runtime/build_session.rs");
    let template_documents_source =
        std::fs::read_to_string(template_documents).expect("builtin template documents");
    let build_session_source = std::fs::read_to_string(build_session).expect("build session");

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
    assert!(
        !template_documents_source.contains("include_str!")
            && !template_documents_source.contains("macro_rules! builtin_host_template"),
        "expected builtin template documents to stop embedding production ui.toml assets into Rust source"
    );
    assert!(
        build_session_source.contains("runtime.register_document_file(document_id, path)?;")
            && !build_session_source
                .contains("runtime.register_document_source(document_id, template)?;"),
        "expected builtin host templates to register from asset file paths instead of embedded source strings"
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

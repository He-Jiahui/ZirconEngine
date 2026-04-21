use std::fs;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImporter, AssetKind, AssetUri, ImportedAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};

const LAYOUT_UI_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 1
display_name = "UI Asset Editor"

[imports]
widgets = ["res://ui/common/button.ui.toml#ToolbarButton"]
styles = ["res://ui/theme/editor.ui.toml"]

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
classes = []
bindings = []
children = []

[root.params]

[root.props]

[root.style_overrides.self]

[root.style_overrides.slot]

[components]
"#;

const WIDGET_UI_TOML: &str = r#"
[asset]
kind = "widget"
id = "ui.common.button"
version = 1
display_name = "Toolbar Button"

[root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = []
bindings = []
children = []

[root.params]

[root.props]

[root.style_overrides.self]

[root.style_overrides.slot]

[components.ToolbarButton]
style_scope = "closed"

[components.ToolbarButton.root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = []
bindings = []
children = []

[components.ToolbarButton.root.params]

[components.ToolbarButton.root.props]

[components.ToolbarButton.root.style_overrides.self]

[components.ToolbarButton.root.style_overrides.slot]

[components.ToolbarButton.params]

[components.ToolbarButton.slots]
"#;

const STYLE_UI_TOML: &str = r#"
[asset]
kind = "style"
id = "ui.theme.editor"
version = 1
display_name = "Editor Theme"

[imports]
widgets = []
styles = []

[tokens]

[components]

[[stylesheets]]
id = "editor"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Styled" } }
"#;

#[test]
fn ui_asset_wrappers_parse_and_validate_kind() {
    let layout = UiLayoutAsset::from_toml_str(LAYOUT_UI_TOML).unwrap();
    let widget = UiWidgetAsset::from_toml_str(WIDGET_UI_TOML).unwrap();
    let style = UiStyleAsset::from_toml_str(STYLE_UI_TOML).unwrap();

    assert_eq!(
        layout.document.asset.kind,
        crate::ui::template::UiAssetKind::Layout
    );
    assert_eq!(
        widget.document.asset.kind,
        crate::ui::template::UiAssetKind::Widget
    );
    assert_eq!(
        style.document.asset.kind,
        crate::ui::template::UiAssetKind::Style
    );
    assert!(UiLayoutAsset::from_toml_str(WIDGET_UI_TOML).is_err());
}

#[test]
fn importer_decodes_ui_layout_widget_and_style_assets_from_ui_toml() {
    let root = unique_temp_project_root("ui_asset_import");
    fs::create_dir_all(&root).unwrap();
    let layout_path = root.join("panel.ui.toml");
    let widget_path = root.join("button.ui.toml");
    let style_path = root.join("theme.ui.toml");
    fs::write(&layout_path, LAYOUT_UI_TOML).unwrap();
    fs::write(&widget_path, WIDGET_UI_TOML).unwrap();
    fs::write(&style_path, STYLE_UI_TOML).unwrap();

    let importer = AssetImporter::default();

    let layout = importer
        .import_from_source(
            &layout_path,
            &AssetUri::parse("res://ui/panel.ui.toml").unwrap(),
        )
        .unwrap();
    let widget = importer
        .import_from_source(
            &widget_path,
            &AssetUri::parse("res://ui/button.ui.toml").unwrap(),
        )
        .unwrap();
    let style = importer
        .import_from_source(
            &style_path,
            &AssetUri::parse("res://ui/theme.ui.toml").unwrap(),
        )
        .unwrap();

    match layout {
        ImportedAsset::UiLayout(asset) => {
            assert_eq!(asset.document.asset.id, "editor.ui_asset_editor");
        }
        other => panic!("unexpected layout import: {other:?}"),
    }
    match widget {
        ImportedAsset::UiWidget(asset) => {
            assert_eq!(asset.document.asset.display_name, "Toolbar Button");
        }
        other => panic!("unexpected widget import: {other:?}"),
    }
    match style {
        ImportedAsset::UiStyle(asset) => {
            assert_eq!(asset.document.stylesheets.len(), 1);
        }
        other => panic!("unexpected style import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_ui_assets_and_assigns_ui_asset_kinds() {
    let root = unique_temp_project_root("ui_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "UiSandbox",
        AssetUri::parse("res://ui/panel.ui.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let ui_dir = paths.assets_root().join("ui");
    fs::create_dir_all(&ui_dir).unwrap();
    fs::write(ui_dir.join("panel.ui.toml"), LAYOUT_UI_TOML).unwrap();
    fs::write(ui_dir.join("button.ui.toml"), WIDGET_UI_TOML).unwrap();
    fs::write(ui_dir.join("theme.ui.toml"), STYLE_UI_TOML).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let layout = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/panel.ui.toml").unwrap())
        .unwrap();
    let widget = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/button.ui.toml").unwrap())
        .unwrap();
    let style = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/theme.ui.toml").unwrap())
        .unwrap();

    assert_eq!(layout.kind, AssetKind::UiLayout);
    assert_eq!(widget.kind, AssetKind::UiWidget);
    assert_eq!(style.kind, AssetKind::UiStyle);

    match manager
        .load_artifact(&AssetUri::parse("res://ui/panel.ui.toml").unwrap())
        .unwrap()
    {
        ImportedAsset::UiLayout(asset) => {
            assert_eq!(asset.document.asset.id, "editor.ui_asset_editor");
        }
        other => panic!("unexpected project layout asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

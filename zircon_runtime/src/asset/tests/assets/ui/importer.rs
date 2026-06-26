use super::*;

#[test]
fn importer_decodes_ui_theme_assets_from_theme_toml() {
    let root = unique_temp_project_root("ui_theme_asset_import");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("editor.theme.toml");
    fs::write(&path, THEME_UI_TOML).unwrap();

    let theme = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://ui/theme/editor.theme.toml").unwrap(),
        )
        .unwrap();

    match theme {
        ImportedAsset::UiTheme(asset) => {
            assert_eq!(asset.document.id, "zircon.test.dark");
            assert_eq!(
                asset.document.palette.accent,
                UiRgbaColor::new(0.1, 0.2, 0.3, 1.0)
            );
        }
        other => panic!("unexpected theme import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
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

    let importer = importer_with_first_wave_plugin_fixtures();

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
fn importer_decodes_ui_icon_assets_from_icon_toml() {
    let root = unique_temp_project_root("ui_icon_asset_import");
    fs::create_dir_all(&root).unwrap();
    let icon_path = root.join("run.icon.toml");
    fs::write(&icon_path, ICON_UI_TOML).unwrap();

    let importer = AssetImporter::default();

    let icon = importer
        .import_from_source(
            &icon_path,
            &AssetUri::parse("res://ui/icons/run.icon.toml").unwrap(),
        )
        .unwrap();

    match icon {
        ImportedAsset::UiIcon(asset) => {
            assert_eq!(asset.semantic_id, "icons/run");
            assert_eq!(asset.direct_references().len(), 1);
        }
        other => panic!("unexpected icon import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_decodes_zui_component_assets_from_zui() {
    let root = unique_temp_project_root("zui_asset_import");
    fs::create_dir_all(&root).unwrap();
    let component_path = root.join("button.zui");
    fs::write(&component_path, V2_COMPONENT_UI_TOML).unwrap();

    let importer = AssetImporter::default();

    let component = importer
        .import_from_source(
            &component_path,
            &AssetUri::parse("res://ui/button.zui").unwrap(),
        )
        .unwrap();

    match component {
        ImportedAsset::UiV2Component(asset) => {
            assert!(asset.document.components.contains_key("ToolbarButton"));
        }
        other => panic!("unexpected zui component import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

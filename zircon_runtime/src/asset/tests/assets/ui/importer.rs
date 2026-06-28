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
fn importer_preserves_typed_theme_and_icon_document_sources() {
    let root = unique_temp_project_root("ui_theme_icon_import_typed_errors");
    fs::create_dir_all(&root).unwrap();
    let theme_path = root.join("broken.theme.toml");
    let icon_path = root.join("broken.icon.toml");
    fs::write(&theme_path, INVALID_UI_TOML).unwrap();
    fs::write(
        &icon_path,
        r##"
semantic_id = "icons/run"
default_size = 16.0

[source]
kind = "bitmap"
uri = "res://"
"##,
    )
    .unwrap();

    let importer = AssetImporter::default();
    let theme_error = importer
        .import_from_source(
            &theme_path,
            &AssetUri::parse("res://ui/theme/broken.theme.toml").unwrap(),
        )
        .unwrap_err();
    let icon_error = importer
        .import_from_source(
            &icon_path,
            &AssetUri::parse("res://ui/icons/broken.icon.toml").unwrap(),
        )
        .unwrap_err();

    assert!(matches!(
        theme_error,
        AssetImportError::UiThemeDocument {
            source: UiThemeAssetDocumentError::Parse(_),
            ..
        }
    ));
    assert!(matches!(
        icon_error,
        AssetImportError::UiIconDocument {
            source: UiIconAssetDocumentError::InvalidSourceUri { .. },
            ..
        }
    ));

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

#[test]
fn importer_decodes_zui_view_and_style_assets_from_zui() {
    let root = unique_temp_project_root("zui_view_style_asset_import");
    fs::create_dir_all(&root).unwrap();
    let view_path = root.join("panel.zui");
    let style_path = root.join("theme.zui");
    fs::write(&view_path, V2_VIEW_UI_TOML).unwrap();
    fs::write(&style_path, V2_STYLE_UI_TOML).unwrap();

    let importer = AssetImporter::default();

    let view = importer
        .import_from_source(&view_path, &AssetUri::parse("res://ui/panel.zui").unwrap())
        .unwrap();
    let style = importer
        .import_from_source(&style_path, &AssetUri::parse("res://ui/theme.zui").unwrap())
        .unwrap();

    match view {
        ImportedAsset::UiV2View(asset) => {
            assert_eq!(asset.document.asset.id, "runtime.ui.panel");
            assert!(asset.document.root.is_some());
        }
        other => panic!("unexpected zui view import: {other:?}"),
    }
    match style {
        ImportedAsset::UiV2Style(asset) => {
            assert_eq!(asset.document.asset.id, "runtime.ui.material");
            assert_eq!(asset.document.stylesheets.len(), 1);
        }
        other => panic!("unexpected zui style import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

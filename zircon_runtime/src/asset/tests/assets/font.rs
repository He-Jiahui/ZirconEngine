use std::fs;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{AssetImporter, AssetKind, AssetUri, FontAsset, ImportedAsset};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

const FONT_TOML: &str = r#"
source = "FiraMono-subset.ttf"
family = "Fira Mono"
render_mode = "sdf"
"#;

#[test]
fn font_asset_wrapper_parses_runtime_font_manifest_fields() {
    let font = FontAsset::from_toml_str(FONT_TOML).unwrap();

    assert_eq!(font.source, "FiraMono-subset.ttf");
    assert_eq!(font.family.as_deref(), Some("Fira Mono"));
    assert_eq!(font.render_mode, Some(UiTextRenderMode::Sdf));
}

#[test]
fn importer_decodes_font_assets_from_font_toml() {
    let root = unique_temp_project_root("font_asset_import");
    fs::create_dir_all(&root).unwrap();
    let font_path = root.join("default.font.toml");
    fs::write(&font_path, FONT_TOML).unwrap();

    let importer = AssetImporter::default();
    let imported = importer
        .import_from_source(
            &font_path,
            &AssetUri::parse("res://fonts/default.font.toml").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Font(asset) => {
            assert_eq!(asset.source, "FiraMono-subset.ttf");
            assert_eq!(asset.render_mode, Some(UiTextRenderMode::Sdf));
        }
        other => panic!("unexpected font import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_font_assets_and_assigns_font_asset_kind() {
    let root = unique_temp_project_root("font_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "FontSandbox",
        AssetUri::parse("res://fonts/default.font.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let font_dir = paths.assets_root().join("fonts");
    fs::create_dir_all(&font_dir).unwrap();
    fs::write(font_dir.join("default.font.toml"), FONT_TOML).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(imported.len(), 1);
    let record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://fonts/default.font.toml").unwrap())
        .unwrap();
    assert_eq!(record.kind, AssetKind::Font);

    match manager
        .load_artifact(&AssetUri::parse("res://fonts/default.font.toml").unwrap())
        .unwrap()
    {
        ImportedAsset::Font(asset) => {
            assert_eq!(asset.family.as_deref(), Some("Fira Mono"));
        }
        other => panic!("unexpected project font asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

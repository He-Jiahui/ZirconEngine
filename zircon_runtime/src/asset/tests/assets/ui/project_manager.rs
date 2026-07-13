use super::*;

#[test]
fn project_manager_scans_ui_theme_assets_and_restores_theme_payloads() {
    let root = unique_temp_project_root("ui_theme_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "UiThemeSandbox",
        AssetUri::parse("res://ui/theme/editor.theme.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let theme_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("ui")
        .join("theme");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("editor.theme.toml"), THEME_UI_TOML).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let theme = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/theme/editor.theme.toml").unwrap())
        .unwrap();

    assert_eq!(theme.kind, AssetKind::UiStyle);

    match manager
        .load_artifact(&AssetUri::parse("res://ui/theme/editor.theme.toml").unwrap())
        .unwrap()
    {
        ImportedAsset::UiTheme(asset) => {
            assert_eq!(asset.document.id, "zircon.test.dark");
            assert_eq!(
                asset.document.palette.surface[0],
                UiRgbaColor::from_u8(17, 20, 22, 255)
            );
        }
        other => panic!("unexpected project theme asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_ui_icon_assets_and_restores_icon_payloads() {
    let root = unique_temp_project_root("ui_icon_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "UiIconSandbox",
        AssetUri::parse("res://ui/icons/run.icon.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let icon_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("ui")
        .join("icons");
    fs::create_dir_all(&icon_dir).unwrap();
    fs::write(icon_dir.join("run.icon.toml"), ICON_UI_TOML).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let icon = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/icons/run.icon.toml").unwrap())
        .unwrap();

    assert_eq!(icon.kind, AssetKind::Texture);

    match manager
        .load_artifact(&AssetUri::parse("res://ui/icons/run.icon.toml").unwrap())
        .unwrap()
    {
        ImportedAsset::UiIcon(asset) => {
            assert_eq!(asset.semantic_id, "icons/run");
            assert_eq!(
                asset
                    .direct_references()
                    .iter()
                    .map(|reference| reference.locator.to_string())
                    .collect::<Vec<_>>(),
                vec!["res://ui/icons/run.svg"]
            );
        }
        other => panic!("unexpected project icon asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_ui_assets_and_assigns_ui_asset_kinds() {
    let root = unique_temp_project_root("ui_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "UiSandbox",
        AssetUri::parse("res://ui/panel.zui").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let ui_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("ui");
    fs::create_dir_all(&ui_dir).unwrap();
    fs::write(ui_dir.join("panel.zui"), V2_VIEW_UI_TOML).unwrap();
    fs::write(ui_dir.join("button.zui"), V2_COMPONENT_UI_TOML).unwrap();
    fs::write(ui_dir.join("theme.zui"), V2_STYLE_UI_TOML).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager.scan_and_import().unwrap();

    let layout = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/panel.zui").unwrap())
        .unwrap();
    let widget = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/button.zui").unwrap())
        .unwrap();
    let style = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/theme.zui").unwrap())
        .unwrap();

    assert_eq!(layout.kind, AssetKind::UiLayout);
    assert_eq!(widget.kind, AssetKind::UiWidget);
    assert_eq!(style.kind, AssetKind::UiStyle);

    match manager
        .load_artifact(&AssetUri::parse("res://ui/panel.zui").unwrap())
        .unwrap()
    {
        ImportedAsset::UiV2View(asset) => {
            assert_eq!(asset.document.asset.id, "runtime.ui.panel");
        }
        other => panic!("unexpected project layout asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_zui_assets_and_restores_component_payloads() {
    let root = unique_temp_project_root("zui_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "ZuiSandbox",
        AssetUri::parse("res://ui/button.zui").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let ui_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("ui");
    fs::create_dir_all(&ui_dir).unwrap();
    fs::write(ui_dir.join("button.zui"), V2_COMPONENT_UI_TOML).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager.scan_and_import().unwrap();

    let component = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://ui/button.zui").unwrap())
        .unwrap();

    assert_eq!(component.kind, AssetKind::UiWidget);

    match manager
        .load_artifact(&AssetUri::parse("res://ui/button.zui").unwrap())
        .unwrap()
    {
        ImportedAsset::UiV2Component(asset) => {
            assert!(asset.document.components.contains_key("ToolbarButton"));
        }
        other => panic!("unexpected project zui component asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

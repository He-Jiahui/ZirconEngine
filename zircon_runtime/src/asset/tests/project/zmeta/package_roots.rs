use super::*;

#[test]
fn project_manager_scans_package_asset_roots_as_package_uris() {
    let root = unique_temp_project_root("project_manager_package_zmeta");
    let package_root = unique_temp_project_root("navigation_package_zmeta");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "PackageSandbox",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let project_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data")
        .join("project.json");
    fs::create_dir_all(project_path.parent().unwrap()).unwrap();
    fs::write(&project_path, r#"{ "project": true }"#).unwrap();

    let package_asset_path = package_root.join("assets").join("nav").join("agent.json");
    fs::create_dir_all(package_asset_path.parent().unwrap()).unwrap();
    fs::write(&package_asset_path, r#"{ "agent": true }"#).unwrap();

    let package_manifest = PluginPackageManifest::new("navigation", "Navigation")
        .with_package_identity("com", "zircon", "navigation");
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_package_asset_roots(
            package_manifest.package_id(),
            package_manifest.asset_roots_or_default(),
            &package_root,
        )
        .unwrap();
    manager.scan_and_import().unwrap();

    let package_uri = AssetUri::parse("package://com.zircon.navigation/nav/agent.json").unwrap();
    let package_record = manager
        .registry()
        .get_by_locator(&package_uri)
        .expect("package asset record");
    let package_meta_path = package_root
        .join("assets")
        .join("nav")
        .join("agent.json.zmeta");
    let package_meta = AssetMetaDocument::load(&package_meta_path).unwrap();

    assert_eq!(package_manifest.package_id(), "com.zircon.navigation");
    assert_eq!(
        package_manifest.asset_roots_or_default(),
        vec!["assets".to_string()]
    );
    assert_eq!(
        manager.source_path_for_uri(&package_uri).unwrap(),
        package_asset_path
    );
    assert_eq!(package_meta.url, package_uri);
    assert_eq!(package_meta.asset_kind, AssetKind::Data);
    assert_eq!(
        package_record.id(),
        AssetId::from_asset_uuid(package_meta.uuid)
    );
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://data/project.json").unwrap())
        .is_some());

    let error = manager
        .source_path_for_uri(
            &AssetUri::parse("package://com.zircon.missing/nav/agent.json").unwrap(),
        )
        .expect_err("unknown package should be rejected");
    assert!(error
        .to_string()
        .contains("unknown package com.zircon.missing"));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

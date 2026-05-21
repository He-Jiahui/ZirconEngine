use std::fs;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetMetaDocument, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter,
    ImportedAsset, ImportedAssetEntry, ProjectManager, ProjectManifest, ProjectPaths,
};
use crate::core::resource::ResourceState;
use crate::plugin::PluginPackageManifest;

#[test]
fn project_manager_registers_direct_package_asset_root() {
    let root = unique_temp_project_root("project_manager_direct_package_root");
    let package_assets_root = unique_temp_project_root("direct_navigation_assets");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "DirectPackageSandbox",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let package_asset_path = package_assets_root.join("nav").join("agent.json");
    fs::create_dir_all(package_asset_path.parent().unwrap()).unwrap();
    fs::write(&package_asset_path, r#"{ "agent": true }"#).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_package_asset_root("com.zircon.navigation", &package_assets_root)
        .unwrap();
    manager.scan_and_import().unwrap();

    let package_uri = AssetUri::parse("package://com.zircon.navigation/nav/agent.json").unwrap();
    let record = manager
        .registry()
        .get_by_locator(&package_uri)
        .expect("package asset record");
    let meta = AssetMetaDocument::load(package_asset_path.with_file_name("agent.json.zmeta"))
        .expect("package zmeta");

    assert_eq!(
        manager.source_path_for_uri(&package_uri).unwrap(),
        package_asset_path
    );
    assert_eq!(meta.url, package_uri);
    assert_eq!(record.state, ResourceState::Ready);
    assert_eq!(record.id(), AssetId::from_asset_uuid(meta.uuid));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_assets_root);
}

#[test]
fn project_manager_imports_package_labeled_subassets_with_package_urls() {
    let root = unique_temp_project_root("project_manager_package_multi_asset");
    let package_root = unique_temp_project_root("package_multi_asset_root");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "PackageMultiSandbox",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = package_root
        .join("assets")
        .join("bundles")
        .join("atlas.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let package_manifest = PluginPackageManifest::new("navigation", "Navigation")
        .with_package_identity("com", "zircon", "navigation");
    let root_uri = AssetUri::parse("package://com.zircon.navigation/bundles/atlas.multi").unwrap();
    let texture_uri =
        AssetUri::parse("package://com.zircon.navigation/bundles/atlas.multi#Texture0").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_package_manifest_asset_roots(&package_manifest, &package_root)
        .unwrap();
    manager
        .register_asset_importer(multi_asset_importer())
        .unwrap();
    manager.scan_and_import().unwrap();

    let meta_path = source_path.with_file_name("atlas.multi.zmeta");
    let meta = AssetMetaDocument::load(&meta_path).unwrap();
    let root_record = manager
        .registry()
        .get_by_locator(&root_uri)
        .expect("package root record");
    let texture_record = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("package subasset record");
    let texture_entry = meta
        .entries
        .iter()
        .find(|entry| entry.url == texture_uri)
        .expect("package texture entry");

    assert_eq!(meta.url, root_uri);
    assert!(meta
        .entries
        .iter()
        .any(|entry| entry.uuid == meta.uuid && entry.url == root_uri));
    assert_eq!(
        texture_record.id(),
        AssetId::from_asset_uuid(texture_entry.uuid)
    );
    assert_ne!(root_record.id(), texture_record.id());
    assert_eq!(
        manager.asset_id_for_uri(&texture_uri),
        Some(texture_record.id())
    );
    assert_eq!(
        manager.asset_id_for_reference(texture_entry.uuid, &texture_uri),
        Some(texture_record.id())
    );

    match manager.load_artifact(&texture_uri).unwrap() {
        ImportedAsset::Texture(texture) => assert_eq!(texture.rgba, vec![255, 0, 255, 255]),
        other => panic!("unexpected package subasset artifact: {other:?}"),
    }
    match manager
        .load_artifact(
            &AssetUri::parse("package://com.zircon.navigation/bundles/atlas.multi#Missing")
                .unwrap(),
        )
        .expect_err("missing package label should be structured")
    {
        AssetImportError::MissingAssetLabel { source_uri, label } => {
            assert_eq!(source_uri, root_uri);
            assert_eq!(label, "Missing");
        }
        other => panic!("unexpected missing package label error: {other:?}"),
    }

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted
        .register_package_manifest_asset_roots(&package_manifest, &package_root)
        .unwrap();
    restarted.scan_and_import().unwrap();

    let restored_texture = restarted
        .registry()
        .get_by_locator(&texture_uri)
        .expect("restored package subasset record");
    assert_eq!(restored_texture.id(), texture_record.id());
    assert_eq!(
        restarted.asset_id_for_reference(texture_entry.uuid, &texture_uri),
        Some(texture_record.id())
    );

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn package_asset_registry_rejects_invalid_manifest_roots() {
    let root = unique_temp_project_root("project_manager_invalid_package_roots");
    let package_root = unique_temp_project_root("invalid_package_root");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "InvalidPackageRootSandbox",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    let parent_root = PluginPackageManifest::new("navigation", "Navigation")
        .with_package_identity("com", "zircon", "navigation")
        .with_asset_root("../outside");
    let parent_error = manager
        .register_package_manifest_asset_roots(&parent_root, &package_root)
        .expect_err("parent-relative package roots must be rejected");
    assert!(parent_error
        .to_string()
        .contains("must be relative and contained by the package root"));

    let multi_root = PluginPackageManifest::new("navigation", "Navigation")
        .with_package_identity("com", "zircon", "navigation")
        .with_asset_roots(["assets", "more_assets"]);
    let multi_error = manager
        .register_package_manifest_asset_roots(&multi_root, &package_root)
        .expect_err("ambiguous package roots must be rejected");
    assert!(multi_error
        .to_string()
        .contains("requires exactly one root"));

    let empty_id_error = manager
        .register_package_asset_root("", package_root.join("assets"))
        .expect_err("empty package ids must be rejected");
    assert!(empty_id_error
        .to_string()
        .contains("package resource locator"));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

fn multi_asset_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.package.multi", "test.package", AssetKind::Data, 1)
            .with_source_extensions(["multi"])
            .with_additional_output_kinds([AssetKind::Texture]),
        import_multi_asset_bundle,
    )
}

fn import_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let texture_uri = AssetUri::parse(&format!("{}#Texture0", context.uri)).unwrap();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "bundle": true }),
        }),
    )
    .with_entry(ImportedAssetEntry::new(
        texture_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            texture_uri,
            1,
            1,
            vec![255, 0, 255, 255],
        )),
    )))
}

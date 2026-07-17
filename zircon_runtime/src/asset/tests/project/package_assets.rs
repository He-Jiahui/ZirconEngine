use std::fs;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetMetaDocument, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter,
    ImportedAsset, ImportedAssetEntry, ProjectManager, ProjectManifest, ProjectPaths,
};
use crate::core::resource::ResourceState;
use crate::plugin::PluginPackageManifest;
use zircon_runtime_interface::project::RelPath;

#[test]
fn project_manager_open_registers_default_and_explicit_ordered_project_roots() {
    let default_root = unique_temp_project_root("project_default_roots");
    let default_paths = ProjectPaths::from_root(&default_root).unwrap();
    ProjectManifest::new(
        "DefaultRoots",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(default_paths.manifest_path())
    .unwrap();

    let default_manager = ProjectManager::open(&default_root).unwrap();
    assert_eq!(
        default_manager.package_assets().project_roots(),
        &[default_root.join("assets")]
    );

    let explicit_root = unique_temp_project_root("project_explicit_roots");
    let explicit_paths = ProjectPaths::from_root(&explicit_root).unwrap();
    let mut manifest = ProjectManifest::new(
        "ExplicitRoots",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    );
    manifest.asset_roots = vec![
        RelPath::parse("game-assets").unwrap(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    manifest.save(explicit_paths.manifest_path()).unwrap();

    let explicit_manager = ProjectManager::open(&explicit_root).unwrap();
    assert_eq!(
        explicit_manager.package_assets().project_roots(),
        &[
            explicit_root.join("game-assets"),
            explicit_root.join("shared-assets")
        ]
    );

    let _ = fs::remove_dir_all(default_root);
    let _ = fs::remove_dir_all(explicit_root);
}

#[test]
fn multiple_project_roots_scan_distinct_res_uris_and_reject_collisions() {
    let root = unique_temp_project_root("project_multi_root_scan");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let mut manifest = ProjectManifest::new(
        "MultiRoot",
        AssetUri::parse("res://scenes/main.json").unwrap(),
        1,
    );
    manifest.asset_roots = vec![
        RelPath::parse("game-assets").unwrap(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    manifest.save(paths.manifest_path()).unwrap();
    fs::create_dir_all(root.join("game-assets/scenes")).unwrap();
    fs::create_dir_all(root.join("shared-assets/data")).unwrap();
    fs::write(root.join("game-assets/scenes/main.json"), "{}").unwrap();
    fs::write(root.join("shared-assets/data/shared.json"), "{}").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    let records = manager.scan_and_import().unwrap();
    assert!(records.iter().any(|record| {
        record.primary_locator() == &AssetUri::parse("res://scenes/main.json").unwrap()
    }));
    assert!(records.iter().any(|record| {
        record.primary_locator() == &AssetUri::parse("res://data/shared.json").unwrap()
    }));
    let shared_uri = AssetUri::parse("res://data/shared.json").unwrap();
    assert_eq!(
        manager.source_path_for_uri(&shared_uri).unwrap(),
        root.join("shared-assets/data/shared.json")
    );
    let new_uri = AssetUri::parse("res://generated/new.json").unwrap();
    assert!(matches!(
        manager.source_path_for_uri(&new_uri),
        Err(AssetImportError::MissingProjectAssetUri { .. })
    ));
    assert_eq!(
        manager
            .primary_project_source_path_for_uri(&new_uri)
            .unwrap(),
        root.join("game-assets/generated/new.json")
    );

    fs::create_dir_all(root.join("shared-assets/scenes")).unwrap();
    fs::write(root.join("shared-assets/scenes/main.json"), "{}").unwrap();
    let error = manager.scan_and_import().unwrap_err();
    assert!(matches!(
        error,
        AssetImportError::DuplicateProjectAssetUri { uri, first, second }
            if uri == AssetUri::parse("res://scenes/main.json").unwrap()
                && first != second
    ));
    assert!(matches!(
        manager.source_path_for_uri(&AssetUri::parse("res://scenes/main.json").unwrap()),
        Err(AssetImportError::AmbiguousProjectAssetUri { paths, .. }) if paths.len() == 2
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_root_registration_rejects_a_canonical_symlink_escape() {
    let root = unique_temp_project_root("project_root_symlink_escape");
    let outside = unique_temp_project_root("project_root_symlink_outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let linked = root.join("linked-assets");
    if !create_directory_symlink(&outside, &linked) {
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
        return;
    }
    let mut registry = crate::asset::project::PackageAssetRegistry::default();
    let error = registry
        .register_project_roots(&root, &[RelPath::parse("linked-assets").unwrap()])
        .unwrap_err();
    assert!(matches!(
        error,
        AssetImportError::CanonicalProjectAssetRootEscape { .. }
    ));
    let _ = fs::remove_file(linked);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

#[cfg(unix)]
fn create_directory_symlink(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
const WINDOWS_ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

#[cfg(windows)]
fn create_directory_symlink(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => true,
        Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(WINDOWS_ERROR_PRIVILEGE_NOT_HELD) =>
        {
            false
        }
        Err(error) => panic!("create directory symlink fixture failed: {error}"),
    }
}

#[test]
fn project_manager_registers_direct_package_asset_root() {
    let root = unique_temp_project_root("project_manager_direct_package_root");
    let package_assets_root = unique_temp_project_root("direct_navigation_assets");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
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
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
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
        .register_package_asset_roots(
            package_manifest.package_id(),
            package_manifest.asset_roots_or_default(),
            &package_root,
        )
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
        manager
            .asset_registry()
            .resolve_asset_id_by_path(&texture_uri),
        Ok(texture_record.id())
    );
    assert_eq!(
        manager
            .asset_registry()
            .resolve_asset_id_for_reference(texture_entry.uuid, &texture_uri),
        Ok(texture_record.id())
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
        .register_package_asset_roots(
            package_manifest.package_id(),
            package_manifest.asset_roots_or_default(),
            &package_root,
        )
        .unwrap();
    restarted.scan_and_import().unwrap();

    let restored_texture = restarted
        .registry()
        .get_by_locator(&texture_uri)
        .expect("restored package subasset record");
    assert_eq!(restored_texture.id(), texture_record.id());
    assert_eq!(
        restarted
            .asset_registry()
            .resolve_asset_id_for_reference(texture_entry.uuid, &texture_uri),
        Ok(texture_record.id())
    );

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn package_asset_registry_rejects_invalid_manifest_roots() {
    let root = unique_temp_project_root("project_manager_invalid_package_roots");
    let package_root = unique_temp_project_root("invalid_package_root");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "InvalidPackageRootSandbox",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    let parent_error = manager
        .register_package_asset_roots("com.zircon.navigation", ["../outside"], &package_root)
        .expect_err("parent-relative package roots must be rejected");
    assert!(parent_error
        .to_string()
        .contains("must be relative and contained by the package root"));

    let multi_error = manager
        .register_package_asset_roots(
            "com.zircon.navigation",
            ["assets", "more_assets"],
            &package_root,
        )
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

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::asset::project::{AssetMetaDocument, ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImportContext, AssetImportOutcome, DataAsset, DataAssetFormat, FunctionAssetImporter,
    ImportedAsset,
};

static QUALIFIED_IMPORT_CALLS: AtomicUsize = AtomicUsize::new(0);

fn descriptor(version: u32) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new("model-importer", "model-plugin", AssetKind::Model, version)
}

fn counted_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.qualified.data", "test.qualified", AssetKind::Data, 1)
            .with_source_extensions(["qualified"]),
        import_qualified_data,
    )
}

fn import_qualified_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    QUALIFIED_IMPORT_CALLS.fetch_add(1, Ordering::SeqCst);
    assert!(context.build_context().is_some());
    assert!(context
        .build_action_key()
        .is_some_and(|key| key.starts_with("blake3:")));
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text: context.source_text()?,
            canonical_json: serde_json::json!({ "qualified": true }),
        }),
    ))
}

#[test]
fn project_import_identity_qualifies_the_production_recipe_and_toolchain() {
    let mut settings = toml::Table::new();
    settings.insert("quality".to_string(), toml::Value::Integer(3));
    let importer = descriptor(7);
    let identity = build_identity_for_import(&settings, "blake3:source-a", Some(&importer), None);

    assert_eq!(identity.function_identity(), "model-importer");
    assert_eq!(
        identity.recipe().setting("quality"),
        Some(&crate::asset::importer::AssetImportRecipeValue::Integer(3))
    );
    assert_eq!(identity.input_digest(), "blake3:source-a");
    assert_eq!(identity.build_context().build_profile(), "project_import");
    assert_eq!(
        identity.build_context().engine_abi_version(),
        zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8
    );
    assert!(identity
        .build_context()
        .toolchain_identity()
        .contains("model-importer@7"));
    assert_ne!(
        identity.action_key(),
        build_identity_for_import(&settings, "blake3:source-b", Some(&importer), None).action_key()
    );
    assert_ne!(
        identity.action_key(),
        build_identity_for_import(&settings, "blake3:source-a", Some(&descriptor(8)), None,)
            .action_key()
    );
    assert_eq!(
        identity.action_key(),
        build_identity_for_import(
            &settings,
            "blake3:source-a",
            None,
            Some(("model-importer", 7)),
        )
        .action_key(),
        "restore without a loaded importer must reproduce the persisted function identity"
    );
}

#[test]
fn project_import_action_key_rebuilds_once_and_matches_full_and_targeted_paths() {
    let root = unique_temp_project_root("qualified_import_build_identity");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/settings.qualified").unwrap();
    ProjectManifest::new("QualifiedImport", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/settings.qualified");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "v1").unwrap();
    let meta_path = source_path.with_file_name("settings.qualified.zmeta");

    QUALIFIED_IMPORT_CALLS.store(0, Ordering::SeqCst);
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.register_asset_importer(counted_importer()).unwrap();
    manager.scan_and_import().unwrap();
    assert_eq!(QUALIFIED_IMPORT_CALLS.load(Ordering::SeqCst), 1);
    let first_key = AssetMetaDocument::load(&meta_path).unwrap().config_hash;
    assert!(first_key.starts_with("blake3:"));

    manager.scan_and_import().unwrap();
    assert_eq!(
        QUALIFIED_IMPORT_CALLS.load(Ordering::SeqCst),
        1,
        "the second full scan must restore the first action"
    );

    let mut legacy = AssetMetaDocument::load(&meta_path).unwrap();
    legacy.config_hash = "legacy-default-hasher-key".to_string();
    legacy.save(&meta_path).unwrap();
    let mut rebuilt = ProjectManager::open(&root).unwrap();
    rebuilt.register_asset_importer(counted_importer()).unwrap();
    rebuilt.scan_and_import().unwrap();
    assert_eq!(QUALIFIED_IMPORT_CALLS.load(Ordering::SeqCst), 2);
    assert_eq!(
        AssetMetaDocument::load(&meta_path).unwrap().config_hash,
        first_key
    );
    rebuilt.scan_and_import().unwrap();
    assert_eq!(
        QUALIFIED_IMPORT_CALLS.load(Ordering::SeqCst),
        2,
        "the rebuilt action must restore on its second scan"
    );

    fs::write(&source_path, "v2").unwrap();
    rebuilt.import_targeted_source(&uri, &source_path).unwrap();
    assert_eq!(QUALIFIED_IMPORT_CALLS.load(Ordering::SeqCst), 3);
    let targeted_key = AssetMetaDocument::load(&meta_path).unwrap().config_hash;
    assert_ne!(targeted_key, first_key);
    rebuilt.scan_and_import().unwrap();
    assert_eq!(
        QUALIFIED_IMPORT_CALLS.load(Ordering::SeqCst),
        3,
        "full generation must reproduce and restore the targeted action key"
    );
    assert_eq!(
        AssetMetaDocument::load(&meta_path).unwrap().config_hash,
        targeted_key
    );

    let _ = fs::remove_dir_all(root);
}

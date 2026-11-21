//! 验证项目源重定位只改变作者路径和 URI，保留 UUID 与 artifact；中断事务必须能恢复原代。

use crate::asset::registry::persistence;
use crate::core::resource::io::transaction::{
    DurableTransactionError, TransactionFault, TransactionPhase,
};

use super::*;

#[test]
fn project_source_relocation_moves_authoring_files_and_preserves_live_identity() {
    let root = unique_temp_project_root("project_manager_source_relocation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "SourceRelocationSandbox",
        AssetUri::parse("res://data/original.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/original.counted");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "relocation-source-v1").unwrap();

    let source = AssetUri::parse("res://data/original.counted").unwrap();
    let target = AssetUri::parse("res://moved/renamed.counted").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let source_uuid = manager
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let source_record = manager.registry().get_by_locator(&source).unwrap().clone();
    let artifact_path = paths.asset_artifact_root().join(
        source_record
            .artifact_locator()
            .expect("source imports an artifact")
            .path(),
    );
    let artifact_before = fs::read(&artifact_path).unwrap();
    let source_meta_path = source_path.with_file_name("original.counted.zmeta");
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("moved/renamed.counted");
    let target_meta_path = target_path.with_file_name("renamed.counted.zmeta");

    let updated = manager
        .relocate_project_source_for_test(source_uuid, target.clone())
        .expect("project source relocation should commit");

    assert!(!source_path.exists());
    assert!(!source_meta_path.exists());
    assert_eq!(fs::read(&target_path).unwrap(), b"relocation-source-v1");
    assert_eq!(
        AssetMetaDocument::load(&target_meta_path).unwrap().url,
        target
    );
    assert_eq!(fs::read(&artifact_path).unwrap(), artifact_before);
    assert!(manager.registry().get_by_locator(&source).is_none());
    assert_eq!(
        manager.registry().get_by_locator(&target).unwrap().id(),
        source_record.id()
    );
    assert_eq!(
        manager
            .asset_registry()
            .entry_by_path(&target)
            .unwrap()
            .uuid(),
        source_uuid
    );
    assert!(updated
        .iter()
        .any(|record| record.primary_locator() == &target));
    assert_eq!(
        manager
            .catalog_input_generation()
            .record(source_record.id())
            .unwrap()
            .source_path(),
        target_path
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn interrupted_project_source_relocation_recovers_the_original_generation() {
    assert_source_relocation_recovers(RelocationInterruption::SourceRetirement);
}

#[test]
fn interrupted_project_source_relocation_restores_replaced_index_and_retired_sidecar() {
    assert_source_relocation_recovers(RelocationInterruption::IndexReplacement);
}

enum RelocationInterruption {
    SourceRetirement,
    IndexReplacement,
}

fn assert_source_relocation_recovers(interruption: RelocationInterruption) {
    let root = unique_temp_project_root("project_manager_source_relocation_recovery");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "SourceRelocationRecoverySandbox",
        AssetUri::parse("res://data/original.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/original.counted");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "relocation-source-v1").unwrap();
    let source_meta_path = source_path.with_file_name("original.counted.zmeta");
    let source = AssetUri::parse("res://data/original.counted").unwrap();
    let target = AssetUri::parse("res://moved/renamed.counted").unwrap();
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("moved/renamed.counted");
    let target_meta_path = target_path.with_file_name("renamed.counted.zmeta");
    let registry_path = paths.registry_root().join("asset-registry.json");

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(relocation_recovery_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let source_uuid = manager
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let source_resource_id = manager.registry().get_by_locator(&source).unwrap().id();
    let source_before = fs::read(&source_path).unwrap();
    // Legal trailing whitespace makes restoration distinguishable from a rebuild or reserialize.
    let source_meta_before = mark_original_bytes(&source_meta_path);
    let registry_before = mark_original_bytes(&registry_path);

    let error = match interruption {
        RelocationInterruption::SourceRetirement => manager
            .relocate_project_source_with_source_retirement_interruption(
                source_uuid,
                target.clone(),
            ),
        // Writes are ordered source, sidecar, index. Interrupt the last replacement before the
        // transaction commit point, so all three files need recovery.
        RelocationInterruption::IndexReplacement => manager.relocate_project_source_with_fault(
            source_uuid,
            target.clone(),
            TransactionFault::CrashAfterTargetReplace(2),
        ),
    }
    .expect_err("the injected interruption must leave recovery evidence");
    let AssetImportError::Io(error) = error else {
        panic!("expected a transaction I/O error, got {error}");
    };
    assert!(
        matches!(
            error.get_ref().and_then(|cause| cause.downcast_ref::<DurableTransactionError>()),
            Some(DurableTransactionError::Operation { phase: TransactionPhase::Commit, source, .. })
                if source.kind() == std::io::ErrorKind::Interrupted
        ),
        "expected a commit interruption, got {error}"
    );
    assert_eq!(fs::read(&target_path).unwrap(), source_before);
    assert!(!source_path.exists());
    match interruption {
        RelocationInterruption::SourceRetirement => {
            assert_eq!(fs::read(&source_meta_path).unwrap(), source_meta_before);
            assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
            assert!(!target_meta_path.exists());
        }
        RelocationInterruption::IndexReplacement => {
            assert!(!source_meta_path.exists());
            assert_eq!(
                AssetMetaDocument::load(&target_meta_path).unwrap().url,
                target
            );
            let interrupted_index = persistence::load(&registry_path).unwrap();
            assert!(interrupted_index.entry_by_path(&source).is_none());
            assert_eq!(
                interrupted_index.entry_by_path(&target).unwrap().uuid(),
                source_uuid
            );
        }
    }
    drop(manager);

    let mut reopened = ProjectManager::open(&root)
        .expect("project recovery must accept and roll back a source relocation transaction");
    assert_eq!(fs::read(&source_path).unwrap(), source_before);
    assert_eq!(fs::read(&source_meta_path).unwrap(), source_meta_before);
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
    assert!(!target_path.exists());
    assert!(!target_meta_path.exists());

    let assert_original_index = |project: &ProjectManager| {
        let index = project.asset_registry();
        let asset = index
            .entry_by_path(&source)
            .expect("project must expose the recovered source index");
        assert_eq!(asset.uuid(), source_uuid);
        assert_eq!(asset.path(), &source);
        assert!(index.entry_by_path(&target).is_none());
    };
    let assert_unpublished = |project: &ProjectManager| {
        let registry = project.registry();
        assert!(registry.get(source_resource_id).is_none());
        assert!(registry.get_by_locator(&source).is_none());
        assert!(registry.get_by_locator(&target).is_none());
    };
    assert_original_index(&reopened);
    assert_unpublished(&reopened);

    // Registering the handler is not publication; only scanning publishes runtime records.
    reopened
        .register_asset_importer(relocation_recovery_importer())
        .unwrap();
    assert_unpublished(&reopened);
    reopened.scan_and_import().unwrap();
    let published = reopened
        .registry()
        .get_by_locator(&source)
        .expect("scan must publish the restored source");
    assert_eq!(published.id(), source_resource_id);
    assert_eq!(published.primary_locator(), &source);
    assert_eq!(published.state, ResourceState::Ready);
    assert!(reopened.registry().get_by_locator(&target).is_none());
    assert_original_index(&reopened);

    let _ = fs::remove_dir_all(root);
}

fn relocation_recovery_importer() -> FunctionAssetImporter {
    // This fixture does not touch the shared counter observed by other manager tests.
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.relocation.data",
            "test.relocation",
            AssetKind::Data,
            1,
        )
        .with_source_extensions(["counted"]),
        |context| {
            Ok(AssetImportOutcome::new(
                context.uri.clone(),
                ImportedAsset::Data(DataAsset {
                    uri: context.uri.clone(),
                    format: DataAssetFormat::Json,
                    text: context.source_text()?,
                    canonical_json: serde_json::json!({ "relocation": true }),
                }),
            ))
        },
    )
}

fn mark_original_bytes(path: &Path) -> Vec<u8> {
    let mut bytes = fs::read(path).unwrap();
    bytes.extend_from_slice(b"\n\t\n");
    fs::write(path, &bytes).unwrap();
    bytes
}

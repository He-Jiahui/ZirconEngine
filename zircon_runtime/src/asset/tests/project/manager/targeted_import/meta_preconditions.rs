use std::path::PathBuf;
use std::sync::Arc;

use crate::asset::project::{
    AssetMetaPreviewStateCasResult, AssetMetaPreviewStateExpectation, PreviewState,
};

use super::*;

#[test]
fn targeted_generation_preserves_preview_cas_completed_after_preparation() {
    let (root, paths, mut manager) = project_fixture("targeted-preview-precondition");
    let source = source_path(&paths, "first");
    let uri = source_uri("first");
    let meta_path = source.with_extension("json.zmeta");
    let artifact = artifact_path(&manager, &paths, &uri);
    let artifact_before = fs::read(&artifact).unwrap();
    let registry_before = fs::read(paths.registry_root().join("asset-registry.json")).unwrap();
    let generation_before = manager.catalog_input_generation();
    let expected = AssetMetaPreviewStateExpectation::from_document(
        &AssetMetaDocument::load(&meta_path).unwrap(),
    );

    fs::write(&source, "{\"value\":2}\n").unwrap();
    let mut candidate = manager.clone();
    let prepared = candidate
        .prepare_targeted_generation(&uri, &source)
        .unwrap();
    let writer_path = meta_path.clone();
    let result = std::thread::spawn(move || {
        AssetMetaDocument::compare_and_set_preview_state(
            writer_path,
            &expected,
            PreviewState::Dirty,
        )
        .unwrap()
    })
    .join()
    .unwrap();
    assert!(matches!(
        result,
        AssetMetaPreviewStateCasResult::Updated {
            previous: PreviewState::Ready,
            current: PreviewState::Dirty
        }
    ));
    let meta_after_cas = fs::read(&meta_path).unwrap();

    assert_stale(prepared.commit().unwrap_err());
    assert_eq!(fs::read(&meta_path).unwrap(), meta_after_cas);
    assert_eq!(fs::read(&artifact).unwrap(), artifact_before);
    assert_eq!(
        fs::read(paths.registry_root().join("asset-registry.json")).unwrap(),
        registry_before
    );
    assert!(Arc::ptr_eq(
        &generation_before,
        &manager.catalog_input_generation()
    ));

    manager.import_targeted_source(&uri, &source).unwrap();
    match manager.load_artifact(&uri).unwrap() {
        ImportedAsset::Data(asset) => assert_eq!(asset.canonical_json["value"], 2),
        other => panic!("unexpected targeted artifact: {other:?}"),
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn targeted_batch_rejects_changed_meta_before_publishing_any_member() {
    let (root, paths, manager) = project_fixture("targeted-batch-meta-precondition");
    let sources = [
        (source_uri("first"), source_path(&paths, "first")),
        (source_uri("second"), source_path(&paths, "second")),
    ];
    let artifacts = sources
        .iter()
        .map(|(uri, _)| artifact_path(&manager, &paths, uri))
        .collect::<Vec<_>>();
    let artifact_bytes = artifacts
        .iter()
        .map(|path| fs::read(path).unwrap())
        .collect::<Vec<_>>();
    let first_meta = sources[0].1.with_extension("json.zmeta");
    let first_meta_before = fs::read(&first_meta).unwrap();
    let second_meta = sources[1].1.with_extension("json.zmeta");
    let registry_before = fs::read(paths.registry_root().join("asset-registry.json")).unwrap();
    for (_, source) in &sources {
        fs::write(source, "{\"value\":2}\n").unwrap();
    }
    let mut candidate = manager.clone();
    let prepared = candidate.prepare_targeted_import_batch(&sources).unwrap();
    let mut latest = AssetMetaDocument::load(&second_meta).unwrap();
    latest.tags.insert("updated-after-preparation".to_string());
    latest
        .import_settings
        .insert("concurrent_setting".to_string(), toml::Value::Boolean(true));
    latest.save(&second_meta).unwrap();
    let second_meta_before = fs::read(&second_meta).unwrap();

    assert_stale(prepared.commit().unwrap_err());
    for (artifact, bytes) in artifacts.iter().zip(artifact_bytes) {
        assert_eq!(fs::read(artifact).unwrap(), bytes);
    }
    assert_eq!(fs::read(first_meta).unwrap(), first_meta_before);
    assert_eq!(fs::read(second_meta).unwrap(), second_meta_before);
    assert_eq!(
        fs::read(paths.registry_root().join("asset-registry.json")).unwrap(),
        registry_before
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn targeted_generation_rejects_sidecar_creation_after_preparing_a_new_source() {
    let (root, paths, manager) = project_fixture("targeted-new-meta-precondition");
    let source = source_path(&paths, "new");
    let uri = source_uri("new");
    fs::write(&source, "{\"value\":2}\n").unwrap();
    let meta_path = source.with_extension("json.zmeta");
    assert!(!meta_path.exists());
    let registry_before = fs::read(paths.registry_root().join("asset-registry.json")).unwrap();
    let mut candidate = manager.clone();
    let prepared = candidate
        .prepare_targeted_generation(&uri, &source)
        .unwrap();
    let artifact = artifact_path(&candidate, &paths, &uri);
    assert!(!artifact.exists());
    let mut concurrent = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Data);
    concurrent.tags.insert("concurrent-owner".to_string());
    concurrent.save(&meta_path).unwrap();
    let meta_bytes = fs::read(&meta_path).unwrap();

    assert_stale(prepared.commit().unwrap_err());
    assert_eq!(fs::read(meta_path).unwrap(), meta_bytes);
    assert!(!artifact.exists());
    assert_eq!(
        fs::read(paths.registry_root().join("asset-registry.json")).unwrap(),
        registry_before
    );
    assert!(manager.registry().get_by_locator(&uri).is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn targeted_generation_rejects_sidecar_removal_after_preparation() {
    let (root, paths, manager) = project_fixture("targeted-removed-meta-precondition");
    let source = source_path(&paths, "first");
    let uri = source_uri("first");
    let meta_path = source.with_extension("json.zmeta");
    let artifact = artifact_path(&manager, &paths, &uri);
    let artifact_before = fs::read(&artifact).unwrap();
    fs::write(&source, "{\"value\":2}\n").unwrap();
    let mut candidate = manager.clone();
    let prepared = candidate
        .prepare_targeted_generation(&uri, &source)
        .unwrap();
    {
        let _guard = crate::asset::project::lock_meta_document_path(&meta_path).unwrap();
        fs::remove_file(&meta_path).unwrap();
    }

    assert_stale(prepared.commit().unwrap_err());
    assert!(!meta_path.exists());
    assert_eq!(fs::read(artifact).unwrap(), artifact_before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn targeted_generation_compares_original_meta_before_identity_normalization() {
    let (root, paths, mut manager) = project_fixture("targeted-original-meta-precondition");
    let source = source_path(&paths, "first");
    let uri = source_uri("first");
    let meta_path = source.with_extension("json.zmeta");
    let mut meta = AssetMetaDocument::load(&meta_path).unwrap();
    meta.asset_kind = AssetKind::Texture;
    meta.tags.insert("preserve-original-fields".to_string());
    meta.save(&meta_path).unwrap();

    manager.import_targeted_source(&uri, &source).unwrap();
    let current = AssetMetaDocument::load(meta_path).unwrap();
    assert_eq!(current.asset_kind, AssetKind::Data);
    assert_eq!(current.uuid, meta.uuid);
    assert_eq!(current.tags, meta.tags);
    fs::remove_dir_all(root).unwrap();
}

fn project_fixture(label: &str) -> (PathBuf, ProjectPaths, ProjectManager) {
    let root = unique_temp_project_root(label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new("TargetedMetaPreconditions", source_uri("first"), 1)
        .save(paths.manifest_path())
        .unwrap();
    for name in ["first", "second"] {
        let source = source_path(&paths, name);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(source, "{\"value\":1}\n").unwrap();
    }
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    (root, paths, manager)
}

fn source_uri(name: &str) -> AssetUri {
    AssetUri::parse(&format!("res://data/{name}.json")).unwrap()
}

fn source_path(paths: &ProjectPaths, name: &str) -> PathBuf {
    paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join(format!("data/{name}.json"))
}

fn artifact_path(manager: &ProjectManager, paths: &ProjectPaths, uri: &AssetUri) -> PathBuf {
    paths.asset_artifact_root().join(
        manager
            .registry()
            .get_by_locator(uri)
            .unwrap()
            .artifact_locator()
            .unwrap()
            .path(),
    )
}

fn assert_stale(error: AssetImportError) {
    assert!(matches!(error, AssetImportError::Parse(message)
        if message.contains("project metadata changed while targeted generation was prepared")));
}

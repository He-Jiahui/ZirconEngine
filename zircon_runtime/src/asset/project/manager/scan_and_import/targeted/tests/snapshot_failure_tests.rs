use std::fs;

use crate::asset::project::{AssetMetaDocument, PreviewState, ProjectManifest, ProjectPaths};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetUri, ImportedAsset};
use crate::core::resource::ResourceState;

use super::ProjectManager;

#[test]
fn targeted_watch_auxiliary_failure_publishes_error_and_preserves_old_artifact() {
    let root = std::env::temp_dir().join(format!(
        "zircon-targeted-snapshot-failure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).unwrap();
    let asset_root = zircon_runtime_interface::project::RelPath::project_assets();
    paths.ensure_layout(&[asset_root.clone()]).unwrap();
    let assets = paths.asset_root(&asset_root);
    let uri = AssetUri::parse("res://model.gltf").unwrap();
    ProjectManifest::new("Targeted Snapshot Failure", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let model_path = assets.join("model.gltf");
    let model_bytes = br#"{
        "asset":{"version":"2.0"},
        "buffers":[{"uri":"aaa.bin","byteLength":36}],
        "bufferViews":[{"buffer":0,"byteLength":36}],
        "accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[10,1,0]}],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}]
    }"#;
    fs::write(&model_path, model_bytes).unwrap();
    let vertices = [0.25_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let buffer_path = assets.join("aaa.bin");
    fs::write(&buffer_path, &vertices).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let ready = manager.registry().get_by_locator(&uri).unwrap();
    assert_eq!(ready.state, ResourceState::Ready);
    let old_artifact = ready.artifact_locator.clone().unwrap();
    fs::remove_file(&buffer_path).unwrap();

    let changed = manager
        .scan_and_import_watch_changes(&[AssetChange::new(
            AssetChangeKind::Modified,
            uri.clone(),
            None,
        )])
        .expect("an admitted primary with a missing auxiliary must publish an Error record");
    assert!(changed
        .iter()
        .any(|record| { record.primary_locator == uri && record.state == ResourceState::Error }));
    let failed = manager.registry().get_by_locator(&uri).unwrap();
    assert_eq!(failed.state, ResourceState::Error);
    assert!(failed.artifact_locator.is_none());
    assert!(manager.load_artifact(&uri).is_err());
    assert!(failed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("aaa.bin")));
    let failed_meta = AssetMetaDocument::load(assets.join("model.gltf.zmeta")).unwrap();
    assert_eq!(failed_meta.preview_state, PreviewState::Error);
    assert!(failed_meta.artifact_locator.is_none());
    assert_eq!(fs::read(&model_path).unwrap(), model_bytes.to_vec());
    let old_payload = manager.artifact_store.read(&paths, &old_artifact).unwrap();
    assert!(matches!(old_payload, ImportedAsset::Model(_)));

    fs::write(&buffer_path, &vertices).unwrap();
    let recovered = manager
        .scan_and_import_watch_changes(&[AssetChange::new(
            AssetChangeKind::Modified,
            uri.clone(),
            None,
        )])
        .expect("restoring the auxiliary source must recover the targeted import");
    assert!(recovered
        .iter()
        .any(|record| record.primary_locator == uri && record.state == ResourceState::Ready));
    let ready_again = manager.registry().get_by_locator(&uri).unwrap();
    assert_eq!(ready_again.state, ResourceState::Ready);
    assert!(ready_again.artifact_locator.is_some());
    let ImportedAsset::Model(current_model) = manager.load_artifact(&uri).unwrap() else {
        panic!("recovered model artifact required");
    };
    assert_eq!(current_model.primitives[0].vertices[0].position[0], 0.25);
    assert_eq!(
        AssetMetaDocument::load(assets.join("model.gltf.zmeta"))
            .unwrap()
            .preview_state,
        PreviewState::Ready
    );

    fs::remove_dir_all(root).unwrap();
}

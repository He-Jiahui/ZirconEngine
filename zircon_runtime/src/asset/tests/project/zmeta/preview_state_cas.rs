use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use crate::asset::project::{
    AssetMetaDocument, AssetMetaEntry, AssetMetaPreviewStateCasResult,
    AssetMetaPreviewStateExpectation, AssetMetaPreviewStateStale, PreviewState,
};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::super::unique_temp_project_root;

#[test]
fn asset_meta_preview_state_cas_preserves_independent_fields_written_after_preview_read() {
    let root = unique_temp_project_root("asset_meta_preview_state_preserves_fields");
    let path = root.join("hero.data.zmeta");
    let mut initial = meta();
    initial.source_digest = "digest-v1".to_string();
    initial.save(&path).unwrap();

    let read_barrier = Arc::new(Barrier::new(2));
    let commit_barrier = Arc::new(Barrier::new(2));
    let preview_path = path.clone();
    let preview_read = Arc::clone(&read_barrier);
    let preview_commit = Arc::clone(&commit_barrier);
    let preview = thread::spawn(move || {
        let snapshot = AssetMetaDocument::load(&preview_path).unwrap();
        let expected = AssetMetaPreviewStateExpectation::from_document(&snapshot);
        preview_read.wait();
        preview_commit.wait();
        AssetMetaDocument::compare_and_set_preview_state(
            &preview_path,
            &expected,
            PreviewState::Ready,
        )
        .unwrap()
    });

    read_barrier.wait();
    let mut external = AssetMetaDocument::load(&path).unwrap();
    external.import_settings.insert(
        "quality".to_string(),
        toml::Value::String("high".to_string()),
    );
    external.tags.insert("hero".to_string());
    external.included_files = vec![AssetUri::parse("res://data/hero/part.json").unwrap()];
    external.entries.push(AssetMetaEntry {
        uuid: AssetUuid::new(),
        url: AssetUri::parse("res://data/hero.data#part").unwrap(),
        asset_kind: AssetKind::Data,
        artifact_locator: None,
        dependencies: Vec::new(),
        tags: Default::default(),
    });
    external.save(&path).unwrap();
    commit_barrier.wait();

    assert_eq!(
        preview.join().unwrap(),
        AssetMetaPreviewStateCasResult::Updated {
            previous: PreviewState::Dirty,
            current: PreviewState::Ready,
        }
    );
    let current = AssetMetaDocument::load(&path).unwrap();
    assert_eq!(current.preview_state, PreviewState::Ready);
    assert_eq!(current.source_digest, "digest-v1");
    assert_eq!(
        current.import_settings.get("quality"),
        Some(&toml::Value::String("high".to_string()))
    );
    assert!(current.tags.contains("hero"));
    assert_eq!(current.included_files, external.included_files);
    assert_eq!(current.entries, external.entries);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn asset_meta_preview_state_cas_returns_typed_stale_when_source_digest_changes() {
    let root = unique_temp_project_root("asset_meta_preview_state_digest_stale");
    let path = root.join("hero.data.zmeta");
    let mut initial = meta();
    initial.source_digest = "digest-v1".to_string();
    initial.save(&path).unwrap();
    let expected = AssetMetaPreviewStateExpectation::from_document(&initial);

    let mut imported = AssetMetaDocument::load(&path).unwrap();
    imported.source_digest = "digest-v2".to_string();
    imported.preview_state = PreviewState::Error;
    imported.save(&path).unwrap();

    let result =
        AssetMetaDocument::compare_and_set_preview_state(&path, &expected, PreviewState::Ready)
            .unwrap();
    assert_eq!(
        result,
        AssetMetaPreviewStateCasResult::Stale(AssetMetaPreviewStateStale::SourceDigest {
            expected: "digest-v1".to_string(),
            current: "digest-v2".to_string(),
        })
    );
    assert_eq!(
        AssetMetaDocument::load(&path).unwrap().preview_state,
        PreviewState::Error
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn asset_meta_preview_state_cas_validates_uuid_and_url_identity() {
    let root = unique_temp_project_root("asset_meta_preview_state_identity_stale");
    let path = root.join("hero.data.zmeta");
    let initial = meta();
    initial.save(&path).unwrap();

    let mut wrong_uuid = AssetMetaPreviewStateExpectation::from_document(&initial);
    wrong_uuid.uuid = AssetUuid::new();
    assert!(matches!(
        AssetMetaDocument::compare_and_set_preview_state(&path, &wrong_uuid, PreviewState::Ready)
            .unwrap(),
        AssetMetaPreviewStateCasResult::Stale(AssetMetaPreviewStateStale::Uuid { .. })
    ));

    let mut wrong_url = AssetMetaPreviewStateExpectation::from_document(&initial);
    wrong_url.url = AssetUri::parse("res://data/other.data").unwrap();
    assert!(matches!(
        AssetMetaDocument::compare_and_set_preview_state(&path, &wrong_url, PreviewState::Ready)
            .unwrap(),
        AssetMetaPreviewStateCasResult::Stale(AssetMetaPreviewStateStale::Url { .. })
    ));
    assert_eq!(
        AssetMetaDocument::load(&path).unwrap().preview_state,
        PreviewState::Dirty
    );

    fs::remove_dir_all(root).unwrap();
}

fn meta() -> AssetMetaDocument {
    AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://data/hero.data").unwrap(),
        AssetKind::Data,
    )
}

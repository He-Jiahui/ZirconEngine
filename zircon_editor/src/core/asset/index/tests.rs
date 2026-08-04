use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime::asset::project::{AssetMetaDocument, AssetMetaEntry, PreviewState};
use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::watch::AssetWatchEvent;
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

use super::{EditorAssetImportState, EditorAssetIndex, EditorAssetIndexError};

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(label)
}

fn entry(label: &str, path: &str, digest: &str) -> AssetRegistryEntry {
    AssetRegistryEntry::new(uuid(label), uri(path), AssetKind::Texture, digest)
}

fn registry(entries: Vec<AssetRegistryEntry>) -> Arc<AssetRegistryIndex> {
    Arc::new(AssetRegistryIndex::from_entries(entries).unwrap())
}

fn ready_meta(label: &str, path: &str, digest: &str) -> Arc<AssetMetaDocument> {
    let mut meta = AssetMetaDocument::new(uuid(label), uri(path), AssetKind::Texture);
    meta.source_digest = digest.to_owned();
    meta.source_mtime_unix_ms = 17;
    meta.artifact_locator = Some(uri("lib://derived/texture.zasset"));
    meta.tags = BTreeSet::from(["environment".to_owned()]);
    Arc::new(meta)
}

fn compound_meta(mtime: u64, child_path: &str) -> Arc<AssetMetaDocument> {
    let mut meta = AssetMetaDocument::new(
        uuid("compound-root"),
        uri("res://models/ship.glb"),
        AssetKind::Model,
    );
    meta.source_digest = "compound-digest".to_owned();
    meta.source_mtime_unix_ms = mtime;
    meta.tags = BTreeSet::from(["vehicle".to_owned()]);
    meta.entries = vec![
        AssetMetaEntry {
            uuid: uuid("compound-root"),
            url: uri("res://models/ship.glb"),
            asset_kind: AssetKind::Model,
            artifact_locator: Some(uri("lib://derived/ship.zasset")),
            dependencies: Vec::new(),
            tags: BTreeSet::new(),
        },
        AssetMetaEntry {
            uuid: uuid("compound-child"),
            url: uri(child_path),
            asset_kind: AssetKind::Mesh,
            artifact_locator: Some(uri("lib://derived/ship-mesh.zasset")),
            dependencies: Vec::new(),
            tags: BTreeSet::from(["lod0".to_owned()]),
        },
    ];
    Arc::new(meta)
}

fn compound_registry() -> Arc<AssetRegistryIndex> {
    registry(vec![
        AssetRegistryEntry::new(
            uuid("compound-root"),
            uri("res://models/ship.glb"),
            AssetKind::Model,
            "compound-digest",
        )
        .with_tags(BTreeSet::from(["vehicle".to_owned()])),
        AssetRegistryEntry::new(
            uuid("compound-child"),
            uri("res://models/ship.glb#mesh"),
            AssetKind::Mesh,
            "compound-digest",
        )
        .with_tags(BTreeSet::from(["lod0".to_owned()])),
    ])
}

#[test]
fn rows_borrow_runtime_registry_authority_and_meta_v7_projection() {
    let registry = registry(vec![
        entry("sky", "res://textures/sky.png", "digest-a")
            .with_tags(BTreeSet::from(["environment".to_owned()]))
            .with_dependencies(vec![uuid("shared-sampler")]),
    ]);
    let expected_entry = registry.entry_by_uuid(uuid("sky")).unwrap() as *const _;
    let mut index = EditorAssetIndex::new(Arc::clone(&registry));

    index
        .ingest_meta_document(ready_meta("sky", "res://textures/sky.png", "digest-a"))
        .unwrap();
    let row = index.row_by_uuid(uuid("sky")).unwrap();

    assert!(Arc::ptr_eq(index.runtime_registry(), &registry));
    assert_eq!(row.runtime_entry() as *const _, expected_entry);
    assert_eq!(row.uuid(), uuid("sky"));
    assert_eq!(row.path(), &uri("res://textures/sky.png"));
    assert_eq!(row.type_marker(), AssetKind::Texture);
    assert!(row.tags().contains("environment"));
    assert_eq!(row.dependencies(), &[uuid("shared-sampler")]);
    assert_eq!(row.source_digest(), "digest-a");
    assert_eq!(row.source_mtime_unix_ms(), Some(17));
    assert_eq!(
        row.import_products().collect::<Vec<_>>(),
        vec![&uri("lib://derived/texture.zasset")]
    );
    assert!(row.import_valid());
    assert_eq!(row.import_state(), EditorAssetImportState::Ready);
}

#[test]
fn watch_events_mark_only_touched_runtime_entries_dirty() {
    let mut index = EditorAssetIndex::new(registry(vec![
        entry("sky", "res://textures/sky.png", "digest-a"),
        entry("ground", "res://textures/ground.png", "digest-b"),
    ]));
    index
        .ingest_meta_document(ready_meta("sky", "res://textures/sky.png", "digest-a"))
        .unwrap();
    index
        .ingest_meta_document(ready_meta(
            "ground",
            "res://textures/ground.png",
            "digest-b",
        ))
        .unwrap();

    index.apply_watch_events(&[AssetWatchEvent::Modified(uri("res://textures/sky.png"))]);

    assert_eq!(
        index.row_by_uuid(uuid("sky")).unwrap().import_state(),
        EditorAssetImportState::Stale
    );
    assert_eq!(
        index.row_by_uuid(uuid("ground")).unwrap().import_state(),
        EditorAssetImportState::Ready
    );
    assert_eq!(index.pending_dirty_path_count(), 0);
}

#[test]
fn runtime_snapshot_replacement_resolves_pending_added_paths() {
    let mut index = EditorAssetIndex::new(registry(Vec::new()));
    let added = uri("res://textures/new.png");
    index.apply_watch_events(&[AssetWatchEvent::Added(added.clone())]);
    assert_eq!(index.pending_dirty_path_count(), 1);

    index.replace_runtime_registry(registry(vec![entry(
        "new",
        "res://textures/new.png",
        "digest-new",
    )]));

    assert_eq!(index.pending_dirty_path_count(), 0);
    let row = index.row_by_uuid(uuid("new")).unwrap();
    assert_eq!(row.path(), &added);
    assert_eq!(row.import_state(), EditorAssetImportState::Stale);
}

#[test]
fn metadata_mismatch_is_typed_and_atomic() {
    let mut index = EditorAssetIndex::new(registry(vec![entry(
        "sky",
        "res://textures/sky.png",
        "digest-a",
    )]));

    let error = index
        .ingest_meta_document(ready_meta("sky", "res://textures/not-sky.png", "digest-a"))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorAssetIndexError::MetadataPathMismatch { uuid: found, .. }
            if found == uuid("sky")
    ));
    let row = index.row_by_uuid(uuid("sky")).unwrap();
    assert_eq!(row.source_mtime_unix_ms(), None);
    assert_eq!(row.import_state(), EditorAssetImportState::Stale);
}

#[test]
fn import_state_precedence_is_deterministic() {
    let mut index = EditorAssetIndex::new(registry(vec![entry(
        "sky",
        "res://textures/sky.png",
        "digest-a",
    )]));
    let mut broken = (*ready_meta("sky", "res://textures/sky.png", "digest-a")).clone();
    broken.artifact_locator = None;
    index.ingest_meta_document(Arc::new(broken)).unwrap();
    assert_eq!(
        index.row_by_uuid(uuid("sky")).unwrap().import_state(),
        EditorAssetImportState::Broken
    );

    index.begin_import(uuid("sky")).unwrap();
    assert_eq!(
        index.row_by_uuid(uuid("sky")).unwrap().import_state(),
        EditorAssetImportState::Importing
    );
    index.clear_import(uuid("sky"));
    assert_eq!(
        index.row_by_uuid(uuid("sky")).unwrap().import_state(),
        EditorAssetImportState::Broken
    );
}

#[test]
fn persisted_artifact_completeness_drives_import_validity_not_preview_state() {
    let mut index = EditorAssetIndex::new(registry(vec![
        entry("sky", "res://textures/sky.png", "digest-a")
            .with_tags(BTreeSet::from(["environment".to_owned()])),
    ]));
    let mut metadata = (*ready_meta("sky", "res://textures/sky.png", "digest-a")).clone();
    metadata.preview_state = PreviewState::Error;

    index.ingest_meta_document(Arc::new(metadata)).unwrap();
    let row = index.row_by_uuid(uuid("sky")).unwrap();

    assert!(row.import_valid());
    assert_eq!(row.import_state(), EditorAssetImportState::Ready);
}

#[test]
fn removed_and_renamed_unknown_paths_cancel_pending_tombstones() {
    let mut index = EditorAssetIndex::new(registry(Vec::new()));
    let removed = uri("res://textures/removed.png");
    let renamed = uri("res://textures/renamed.png");
    let target = uri("res://textures/target.png");

    index.apply_watch_events(&[
        AssetWatchEvent::Added(removed.clone()),
        AssetWatchEvent::Removed(removed),
        AssetWatchEvent::Added(renamed.clone()),
        AssetWatchEvent::Renamed {
            from: renamed,
            to: target.clone(),
        },
    ]);

    assert_eq!(index.pending_dirty_path_count(), 1);
    index.replace_runtime_registry(registry(vec![entry(
        "target",
        "res://textures/target.png",
        "digest-target",
    )]));
    assert_eq!(index.pending_dirty_path_count(), 0);
    assert_eq!(
        index.row_by_path(&target).unwrap().import_state(),
        EditorAssetImportState::Stale
    );
}

#[test]
fn reingesting_a_document_removes_deleted_child_projections() {
    let mut index = EditorAssetIndex::new(compound_registry());
    index
        .ingest_meta_document(compound_meta(17, "res://models/ship.glb#mesh"))
        .unwrap();
    assert_eq!(
        index
            .row_by_uuid(uuid("compound-child"))
            .unwrap()
            .source_mtime_unix_ms(),
        Some(17)
    );

    let mut reduced = (*compound_meta(23, "res://models/ship.glb#mesh")).clone();
    reduced.entries.pop();
    index.ingest_meta_document(Arc::new(reduced)).unwrap();

    let removed_child = index.row_by_uuid(uuid("compound-child")).unwrap();
    assert_eq!(removed_child.source_mtime_unix_ms(), None);
    assert!(!removed_child.import_valid());
    assert_eq!(removed_child.import_state(), EditorAssetImportState::Stale);
}

#[test]
fn multi_entry_validation_failure_rolls_back_the_whole_document() {
    let mut index = EditorAssetIndex::new(compound_registry());
    index
        .ingest_meta_document(compound_meta(17, "res://models/ship.glb#mesh"))
        .unwrap();

    let error = index
        .ingest_meta_document(compound_meta(99, "res://models/ship.glb#wrong"))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorAssetIndexError::MetadataPathMismatch { uuid: found, .. }
            if found == uuid("compound-child")
    ));
    for projected in [uuid("compound-root"), uuid("compound-child")] {
        assert_eq!(
            index.row_by_uuid(projected).unwrap().source_mtime_unix_ms(),
            Some(17)
        );
    }
}

#[test]
fn metadata_refresh_does_not_complete_an_active_import() {
    let mut index = EditorAssetIndex::new(registry(vec![
        entry("sky", "res://textures/sky.png", "digest-a")
            .with_tags(BTreeSet::from(["environment".to_owned()])),
    ]));
    index.begin_import(uuid("sky")).unwrap();

    index
        .ingest_meta_document(ready_meta("sky", "res://textures/sky.png", "digest-a"))
        .unwrap();

    assert_eq!(
        index.row_by_uuid(uuid("sky")).unwrap().import_state(),
        EditorAssetImportState::Importing
    );
    index.clear_import(uuid("sky"));
    assert_eq!(
        index.row_by_uuid(uuid("sky")).unwrap().import_state(),
        EditorAssetImportState::Ready
    );
}

#[test]
fn rows_are_path_sorted_for_reverse_registry_input() {
    let index = EditorAssetIndex::new(registry(vec![
        entry("z", "res://textures/z.png", "digest-z"),
        entry("a", "res://textures/a.png", "digest-a"),
        entry("m", "res://textures/m.png", "digest-m"),
    ]));

    assert_eq!(
        index
            .rows()
            .into_iter()
            .map(|row| row.path().clone())
            .collect::<Vec<_>>(),
        vec![
            uri("res://textures/a.png"),
            uri("res://textures/m.png"),
            uri("res://textures/z.png"),
        ]
    );
}

#[test]
fn runtime_registry_replacement_resolves_pending_paths_without_cloning_path_keys() {
    let source = include_str!("../index.rs");

    assert!(source.contains("pending_dirty_paths.retain"));
    assert!(!source.contains(".map(|entry| (path.clone(), entry.uuid()))"));
}

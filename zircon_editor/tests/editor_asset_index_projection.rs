use std::sync::Arc;

use zircon_editor::core::asset::{EditorAssetImportState, EditorAssetIndex};
use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::watch::AssetWatchEvent;
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(label)
}

fn runtime_registry(path: &str, digest: &str) -> Arc<AssetRegistryIndex> {
    Arc::new(
        AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
            uuid("asset"),
            uri(path),
            AssetKind::Texture,
            digest,
        )])
        .unwrap(),
    )
}

fn meta(path: &str, digest: &str) -> Arc<AssetMetaDocument> {
    let mut meta = AssetMetaDocument::new(uuid("asset"), uri(path), AssetKind::Texture);
    meta.source_digest = digest.to_owned();
    meta.source_mtime_unix_ms = 42;
    meta.artifact_locator = Some(uri("lib://derived/asset.zasset"));
    meta.preview_state = PreviewState::Ready;
    Arc::new(meta)
}

#[test]
fn public_editor_asset_index_projects_runtime_registry_and_meta_v7() {
    let runtime = runtime_registry("res://textures/icon.png", "digest-1");
    let mut index = EditorAssetIndex::new(Arc::clone(&runtime));
    index
        .ingest_meta_document(meta("res://textures/icon.png", "digest-1"))
        .unwrap();

    let row = index.row_by_uuid(uuid("asset")).unwrap();
    assert_eq!(
        row.runtime_entry(),
        runtime.entry_by_uuid(uuid("asset")).unwrap()
    );
    assert_eq!(row.source_mtime_unix_ms(), Some(42));
    assert_eq!(row.import_state(), EditorAssetImportState::Ready);
}

#[test]
fn public_editor_asset_index_reconciles_watch_events_against_replaced_runtime_snapshot() {
    let mut index = EditorAssetIndex::new(runtime_registry("res://textures/old.png", "digest-1"));
    index
        .ingest_meta_document(meta("res://textures/old.png", "digest-1"))
        .unwrap();
    index.apply_watch_events(&[AssetWatchEvent::Renamed {
        from: uri("res://textures/old.png"),
        to: uri("res://textures/new.png"),
    }]);
    index.replace_runtime_registry(runtime_registry("res://textures/new.png", "digest-2"));

    let row = index.row_by_uuid(uuid("asset")).unwrap();
    assert_eq!(row.path(), &uri("res://textures/new.png"));
    assert_eq!(row.source_digest(), "digest-2");
    assert_eq!(row.import_state(), EditorAssetImportState::Stale);
    assert_eq!(row.source_mtime_unix_ms(), None);
}

use std::fs;
use std::sync::Arc;

use crate::asset::project::ProjectPaths;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    ArtifactStore, AssetId, AssetKind, AssetUri, DataAsset, DataAssetFormat, ImportedAsset,
    TextureAsset,
};
use crate::core::resource::ResourceRecord;

#[test]
fn artifact_store_lazily_resides_only_requested_compressed_chunks() {
    let root = unique_temp_project_root("artifact_store_lazy_chunk_residency");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://textures/lazy-chunks.png").unwrap();
    let mut state = 0x9e37_79b9_u32;
    let rgba = (0..(384 * 192 * 4))
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect();
    let asset = ImportedAsset::Texture(TextureAsset::new_rgba8(uri.clone(), 384, 192, rgba));
    let store = ArtifactStore::with_chunk_residency_budget(64 * 1024);
    let artifact_uri = store
        .write(
            &paths,
            &ResourceRecord::new(AssetId::new(), AssetKind::Texture, uri),
            &asset,
        )
        .unwrap();
    let inventory = store.open_chunk_inventory(&paths, &artifact_uri).unwrap();

    assert!(inventory.len() > 1);
    assert_eq!(inventory.kind(), AssetKind::Texture);
    assert!(!inventory.content_hash().is_empty());
    let before = store.chunk_residency_diagnostics().unwrap();
    let first = store.read_compressed_chunk(&inventory, 0).unwrap();
    let warm = store.read_compressed_chunk(&inventory, 0).unwrap();
    let after_warm = store.chunk_residency_diagnostics().unwrap();

    assert!(Arc::ptr_eq(&first, &warm));
    assert_eq!(after_warm.disk_reads - before.disk_reads, 1);
    assert_eq!(after_warm.cache_hits - before.cache_hits, 1);
    assert_eq!(after_warm.resident_chunks, 1);
    assert!(after_warm.resident_bytes <= after_warm.max_resident_bytes);

    let _ = store.read_compressed_chunk(&inventory, 1).unwrap();
    let after_second = store.chunk_residency_diagnostics().unwrap();
    assert!(after_second.resident_bytes <= after_second.max_resident_bytes);
    assert!(after_second.evictions > after_warm.evictions);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_a_corrupt_requested_chunk_without_residing_it() {
    let root = unique_temp_project_root("artifact_store_lazy_corrupt_chunk");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/lazy-corrupt.json").unwrap();
    let asset = ImportedAsset::Data(DataAsset {
        uri: uri.clone(),
        format: DataAssetFormat::Json,
        text: "{\"current\":true}".to_string(),
        canonical_json: serde_json::json!({"current": true}),
    });
    let store = ArtifactStore::default();
    let artifact_uri = store
        .write(
            &paths,
            &ResourceRecord::new(AssetId::new(), AssetKind::Data, uri),
            &asset,
        )
        .unwrap();
    let inventory = store.open_chunk_inventory(&paths, &artifact_uri).unwrap();
    let chunk = inventory.chunk(0).unwrap();
    let chunk_path = paths
        .asset_artifact_root()
        .join("chunks")
        .join(format!("{}.zchunk", chunk.content_hash()));
    fs::write(chunk_path, vec![0_u8; chunk.compressed_bytes() as usize]).unwrap();

    assert!(store.read_compressed_chunk(&inventory, 0).is_err());
    assert_eq!(
        store.chunk_residency_diagnostics().unwrap().resident_chunks,
        0
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_unpublished_prepared_generation_keeps_last_good_manifest() {
    let root = unique_temp_project_root("artifact_store_interrupted_generation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/interrupted.json").unwrap();
    let first = ImportedAsset::Data(DataAsset {
        uri: uri.clone(),
        format: DataAssetFormat::Json,
        text: "{\"generation\":1}".to_string(),
        canonical_json: serde_json::json!({"generation": 1}),
    });
    let second = ImportedAsset::Data(DataAsset {
        uri: uri.clone(),
        format: DataAssetFormat::Json,
        text: "{\"generation\":2}".to_string(),
        canonical_json: serde_json::json!({"generation": 2}),
    });
    let store = ArtifactStore::default();
    let mut record = ResourceRecord::new(AssetId::new(), AssetKind::Data, uri);
    let artifact_uri = store.write(&paths, &record, &first).unwrap();

    record.revision = 2;
    let prepared = store.prepare_write(&paths, &record, &second).unwrap();
    assert_eq!(prepared.locator, artifact_uri);
    drop(prepared);

    assert_eq!(store.read(&paths, &artifact_uri).unwrap(), first);

    let _ = fs::remove_dir_all(root);
}

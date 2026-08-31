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
fn artifact_chunk_external_lease_registry_indexes_payload_identity() {
    let source = include_str!("../../../artifact/chunk_residency.rs");
    let tracking = source
        .split("fn track_external_lease")
        .nth(1)
        .and_then(|source| source.split("fn collect_retired_external_leases").next())
        .expect("artifact residency must retain the external-lease tracking path");

    assert!(source.contains("retired_external_leases: HashMap<usize, RetiredArtifactChunkLease>"));
    assert!(tracking.contains(".entry(payload_identity)"));
    assert!(tracking.contains("Entry::Occupied"));
    assert!(tracking.contains("Some(occupied.get().slot_index)"));
    assert!(tracking.contains("self.retired_external_leases.insert("));
    assert!(!tracking.contains(".retain("));
}

#[test]
fn artifact_chunk_residency_uses_a_bounded_lazy_eviction_index() {
    let source = include_str!("../../../artifact/chunk_residency.rs");

    assert!(source
        .contains("eviction_candidates: BinaryHeap<Reverse<(u64, Arc<ArtifactChunkCacheKey>)>>"));
    assert!(source.contains("MAX_EVICTION_INDEX_CANDIDATES_PER_RESIDENT_ENTRY"));
    assert!(source.contains("fn pop_oldest_resident_key"));
    assert!(!source.contains(".min_by_key(|(_, entry)| entry.last_access)"));
}

#[test]
fn artifact_chunk_residency_reuses_the_resident_canonical_key_on_hot_hits() {
    let source = include_str!("../../../artifact/chunk_residency.rs");
    let cached = source
        .split("fn cached")
        .nth(1)
        .and_then(|source| source.split("fn publish").next())
        .expect("artifact residency must retain the cache-hit path");

    assert!(cached.contains("Arc::clone(&entry.cache_key)"));
    assert!(!cached.contains(".get_key_value(key)"));
}

#[test]
fn artifact_chunk_residency_hot_reads_reuse_inventory_identity_arcs() {
    let source = include_str!("../../../artifact/chunk_residency.rs");
    let store = include_str!("../../../artifact/store.rs");
    let read = source
        .split("pub(super) fn read(")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn diagnostics").next())
        .expect("artifact residency read path");

    assert!(source.contains("chunk_root: Arc<PathBuf>"));
    assert!(source.contains("content_hash: Arc<str>"));
    assert!(read.contains("Arc::clone(&inventory.chunk_root)"));
    assert!(read.contains("Arc::clone(&descriptor.content_hash)"));
    assert!(!read.contains("inventory.chunk_root.clone()"));
    assert!(!read.contains("descriptor.content_hash.clone()"));
    assert!(store.contains("let content_hash: Arc<str>"));
    assert!(store.contains("ArtifactChunkDescriptor::new("));
    assert!(!store.contains("to_hex().to_string()"));
}

#[test]
fn artifact_chunk_reader_reuses_its_owned_inventory_hash() {
    let source = include_str!("../../../artifact/chunk_residency.rs");
    let reader = source
        .split("impl ChunkReader")
        .nth(1)
        .and_then(|source| source.split("impl Read for ChunkReader").next())
        .expect("chunk reader implementation");

    assert!(!source.contains("expected_content_hash: String"));
    assert!(reader.contains("self.inventory.content_hash()"));
}

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
    assert_eq!(after_warm.external_lease_tracking_overflows, 0);

    let _ = store.read_compressed_chunk(&inventory, 1).unwrap();
    let after_second = store.chunk_residency_diagnostics().unwrap();
    assert!(after_second.resident_bytes <= after_second.max_resident_bytes);
    assert!(after_second.evictions > after_warm.evictions);
    assert_eq!(after_second.externally_leased_chunks, 1);
    assert_eq!(after_second.externally_leased_bytes, first.len());
    assert_eq!(after_second.external_lease_tracking_overflows, 0);
    assert_eq!(
        after_second.tracked_payload_chunks,
        after_second.resident_chunks + 1
    );
    assert_eq!(
        after_second.tracked_payload_bytes,
        after_second.resident_bytes + first.len()
    );

    drop(warm);
    drop(first);
    let after_external_release = store.chunk_residency_diagnostics().unwrap();
    assert_eq!(after_external_release.externally_leased_chunks, 0);
    assert_eq!(after_external_release.externally_leased_bytes, 0);
    assert_eq!(after_external_release.external_lease_tracking_overflows, 0);
    assert_eq!(
        after_external_release.tracked_payload_chunks,
        after_external_release.resident_chunks
    );
    assert_eq!(
        after_external_release.tracked_payload_bytes,
        after_external_release.resident_bytes
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_lazily_tracks_an_oversized_chunk_while_a_consumer_holds_it() {
    let root = unique_temp_project_root("artifact_store_oversized_chunk_external_lease");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://textures/oversized-chunk.png").unwrap();
    let mut state = 0xa341_316c_u32;
    let rgba = (0..(384 * 192 * 4))
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect();
    let asset = ImportedAsset::Texture(TextureAsset::new_rgba8(uri.clone(), 384, 192, rgba));
    let store = ArtifactStore::with_chunk_residency_budget(1);
    let artifact_uri = store
        .write(
            &paths,
            &ResourceRecord::new(AssetId::new(), AssetKind::Texture, uri),
            &asset,
        )
        .unwrap();
    let inventory = store.open_chunk_inventory(&paths, &artifact_uri).unwrap();

    let chunk = store.read_compressed_chunk(&inventory, 0).unwrap();
    let while_held = store.chunk_residency_diagnostics().unwrap();
    assert_eq!(while_held.resident_chunks, 0);
    assert_eq!(while_held.resident_bytes, 0);
    assert_eq!(while_held.externally_leased_chunks, 1);
    assert_eq!(while_held.externally_leased_bytes, chunk.len());
    assert_eq!(while_held.tracked_payload_chunks, 1);
    assert_eq!(while_held.tracked_payload_bytes, chunk.len());
    assert_eq!(while_held.external_lease_tracking_overflows, 0);

    drop(chunk);
    let after_release = store.chunk_residency_diagnostics().unwrap();
    assert_eq!(after_release.externally_leased_chunks, 0);
    assert_eq!(after_release.externally_leased_bytes, 0);
    assert_eq!(after_release.tracked_payload_chunks, 0);
    assert_eq!(after_release.tracked_payload_bytes, 0);
    assert_eq!(after_release.external_lease_tracking_overflows, 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_trim_releases_cache_owned_payloads_but_preserves_consumer_leases() {
    let root = unique_temp_project_root("artifact_store_trim_chunk_residency");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/trimmed-cache.json").unwrap();
    let asset = ImportedAsset::Data(DataAsset {
        uri: uri.clone(),
        format: DataAssetFormat::Json,
        text: "{\"trimmed\":true}".to_string(),
        canonical_json: serde_json::json!({"trimmed": true}),
    });
    let store = ArtifactStore::with_chunk_residency_budget(64 * 1024);
    let artifact_uri = store
        .write(
            &paths,
            &ResourceRecord::new(AssetId::new(), AssetKind::Data, uri),
            &asset,
        )
        .unwrap();
    let inventory = store.open_chunk_inventory(&paths, &artifact_uri).unwrap();
    let held = store.read_compressed_chunk(&inventory, 0).unwrap();
    let before_trim = store.chunk_residency_diagnostics().unwrap();

    let report = store.trim_chunk_residency().unwrap();
    let after_trim = store.chunk_residency_diagnostics().unwrap();

    assert_eq!(report.released_cache_chunks, before_trim.resident_chunks);
    assert_eq!(report.released_cache_bytes, before_trim.resident_bytes);
    assert_eq!(after_trim.resident_chunks, 0);
    assert_eq!(after_trim.resident_bytes, 0);
    assert_eq!(after_trim.externally_leased_chunks, 1);
    assert_eq!(after_trim.externally_leased_bytes, held.len());
    assert_eq!(after_trim.tracked_payload_chunks, 1);
    assert_eq!(after_trim.tracked_payload_bytes, held.len());

    drop(held);
    let after_release = store.chunk_residency_diagnostics().unwrap();
    assert_eq!(after_release.externally_leased_chunks, 0);
    assert_eq!(after_release.tracked_payload_chunks, 0);

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

use std::collections::BTreeMap;
use std::fs;

use crate::core::framework::render::{RenderShaderDefinitionValue, ShaderAssetKind};
use crate::core::framework::scene::physics::{
    PhysicsJointConstraintMetadata, PhysicsMassProperties, PhysicsMaterialMetadata,
};
use crate::core::resource::ResourceRecord;

use crate::asset::project::ProjectPaths;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    sample_animation_sequence_asset, sample_physics_material_asset,
};
use crate::asset::{
    AlphaMode, ArtifactStore, AssetId, AssetImportError, AssetKind, AssetReference, AssetUri,
    DataAsset, DataAssetFormat, ImportedAsset, MaterialAsset, MeshAsset, MeshAttributeValues,
    MeshIndices, SceneAsset, SceneCameraAsset, SceneCameraTargetAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneEntityAsset, SceneJointAsset, SceneJointKindAsset,
    SceneMobilityAsset, SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneScriptBindingAsset,
    ShaderAsset, ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, TextureAsset, TransformAsset, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::RenderMeshTopology;

mod artifact_cache_assets;
mod binary_payloads;
mod material_data;
mod scene_components;
mod scene_script;

#[derive(serde::Serialize, serde::Deserialize)]
struct ArtifactManifestFixture {
    schema_version: u32,
    kind: AssetKind,
    revision: u64,
    content_hash: String,
    raw_bytes: u64,
    compressed_bytes: u64,
    chunks: Vec<ArtifactChunkFixture>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ArtifactChunkFixture {
    content_hash: String,
    compressed_bytes: u32,
}

#[test]
fn artifact_store_streams_compressed_bytes_into_final_payload() {
    let source = include_str!("../../artifact/store.rs");

    assert!(source.contains("struct ArtifactManifest"));
    assert!(source.contains("struct ChunkReader"));
    assert!(source.contains("atomic_write(&artifact_path"));
    assert!(source.contains("file.metadata()?.len() != expected_bytes"));
    assert!(source.contains("revision: metadata.revision"));
    assert!(source.contains("ARTIFACT_MANIFEST_SCHEMA_VERSION: u32 = 3"));
    assert!(source.contains("ARTIFACT_MANIFEST_MAX_BYTES: usize = 4 * 1024 * 1024"));
    assert!(source.contains(
        "payload.extend_from_slice(&bytes);\n    if payload.len() > ARTIFACT_MANIFEST_MAX_BYTES"
    ));
    assert!(source.contains("ARTIFACT_RAW_PAYLOAD_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024"));
    assert!(source.contains(".with_limit(expected_raw_bytes)"));
    assert!(source.contains("struct RawPayloadLimitWriter"));
    assert!(source.contains("bincode::serialize_into(&mut raw_writer, cache_asset)"));
    assert!(source.contains("let raw_bytes = raw_writer.bytes_written();"));
    assert!(source.contains("if raw_writer.limit_exceeded()"));
    assert!(source.contains(
        "validate_artifact_compressed_payload_bytes(manifest.raw_bytes, compressed_bytes)?;"
    ));
    assert!(source
        .contains("validate_artifact_compressed_payload_bytes(raw_bytes, compressed_bytes)?;"));
    assert!(!source.contains("bincode::serialized_size(cache_asset)"));
    assert!(!source.contains("zstd::stream::encode_all(&bytes"));
    assert!(!source.contains("fs::read(&path)"));
}

fn assert_binary_artifact_payload(paths: &ProjectPaths, artifact_uri: &AssetUri) {
    let payload = fs::read(paths.asset_artifact_root().join(artifact_uri.path())).unwrap();
    assert!(payload.starts_with(b"ZRARTM03"));
    assert!(paths.asset_artifact_root().join("chunks").is_dir());
}

#[test]
fn artifact_store_manifest_records_resource_revision() {
    let root = unique_temp_project_root("artifact_store_manifest_resource_revision");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let asset = ImportedAsset::Data(DataAsset {
        uri: AssetUri::parse("res://data/revision.json").unwrap(),
        format: DataAssetFormat::Json,
        text: "{\"revision\":47}".to_string(),
        canonical_json: serde_json::json!({"revision": 47}),
    });
    let mut record = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Data,
        AssetUri::parse("res://data/revision.json").unwrap(),
    );
    record.revision = 47;

    let artifact_uri = ArtifactStore::default()
        .write(&paths, &record, &asset)
        .unwrap();
    let payload = fs::read(paths.asset_artifact_root().join(artifact_uri.path())).unwrap();
    let manifest: ArtifactManifestFixture = bincode::deserialize(&payload[8..]).unwrap();

    assert_eq!(manifest.schema_version, 3);
    assert_eq!(manifest.revision, record.revision);

    record.revision = 48;
    let refreshed_uri = ArtifactStore::default()
        .write(&paths, &record, &asset)
        .unwrap();
    let refreshed_payload =
        fs::read(paths.asset_artifact_root().join(refreshed_uri.path())).unwrap();
    let refreshed_manifest: ArtifactManifestFixture =
        bincode::deserialize(&refreshed_payload[8..]).unwrap();

    assert_eq!(refreshed_uri, artifact_uri);
    assert_eq!(refreshed_manifest.revision, record.revision);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_previous_manifest_magic_without_a_compatibility_decoder() {
    let root = unique_temp_project_root("artifact_store_rejects_previous_manifest_magic");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let asset = ImportedAsset::Data(DataAsset {
        uri: AssetUri::parse("res://data/legacy-magic.json").unwrap(),
        format: DataAssetFormat::Json,
        text: "{\"legacy\":false}".to_string(),
        canonical_json: serde_json::json!({"legacy": false}),
    });
    let record = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Data,
        AssetUri::parse("res://data/legacy-magic.json").unwrap(),
    );
    let store = ArtifactStore::default();
    let artifact_uri = store.write(&paths, &record, &asset).unwrap();
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    let mut payload = fs::read(&manifest_path).unwrap();
    payload[..8].copy_from_slice(b"ZRARTM02");
    fs::write(manifest_path, payload).unwrap();

    assert!(store.read(&paths, &artifact_uri).is_err());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_shares_content_addressed_chunks_across_manifests() {
    let root = unique_temp_project_root("artifact_store_content_addressed_chunks");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let asset = ImportedAsset::Data(DataAsset {
        uri: AssetUri::parse("res://data/shared.json").unwrap(),
        format: DataAssetFormat::Json,
        text: "{\"shared\":true}".to_string(),
        canonical_json: serde_json::json!({"shared": true}),
    });
    let store = ArtifactStore::default();
    let first = store
        .write(
            &paths,
            &ResourceRecord::new(
                AssetId::new(),
                AssetKind::Data,
                AssetUri::parse("res://data/first.json").unwrap(),
            ),
            &asset,
        )
        .unwrap();
    let second = store
        .write(
            &paths,
            &ResourceRecord::new(
                AssetId::new(),
                AssetKind::Data,
                AssetUri::parse("res://data/second.json").unwrap(),
            ),
            &asset,
        )
        .unwrap();

    assert_binary_artifact_payload(&paths, &first);
    assert_binary_artifact_payload(&paths, &second);
    let chunks = fs::read_dir(paths.asset_artifact_root().join("chunks"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(chunks.len(), 1, "identical payloads should share one chunk");
    assert_eq!(store.read(&paths, &first).unwrap(), asset);
    assert_eq!(store.read(&paths, &second).unwrap(), asset);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_streams_chunk_boundaries_without_reassembling_the_payload() {
    let root = unique_temp_project_root("artifact_store_chunk_boundaries");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let mut state = 0x9e37_79b9_u32;
    let rgba = (0..(384 * 192 * 4))
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect();
    let asset = ImportedAsset::Texture(TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/chunked.png").unwrap(),
        384,
        192,
        rgba,
    ));
    let artifact_uri = ArtifactStore::default()
        .write(
            &paths,
            &ResourceRecord::new(
                AssetId::new(),
                AssetKind::Texture,
                AssetUri::parse("res://textures/chunked.png").unwrap(),
            ),
            &asset,
        )
        .unwrap();

    let chunk_count = fs::read_dir(paths.asset_artifact_root().join("chunks"))
        .unwrap()
        .count();
    assert!(
        chunk_count > 1,
        "test payload should cross chunk boundaries"
    );
    assert_eq!(
        ArtifactStore::default()
            .read(&paths, &artifact_uri)
            .unwrap(),
        asset
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_corrupted_content_addressed_chunks() {
    let root = unique_temp_project_root("artifact_store_corrupted_chunk");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let asset = ImportedAsset::Data(DataAsset {
        uri: AssetUri::parse("res://data/corrupted.json").unwrap(),
        format: DataAssetFormat::Json,
        text: "{\"value\":42}".to_string(),
        canonical_json: serde_json::json!({"value": 42}),
    });
    let store = ArtifactStore::default();
    let record = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Data,
        AssetUri::parse("res://data/corrupted.json").unwrap(),
    );
    let artifact_uri = store.write(&paths, &record, &asset).unwrap();
    let chunk_path = fs::read_dir(paths.asset_artifact_root().join("chunks"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(chunk_path, vec![0_u8; 128 * 1024]).unwrap();

    assert!(store.read(&paths, &artifact_uri).is_err());
    store.write(&paths, &record, &asset).unwrap();
    assert_eq!(store.read(&paths, &artifact_uri).unwrap(), asset);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_oversized_manifest_before_deserialization() {
    let root = unique_temp_project_root("artifact_store_oversized_manifest");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let artifact_uri = AssetUri::parse("lib://data/oversized.zasset").unwrap();
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    let mut payload = b"ZRARTM03".to_vec();
    payload.resize(4 * 1024 * 1024 + 1, 0);
    fs::write(manifest_path, payload).unwrap();

    assert!(matches!(
        ArtifactStore::default().read(&paths, &artifact_uri),
        Err(AssetImportError::Parse(_))
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_manifest_budget_covers_maximum_chunk_inventory() {
    let raw_bytes = 2_u64 * 1024 * 1024 * 1024;
    let chunk_bytes = 64_u64 * 1024;
    // Zstd's documented compression bound permits a one-in-256 expansion.
    let compressed_bound_bytes = raw_bytes + raw_bytes / 256;
    let chunk_count = compressed_bound_bytes.div_ceil(chunk_bytes) as usize;
    let chunk_hash = blake3::hash(b"chunk").to_hex().to_string();
    let manifest = ArtifactManifestFixture {
        schema_version: 3,
        kind: AssetKind::Data,
        revision: 0,
        content_hash: chunk_hash.clone(),
        raw_bytes,
        compressed_bytes: compressed_bound_bytes,
        chunks: (0..chunk_count)
            .map(|_| ArtifactChunkFixture {
                content_hash: chunk_hash.clone(),
                compressed_bytes: chunk_bytes as u32,
            })
            .collect(),
    };
    let manifest_bytes = bincode::serialize(&manifest).unwrap().len() + b"ZRARTM03".len();

    assert!(
        manifest_bytes <= 4 * 1024 * 1024,
        "maximum chunk inventory must fit the bounded artifact manifest"
    );
}

#[test]
fn artifact_store_rejects_zero_length_manifest_payloads_before_chunk_io() {
    let root = unique_temp_project_root("artifact_store_zero_length_payload");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let artifact_uri = AssetUri::parse("lib://data/zero-length.zasset").unwrap();
    let chunk_hash = blake3::hash(b"x").to_hex().to_string();
    let manifest = ArtifactManifestFixture {
        schema_version: 3,
        kind: AssetKind::Data,
        revision: 0,
        content_hash: chunk_hash.clone(),
        raw_bytes: 0,
        compressed_bytes: 1,
        chunks: vec![ArtifactChunkFixture {
            content_hash: chunk_hash,
            compressed_bytes: 1,
        }],
    };
    let mut payload = b"ZRARTM03".to_vec();
    payload.extend_from_slice(&bincode::serialize(&manifest).unwrap());
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, payload).unwrap();

    assert!(matches!(
        ArtifactStore::default().read(&paths, &artifact_uri),
        Err(AssetImportError::Parse(_))
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_manifest_raw_payloads_over_the_runtime_budget() {
    let root = unique_temp_project_root("artifact_store_raw_payload_budget");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let artifact_uri = AssetUri::parse("lib://data/raw-budget.zasset").unwrap();
    let chunk_hash = blake3::hash(b"x").to_hex().to_string();
    let manifest = ArtifactManifestFixture {
        schema_version: 3,
        kind: AssetKind::Data,
        revision: 0,
        content_hash: chunk_hash.clone(),
        raw_bytes: u64::MAX,
        compressed_bytes: 1,
        chunks: vec![ArtifactChunkFixture {
            content_hash: chunk_hash,
            compressed_bytes: 1,
        }],
    };
    let mut payload = b"ZRARTM03".to_vec();
    payload.extend_from_slice(&bincode::serialize(&manifest).unwrap());
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, payload).unwrap();

    assert!(matches!(
        ArtifactStore::default().read(&paths, &artifact_uri),
        Err(AssetImportError::Parse(_))
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_manifest_compressed_payloads_over_the_zstd_bound_before_chunk_io() {
    let root = unique_temp_project_root("artifact_store_compressed_payload_budget");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let artifact_uri = AssetUri::parse("lib://data/compressed-budget.zasset").unwrap();
    let chunk_hash = blake3::hash(b"x").to_hex().to_string();
    let manifest = ArtifactManifestFixture {
        schema_version: 3,
        kind: AssetKind::Data,
        revision: 0,
        content_hash: chunk_hash.clone(),
        raw_bytes: 1,
        compressed_bytes: 65,
        chunks: vec![ArtifactChunkFixture {
            content_hash: chunk_hash,
            compressed_bytes: 65,
        }],
    };
    let mut payload = b"ZRARTM03".to_vec();
    payload.extend_from_slice(&bincode::serialize(&manifest).unwrap());
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, payload).unwrap();

    assert!(matches!(
        ArtifactStore::default().read(&paths, &artifact_uri),
        Err(AssetImportError::Parse(_))
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_verifies_final_manifest_chunk_after_payload_deserializes() {
    let root = unique_temp_project_root("artifact_store_final_chunk_verification");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let asset = ImportedAsset::Data(DataAsset {
        uri: AssetUri::parse("res://data/final-chunk.json").unwrap(),
        format: DataAssetFormat::Json,
        text: "{\"value\":42}".to_string(),
        canonical_json: serde_json::json!({"value": 42}),
    });
    let store = ArtifactStore::default();
    let artifact_uri = store
        .write(
            &paths,
            &ResourceRecord::new(
                AssetId::new(),
                AssetKind::Data,
                AssetUri::parse("res://data/final-chunk.json").unwrap(),
            ),
            &asset,
        )
        .unwrap();
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    let payload = fs::read(&manifest_path).unwrap();
    let mut manifest: ArtifactManifestFixture = bincode::deserialize(&payload[8..]).unwrap();
    let chunk_root = paths.asset_artifact_root().join("chunks");
    let mut compressed_payload = Vec::new();
    for chunk in &manifest.chunks {
        compressed_payload
            .extend(fs::read(chunk_root.join(format!("{}.zchunk", chunk.content_hash))).unwrap());
    }

    let empty_frame = zstd::stream::encode_all(&[][..], 1).unwrap();
    let final_chunk_hash = blake3::hash(&empty_frame).to_hex().to_string();
    let final_chunk_path = chunk_root.join(format!("{final_chunk_hash}.zchunk"));
    fs::write(&final_chunk_path, &empty_frame).unwrap();
    compressed_payload.extend_from_slice(&empty_frame);
    manifest.content_hash = blake3::hash(&compressed_payload).to_hex().to_string();
    manifest.compressed_bytes += empty_frame.len() as u64;
    manifest.chunks.push(ArtifactChunkFixture {
        content_hash: final_chunk_hash,
        compressed_bytes: empty_frame.len() as u32,
    });
    let mut rewritten = b"ZRARTM03".to_vec();
    rewritten.extend_from_slice(&bincode::serialize(&manifest).unwrap());
    fs::write(&manifest_path, rewritten).unwrap();

    assert_eq!(store.read(&paths, &artifact_uri).unwrap(), asset);
    fs::write(final_chunk_path, b"corrupt final chunk").unwrap();
    assert!(store.read(&paths, &artifact_uri).is_err());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_metadata_kind_mismatches_before_publication() {
    let root = unique_temp_project_root("artifact_store_metadata_kind_mismatch");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let asset = ImportedAsset::Data(DataAsset {
        uri: AssetUri::parse("res://data/mismatch.json").unwrap(),
        format: DataAssetFormat::Json,
        text: "{\"value\":42}".to_string(),
        canonical_json: serde_json::json!({"value": 42}),
    });

    let store = ArtifactStore::default();
    let record = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Data,
        AssetUri::parse("res://data/mismatch.json").unwrap(),
    );
    let artifact_uri = store.write(&paths, &record, &asset).unwrap();
    let mismatched_asset = ImportedAsset::Texture(TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/mismatch.png").unwrap(),
        1,
        1,
        vec![255, 255, 255, 255],
    ));

    assert!(matches!(
        store.write(&paths, &record, &mismatched_asset),
        Err(AssetImportError::Parse(_))
    ));
    assert_eq!(store.read(&paths, &artifact_uri).unwrap(), asset);
    assert!(
        !paths.asset_artifact_root().join("textures").exists(),
        "mismatched metadata must fail before publishing an artifact generation"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_manifest_chunk_ids_that_escape_the_chunk_root() {
    let root = unique_temp_project_root("artifact_store_malicious_chunk_id");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let artifact_uri = AssetUri::parse("lib://data/malicious.zasset").unwrap();
    let manifest = ArtifactManifestFixture {
        schema_version: 3,
        kind: AssetKind::Data,
        revision: 0,
        content_hash: blake3::hash(b"manifest-content").to_hex().to_string(),
        raw_bytes: 1,
        compressed_bytes: 1,
        chunks: vec![ArtifactChunkFixture {
            content_hash: "../outside".to_string(),
            compressed_bytes: 1,
        }],
    };
    let mut payload = b"ZRARTM03".to_vec();
    payload.extend_from_slice(&bincode::serialize(&manifest).unwrap());
    let manifest_path = paths.asset_artifact_root().join(artifact_uri.path());
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, payload).unwrap();

    assert!(ArtifactStore::default()
        .read(&paths, &artifact_uri)
        .is_err());

    let _ = fs::remove_dir_all(root);
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

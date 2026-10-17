use super::*;

#[test]
fn pre_scene_component_bincode_cache_manifest_is_rejected_for_rebuild() {
    let manifest = ArtifactManifest {
        schema_version: PRE_SCENE_COMPONENT_MANIFEST_SCHEMA_VERSION,
        kind: AssetKind::Scene,
        revision: 1,
        content_hash: "a".repeat(BLAKE3_HEX_LENGTH),
        raw_bytes: 1,
        compressed_bytes: 1,
        chunks: vec![ArtifactChunkDescriptor::new(
            Arc::from("b".repeat(BLAKE3_HEX_LENGTH)),
            1,
        )],
    };
    let mut old_wire = PRE_SCENE_COMPONENT_MANIFEST_MAGIC.to_vec();
    let old_payload = PositionalEncoding::FixedLittleEndian
        .encode(&manifest)
        .unwrap();
    old_wire.extend(old_payload);
    let decoded = PositionalEncoding::FixedLittleEndian
        .decode::<ArtifactManifest>(&old_wire[PRE_SCENE_COMPONENT_MANIFEST_MAGIC.len()..])
        .unwrap();
    let error = validate_manifest("scenes/legacy.zasset", &decoded)
        .expect_err("schema 6 positional scene payload must require rebuild");
    assert!(error.to_string().contains("must be rebuilt"));
    assert_eq!(
        &old_wire[..PRE_SCENE_COMPONENT_MANIFEST_MAGIC.len()],
        PRE_SCENE_COMPONENT_MANIFEST_MAGIC
    );
}

#[test]
fn artifact_zstd_compress_bound_large_payload_accepts_exact_bound() {
    let raw_bytes = ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES + 1;
    let compressed_bound = raw_bytes + raw_bytes / 256;

    assert!(validate_artifact_compressed_payload_bytes(raw_bytes, compressed_bound).is_ok());
}

#[test]
fn artifact_zstd_compress_bound_large_payload_rejects_bytes_above_bound() {
    let raw_bytes = ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES + 1;
    let compressed_bound = raw_bytes + raw_bytes / 256;

    let error = validate_artifact_compressed_payload_bytes(raw_bytes, compressed_bound + 1)
        .expect_err("bytes above the Zstd bound must be rejected");
    let AssetImportError::Parse(message) = error else {
        panic!("expected a parse error for an oversized compressed payload");
    };
    assert!(message.contains("exceeds the"));
}

#[test]
fn scene_artifact_manifest_has_stable_schema_codec_and_content_identity() {
    let root = std::env::temp_dir().join(format!(
        "zircon_scene_artifact_manifest_golden_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let metadata = ResourceRecord::new(
        crate::asset::AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/canonical.scene.toml").unwrap(),
    );
    let scene = ImportedAsset::Scene(crate::asset::SceneAsset {
        entities: Vec::new(),
    });
    let store = ArtifactStore::default();

    let artifact_uri = store.write(&paths, &metadata, &scene).unwrap();
    let artifact_path = resolve_artifact_cache_path(&paths, &artifact_uri).unwrap();
    let manifest = read_manifest(&artifact_path).unwrap();
    let prepared = store.prepare_write(&paths, &metadata, &scene).unwrap();

    assert_eq!(
        &prepared.payload[..ARTIFACT_MANIFEST_MAGIC.len()],
        ARTIFACT_MANIFEST_MAGIC
    );
    assert_eq!(manifest.schema_version, ARTIFACT_MANIFEST_SCHEMA_VERSION);
    assert_eq!(manifest.kind, AssetKind::Scene);
    assert_eq!(manifest.content_hash.len(), BLAKE3_HEX_LENGTH);
    assert!(manifest
        .content_hash
        .chars()
        .all(|character| character.is_ascii_hexdigit()));
    assert_eq!(manifest.raw_bytes, prepared.raw_bytes);
    assert_eq!(manifest.compressed_bytes, prepared.compressed_bytes);
    assert_eq!(manifest.chunks.len(), prepared.chunk_count);
    assert_eq!(store.read(&paths, &artifact_uri).unwrap(), scene);

    let prepared_again = store.prepare_write(&paths, &metadata, &scene).unwrap();
    let manifest_again = read_manifest(&artifact_path).unwrap();
    assert_eq!(prepared.payload, prepared_again.payload);
    assert_eq!(manifest.content_hash, manifest_again.content_hash);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_stream_serialization_preserves_the_last_model_cache_and_removes_staging() {
    struct FailsAfterPrefix;
    impl serde::Serialize for FailsAfterPrefix {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            use serde::ser::SerializeSeq;
            let mut sequence = serializer.serialize_seq(Some(2))?;
            sequence.serialize_element(&[19_u8; 1024].as_slice())?;
            Err(<S::Error as serde::ser::Error>::custom(
                "injected stream failure",
            ))
        }
    }
    let root = crate::asset::tests::project::unique_temp_project_root("model_cache_stream_failure");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://models/last-valid.zmodel").unwrap();
    let metadata = ResourceRecord::new(crate::asset::AssetId::new(), AssetKind::Model, uri.clone());
    let asset = ImportedAsset::Model(crate::asset::ModelAsset {
        uri,
        primitives: Vec::new(),
    });
    let store = ArtifactStore::default();
    let locator = store.write(&paths, &metadata, &asset).unwrap();
    let manifest = resolve_artifact_cache_path(&paths, &locator).unwrap();
    let original = fs::read(&manifest).unwrap();

    assert!(StagedArtifactPayload::write(&paths.asset_artifact_root(), &FailsAfterPrefix).is_err());
    assert_eq!(fs::read(&manifest).unwrap(), original);
    assert_eq!(store.read(&paths, &locator).unwrap(), asset);
    assert_eq!(
        fs::read_dir(paths.asset_artifact_root().join(ARTIFACT_STAGING_DIRECTORY))
            .unwrap()
            .count(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}

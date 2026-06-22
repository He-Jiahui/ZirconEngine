use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::asset::pack::{
    zrpack_content_hash, ZrPackAssetEntry, ZrPackDeltaDocumentManifest, ZrPackDeltaInstallError,
    ZrPackDeltaInstaller, ZrPackDeltaReader, ZrPackDeltaWriter, ZrPackDocumentManifest,
    ZrPackError, ZrPackInputAsset, ZrPackPromotionMethod, ZrPackReader, ZrPackTrimConfig,
    ZrPackTrimInputAsset, ZrPackTrimPlanner, ZrPackTrimReason, ZrPackWriter, ZRPACK_DELTA_MAGIC,
    ZRPACK_FORMAT_VERSION, ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION, ZRPACK_MAGIC,
};
use crate::core::framework::net::{ZrChunkEntry, ZrPackManifest};

#[test]
fn pack_round_trip() {
    let report = ZrPackWriter::write([
        ZrPackInputAsset::new("textures/albedo.bin", b"albedo".to_vec()),
        ZrPackInputAsset::new("meshes/cube.bin", b"cube".to_vec()),
    ])
    .unwrap();
    let reader = ZrPackReader::from_bytes(report.bytes).unwrap();

    assert_eq!(reader.read_asset("textures/albedo.bin").unwrap(), b"albedo");
    assert_eq!(reader.read_asset("meshes/cube.bin").unwrap(), b"cube");
    assert_eq!(reader.manifest().pack.chunks.len(), 2);
    assert!(reader.manifest().pack.is_complete_byte_plan());
}

#[test]
fn duplicate_content_stored_once() {
    let report = ZrPackWriter::write([
        ZrPackInputAsset::new("a/same.bin", b"same-bytes".to_vec()),
        ZrPackInputAsset::new("b/same.bin", b"same-bytes".to_vec()),
        ZrPackInputAsset::new("c/other.bin", b"other".to_vec()),
    ])
    .unwrap();
    let reader = ZrPackReader::from_bytes(report.bytes).unwrap();

    assert_eq!(reader.manifest().assets.len(), 3);
    assert_eq!(reader.manifest().pack.chunks.len(), 2);
    assert_eq!(report.deduplicated_assets, ["b/same.bin"]);
    assert_eq!(reader.read_asset("a/same.bin").unwrap(), b"same-bytes");
    assert_eq!(reader.read_asset("b/same.bin").unwrap(), b"same-bytes");
}

#[test]
fn deterministic_pack_double_run_byte_identical() {
    let first = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/cube.bin", b"cube".to_vec()),
        ZrPackInputAsset::new("textures/albedo.bin", b"albedo".to_vec()),
    ])
    .unwrap();
    let second = ZrPackWriter::write([
        ZrPackInputAsset::new("textures/albedo.bin", b"albedo".to_vec()),
        ZrPackInputAsset::new("meshes/cube.bin", b"cube".to_vec()),
    ])
    .unwrap();

    assert_eq!(first.bytes, second.bytes);
    assert!(first.manifest.pack.is_complete_byte_plan());
    assert!(second.manifest.pack.is_complete_byte_plan());
}

#[test]
fn pack_writer_rejects_unsafe_asset_paths() {
    let cases = [
        "",
        " ",
        "/textures/hero.bin",
        "../escape.bin",
        "textures//hero.bin",
        "textures/./hero.bin",
        "textures/../hero.bin",
        "C:/absolute.bin",
    ];

    for path in cases {
        let error =
            ZrPackWriter::write([ZrPackInputAsset::new(path, b"data".to_vec())]).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("zrpack asset path {path} must be a safe relative asset path")
        );
    }
}

#[test]
fn pack_writer_rejects_unnormalized_asset_paths() {
    let cases = [
        (" textures/hero.bin ", "textures/hero.bin"),
        ("textures\\hero.bin", "textures/hero.bin"),
    ];

    for (path, normalized) in cases {
        let error =
            ZrPackWriter::write([ZrPackInputAsset::new(path, b"data".to_vec())]).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "zrpack asset path {path} must use normalized relative asset path {normalized}"
            )
        );
    }
}

#[test]
fn pack_reader_rejects_manifest_asset_path_schema() {
    let cases = [
        (
            "../escape.bin",
            "zrpack asset path ../escape.bin must be a safe relative asset path".to_string(),
        ),
        (
            "textures\\hero.bin",
            "zrpack asset path textures\\hero.bin must use normalized relative asset path textures/hero.bin"
                .to_string(),
        ),
    ];

    for (path, expected) in cases {
        let bytes = malformed_pack_bytes_with_assets(vec![pack_asset_entry(path)]);

        let error = ZrPackReader::from_bytes(bytes).unwrap_err();

        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn pack_reader_rejects_duplicate_manifest_asset_paths() {
    let bytes = malformed_pack_bytes_with_assets(vec![
        pack_asset_entry("scenes/main.zscene"),
        pack_asset_entry("scenes/main.zscene"),
    ]);

    let error = ZrPackReader::from_bytes(bytes).unwrap_err();

    assert_eq!(
        error,
        ZrPackError::DuplicateAssetPath("scenes/main.zscene".to_string())
    );
}

#[test]
fn pack_reader_rejects_unsorted_manifest_asset_paths() {
    let bytes = malformed_pack_bytes_with_assets(vec![
        pack_asset_entry("textures/hero.png"),
        pack_asset_entry("scenes/main.zscene"),
    ]);

    let error = ZrPackReader::from_bytes(bytes).unwrap_err();

    assert_eq!(
        error.to_string(),
        "zrpack asset paths must be sorted by asset path"
    );
}

#[test]
fn pack_reader_rejects_manifest_pack_version_mismatch() {
    let mut manifest =
        pack_document_manifest_with_assets(vec![pack_asset_entry("textures/hero.bin")]);
    manifest.pack.version = ZRPACK_FORMAT_VERSION + 1;

    let error = ZrPackReader::from_bytes(malformed_pack_bytes(manifest)).unwrap_err();

    assert_eq!(
        error,
        ZrPackError::UnsupportedVersion(ZRPACK_FORMAT_VERSION + 1)
    );
}

#[test]
fn pack_reader_rejects_manifest_chunk_table_shape() {
    let mut duplicate = pack_document_manifest_with_assets(vec![
        pack_asset_entry_with_payload("meshes/a.bin", b"a"),
        pack_asset_entry_with_payload("textures/b.bin", b"b"),
    ]);
    duplicate.pack.chunks.push(duplicate.pack.chunks[0].clone());
    let duplicate_error = ZrPackReader::from_bytes(malformed_pack_bytes(duplicate)).unwrap_err();
    assert_eq!(duplicate_error, ZrPackError::DuplicateChunkHash);

    let mut unsorted = pack_document_manifest_with_assets(vec![
        pack_asset_entry_with_payload("meshes/a.bin", b"a"),
        pack_asset_entry_with_payload("textures/b.bin", b"b"),
    ]);
    unsorted.pack.chunks.reverse();
    let unsorted_error = ZrPackReader::from_bytes(malformed_pack_bytes(unsorted)).unwrap_err();
    assert_eq!(unsorted_error, ZrPackError::UnsortedChunkHashes);
}

#[test]
fn pack_reader_rejects_manifest_total_size_mismatch() {
    let mut manifest =
        pack_document_manifest_with_assets(vec![pack_asset_entry("textures/hero.bin")]);
    manifest.pack.total_size += 1;

    let error = ZrPackReader::from_bytes(malformed_pack_bytes(manifest)).unwrap_err();

    assert_eq!(error, ZrPackError::PackTotalSizeMismatch);
}

#[test]
fn pack_reader_rejects_manifest_asset_chunk_mismatch() {
    let mut missing_chunk =
        pack_document_manifest_with_assets(vec![pack_asset_entry("textures/hero.bin")]);
    missing_chunk.assets[0].chunk_hash = zrpack_content_hash(b"missing");
    let missing_error = ZrPackReader::from_bytes(malformed_pack_bytes(missing_chunk)).unwrap_err();
    assert_eq!(
        missing_error,
        ZrPackError::MissingChunk("textures/hero.bin".to_string())
    );

    let mut size_mismatch =
        pack_document_manifest_with_assets(vec![pack_asset_entry("textures/hero.bin")]);
    size_mismatch.assets[0].size += 1;
    let size_error = ZrPackReader::from_bytes(malformed_pack_bytes(size_mismatch)).unwrap_err();
    assert_eq!(
        size_error,
        ZrPackError::ChunkOutOfBounds("textures/hero.bin".to_string())
    );
}

#[test]
fn pack_reader_rejects_manifest_extra_unreferenced_chunks() {
    let mut manifest =
        pack_document_manifest_with_assets(vec![pack_asset_entry("textures/hero.bin")]);
    manifest.pack.chunks.push(ZrChunkEntry::new(
        zrpack_content_hash(b"extra"),
        ZRPACK_TEST_HEADER_SIZE as u64 + manifest.pack.total_size,
        5,
    ));
    manifest
        .pack
        .chunks
        .sort_by(|left, right| left.hash.cmp(&right.hash));
    manifest.pack.total_size = total_chunk_size(&manifest.pack.chunks);

    let error = ZrPackReader::from_bytes(malformed_pack_bytes(manifest)).unwrap_err();

    assert_eq!(error, ZrPackError::PackChunkTableMismatch);
}

#[test]
fn pack_reader_rejects_chunk_payload_hash_mismatch() {
    let report =
        ZrPackWriter::write([ZrPackInputAsset::new("textures/hero.bin", b"hero".to_vec())])
            .unwrap();
    let mut bytes = report.bytes;
    bytes[ZRPACK_TEST_HEADER_SIZE] ^= 0xff;

    let error = ZrPackReader::from_bytes(bytes).unwrap_err();

    assert_eq!(
        error,
        ZrPackError::ChunkHashMismatch("textures/hero.bin".to_string())
    );
}

#[test]
fn pack_reader_rejects_payload_manifest_gap() {
    let report =
        ZrPackWriter::write([ZrPackInputAsset::new("textures/hero.bin", b"hero".to_vec())])
            .unwrap();
    let bytes = bytes_with_manifest_gap(report.bytes, b"gap");

    let error = ZrPackReader::from_bytes(bytes).unwrap_err();

    assert_eq!(error, ZrPackError::PayloadExtentMismatch);
}

#[test]
fn pack_reader_rejects_manifest_trailing_bytes() {
    let report =
        ZrPackWriter::write([ZrPackInputAsset::new("textures/hero.bin", b"hero".to_vec())])
            .unwrap();
    let bytes = bytes_with_manifest_trailing_bytes(report.bytes, b"trail");

    let error = ZrPackReader::from_bytes(bytes).unwrap_err();

    assert_eq!(error, ZrPackError::ManifestTrailingBytes);
}

#[test]
fn delta_reader_rejects_nested_pack_manifest_asset_path_schema() {
    let cases = [
        (
            vec![pack_asset_entry("../base.bin")],
            Vec::new(),
            "zrpack asset path ../base.bin must be a safe relative asset path".to_string(),
        ),
        (
            Vec::new(),
            vec![pack_asset_entry("textures\\target.bin")],
            "zrpack asset path textures\\target.bin must use normalized relative asset path textures/target.bin"
                .to_string(),
        ),
    ];

    for (base_assets, target_assets, expected) in cases {
        let manifest =
            delta_manifest_with_assets(base_assets, target_assets, Vec::new(), Vec::new());

        let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn delta_reader_rejects_changed_asset_path_schema() {
    let manifest = delta_manifest_with_assets(
        Vec::new(),
        Vec::new(),
        vec![pack_asset_entry("textures\\changed.bin")],
        Vec::new(),
    );

    let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

    assert_eq!(
        error.to_string(),
        "zrpack asset path textures\\changed.bin must use normalized relative asset path textures/changed.bin"
    );
}

#[test]
fn delta_reader_rejects_removed_asset_path_schema() {
    let manifest = delta_manifest_with_assets(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec!["../removed.bin".to_string()],
    );

    let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

    assert_eq!(
        error.to_string(),
        "zrpack asset path ../removed.bin must be a safe relative asset path"
    );
}

#[test]
fn delta_reader_rejects_duplicate_changed_and_removed_asset_paths() {
    let cases = [
        (
            delta_manifest_with_assets(
                Vec::new(),
                Vec::new(),
                vec![
                    pack_asset_entry("textures/changed.bin"),
                    pack_asset_entry("textures/changed.bin"),
                ],
                Vec::new(),
            ),
            "textures/changed.bin",
        ),
        (
            delta_manifest_with_assets(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![
                    "textures/removed.bin".to_string(),
                    "textures/removed.bin".to_string(),
                ],
            ),
            "textures/removed.bin",
        ),
    ];

    for (manifest, expected_path) in cases {
        let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

        assert_eq!(
            error,
            ZrPackError::DuplicateAssetPath(expected_path.to_string())
        );
    }
}

#[test]
fn delta_reader_rejects_unsorted_changed_and_removed_asset_paths() {
    let cases = [
        delta_manifest_with_assets(
            Vec::new(),
            Vec::new(),
            vec![
                pack_asset_entry("textures/z.bin"),
                pack_asset_entry("textures/a.bin"),
            ],
            Vec::new(),
        ),
        delta_manifest_with_assets(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec!["textures/z.bin".to_string(), "textures/a.bin".to_string()],
        ),
    ];

    for manifest in cases {
        let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

        assert_eq!(
            error.to_string(),
            "zrpack asset paths must be sorted by asset path"
        );
    }
}

#[test]
fn delta_reader_rejects_delta_manifest_format_version_mismatch() {
    let mut manifest = delta_manifest_with_assets(Vec::new(), Vec::new(), Vec::new(), Vec::new());
    manifest.format_version = ZRPACK_FORMAT_VERSION + 1;

    let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

    assert_eq!(
        error,
        ZrPackError::UnsupportedVersion(ZRPACK_FORMAT_VERSION + 1)
    );
}

#[test]
fn delta_reader_rejects_removed_asset_set_mismatch() {
    let keep = pack_asset_entry_with_payload("meshes/keep.bin", b"keep");
    let removed = pack_asset_entry_with_payload("textures/removed.bin", b"removed");
    let manifest = delta_manifest_with_assets(
        vec![keep.clone(), removed],
        vec![keep],
        Vec::new(),
        Vec::new(),
    );

    let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

    assert_eq!(error, ZrPackError::DeltaRemovedAssetsMismatch);
}

#[test]
fn delta_reader_rejects_changed_asset_set_mismatch() {
    let keep = pack_asset_entry_with_payload("meshes/keep.bin", b"keep");
    let changed = pack_asset_entry_with_payload("textures/changed.bin", b"new");
    let manifest = delta_manifest_with_assets(
        vec![keep.clone()],
        vec![keep, changed],
        Vec::new(),
        Vec::new(),
    );

    let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

    assert_eq!(error, ZrPackError::DeltaChangedAssetsMismatch);
}

#[test]
fn delta_reader_rejects_delta_chunk_table_mismatch() {
    let changed = pack_asset_entry_with_payload("textures/changed.bin", b"new");
    let mut manifest =
        delta_manifest_with_assets(Vec::new(), vec![changed.clone()], vec![changed], Vec::new());
    manifest.chunks.clear();

    let error = ZrPackDeltaReader::from_bytes(malformed_delta_bytes(manifest)).unwrap_err();

    assert_eq!(error, ZrPackError::DeltaChunkTableMismatch);
}

#[test]
fn delta_reader_rejects_changed_chunk_payload_hash_mismatch() {
    let base = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    let target = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"new".to_vec(),
    )])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    let mut bytes = delta.bytes;
    bytes[ZRPACK_TEST_HEADER_SIZE] ^= 0xff;

    let error = ZrPackDeltaReader::from_bytes(bytes).unwrap_err();

    assert_eq!(
        error,
        ZrPackError::ChunkHashMismatch("textures/changed.bin".to_string())
    );
}

#[test]
fn delta_reader_rejects_payload_manifest_gap() {
    let base = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    let target = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"new".to_vec(),
    )])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    let bytes = bytes_with_manifest_gap(delta.bytes, b"gap");

    let error = ZrPackDeltaReader::from_bytes(bytes).unwrap_err();

    assert_eq!(error, ZrPackError::PayloadExtentMismatch);
}

#[test]
fn delta_reader_rejects_manifest_trailing_bytes() {
    let base = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    let target = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"new".to_vec(),
    )])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    let bytes = bytes_with_manifest_trailing_bytes(delta.bytes, b"trail");

    let error = ZrPackDeltaReader::from_bytes(bytes).unwrap_err();

    assert_eq!(error, ZrPackError::ManifestTrailingBytes);
}

#[test]
fn delta_pack_contains_only_changed_chunks() {
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("meshes/reused-source.bin", b"reused".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
        ZrPackInputAsset::new("textures/removed.bin", b"removed".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/added.bin", b"added".to_vec()),
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("meshes/reused-alias.bin", b"reused".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();

    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    let delta_reader = ZrPackDeltaReader::from_bytes(delta.bytes).unwrap();

    assert_eq!(
        delta.changed_assets,
        ["meshes/added.bin", "textures/changed.bin"]
    );
    assert_eq!(
        delta.removed_assets,
        ["meshes/reused-source.bin", "textures/removed.bin"]
    );
    assert_eq!(
        delta.reused_assets,
        ["meshes/keep.bin", "meshes/reused-alias.bin"]
    );
    assert_eq!(delta.manifest.chunks.len(), 2);
    assert_eq!(
        delta_reader
            .read_changed_asset("textures/changed.bin")
            .unwrap(),
        b"new"
    );
    assert_eq!(
        delta_reader.read_changed_asset("meshes/added.bin").unwrap(),
        b"added"
    );
    assert!(delta_reader.read_changed_asset("meshes/keep.bin").is_err());
    assert!(delta_reader
        .manifest()
        .target
        .asset("meshes/reused-alias.bin")
        .is_some());
}

#[test]
fn delta_pack_applies_to_base_pack() {
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("meshes/reused-source.bin", b"reused".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
        ZrPackInputAsset::new("textures/removed.bin", b"removed".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/added.bin", b"added".to_vec()),
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("meshes/reused-alias.bin", b"reused".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let target_bytes = target.bytes.clone();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    let delta_reader = ZrPackDeltaReader::from_bytes(delta.bytes).unwrap();

    let applied = delta_reader.apply_to_base(&base_reader).unwrap();
    let applied_reader = ZrPackReader::from_bytes(applied.bytes.clone()).unwrap();

    assert_eq!(applied.manifest, target_reader.manifest().clone());
    assert_eq!(applied.bytes, target_bytes);
    assert_eq!(
        applied_reader.read_asset("meshes/added.bin").unwrap(),
        b"added"
    );
    assert_eq!(
        applied_reader.read_asset("meshes/keep.bin").unwrap(),
        b"keep"
    );
    assert_eq!(
        applied_reader
            .read_asset("meshes/reused-alias.bin")
            .unwrap(),
        b"reused"
    );
    assert_eq!(
        applied_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    assert!(applied_reader.read_asset("textures/removed.bin").is_err());
    assert!(applied_reader
        .read_asset("meshes/reused-source.bin")
        .is_err());
}

#[test]
fn delta_pack_rejects_wrong_base_manifest() {
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let wrong_base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"wrong".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let wrong_base_reader = ZrPackReader::from_bytes(wrong_base.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    let delta_reader = ZrPackDeltaReader::from_bytes(delta.bytes).unwrap();

    let error = delta_reader.apply_to_base(&wrong_base_reader).unwrap_err();

    assert_eq!(error, ZrPackError::DeltaBaseManifestMismatch);
}

#[test]
fn delta_installer_rebuilds_target_pack_to_staging() {
    let root = unique_pack_temp_dir("delta-install");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/added.bin", b"added".to_vec()),
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();

    let report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();

    let staged_reader = ZrPackReader::from_bytes(fs::read(&staged_path).unwrap()).unwrap();
    assert_eq!(report.base_pack, base_path);
    assert_eq!(report.delta_pack, delta_path);
    assert_eq!(report.staged_pack, staged_path);
    assert_eq!(report.target_manifest, target_reader.manifest().clone());
    assert!(report.delta_apply_verified);
    assert_eq!(
        report.staged_size,
        fs::metadata(&report.staged_pack).unwrap().len()
    );
    assert_eq!(
        staged_reader.read_asset("meshes/added.bin").unwrap(),
        b"added"
    );
    assert_eq!(
        staged_reader.read_asset("meshes/keep.bin").unwrap(),
        b"keep"
    );
    assert_eq!(
        staged_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_rejects_wrong_base_without_staging() {
    let root = unique_pack_temp_dir("delta-install-wrong-base");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let wrong_base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"wrong".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, wrong_base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();

    let error = ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path)
        .unwrap_err();

    assert_eq!(
        error,
        ZrPackDeltaInstallError::Pack(ZrPackError::DeltaBaseManifestMismatch)
    );
    assert!(!staged_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_promotes_staged_pack_with_backup() {
    let root = unique_pack_temp_dir("delta-promote");
    let installed_path = root.join("installed").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let staged_path = root.join("staging").join("assets.zrpack");
    let installed = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let staged = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let staged_reader = ZrPackReader::from_bytes(staged.bytes.clone()).unwrap();
    fs::create_dir_all(installed_path.parent().unwrap()).unwrap();
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&installed_path, installed.bytes).unwrap();
    fs::write(&staged_path, staged.bytes).unwrap();

    let report = ZrPackDeltaInstaller::promote_staged_pack(
        &staged_path,
        &installed_path,
        Some(&backup_path),
    )
    .unwrap();

    let installed_reader = ZrPackReader::from_bytes(fs::read(&installed_path).unwrap()).unwrap();
    let backup_reader = ZrPackReader::from_bytes(fs::read(&backup_path).unwrap()).unwrap();
    assert_eq!(report.installed_pack, installed_path);
    assert_eq!(report.backup_pack, Some(backup_path.clone()));
    assert_eq!(report.staged_pack, staged_path);
    assert_eq!(report.installed_manifest, staged_reader.manifest().clone());
    assert_eq!(
        report.installed_size,
        fs::metadata(&report.installed_pack).unwrap().len()
    );
    assert_eq!(
        installed_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    assert_eq!(
        backup_reader.read_asset("textures/changed.bin").unwrap(),
        b"old"
    );
    assert!(!report.staged_pack.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_copies_staged_pack_when_promotion_rename_fails() {
    let root = unique_pack_temp_dir("delta-promote-copy-fallback");
    let installed_path = root.join("installed").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let staged_path = root.join("staging").join("assets.zrpack");
    let installed = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    let staged = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"new".to_vec(),
    )])
    .unwrap();
    let staged_reader = ZrPackReader::from_bytes(staged.bytes.clone()).unwrap();
    fs::create_dir_all(installed_path.parent().unwrap()).unwrap();
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&installed_path, installed.bytes).unwrap();
    fs::write(&staged_path, staged.bytes).unwrap();

    let report = ZrPackDeltaInstaller::promote_staged_pack_with_forced_staged_rename_failure(
        &staged_path,
        &installed_path,
        Some(&backup_path),
    )
    .unwrap();

    let installed_reader = ZrPackReader::from_bytes(fs::read(&installed_path).unwrap()).unwrap();
    let backup_reader = ZrPackReader::from_bytes(fs::read(&backup_path).unwrap()).unwrap();
    assert_eq!(
        report.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert_eq!(report.installed_manifest, staged_reader.manifest().clone());
    assert_eq!(
        installed_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    assert_eq!(
        backup_reader.read_asset("textures/changed.bin").unwrap(),
        b"old"
    );
    assert!(!staged_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_rejects_invalid_staged_pack_without_replacing_installed() {
    let root = unique_pack_temp_dir("delta-promote-invalid");
    let installed_path = root.join("installed").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let staged_path = root.join("staging").join("assets.zrpack");
    let installed = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    fs::create_dir_all(installed_path.parent().unwrap()).unwrap();
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&installed_path, installed.bytes).unwrap();
    fs::write(&staged_path, b"not-a-pack").unwrap();

    let error = ZrPackDeltaInstaller::promote_staged_pack(
        &staged_path,
        &installed_path,
        Some(&backup_path),
    )
    .unwrap_err();

    let installed_reader = ZrPackReader::from_bytes(fs::read(&installed_path).unwrap()).unwrap();
    assert_eq!(
        error,
        ZrPackDeltaInstallError::Pack(ZrPackError::HeaderTooSmall)
    );
    assert_eq!(
        installed_reader.read_asset("textures/changed.bin").unwrap(),
        b"old"
    );
    assert!(staged_path.exists());
    assert!(!backup_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_writes_install_receipt_from_staging_and_promotion() {
    let root = unique_pack_temp_dir("delta-receipt");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let receipt_path = root.join("receipts").join("assets.install.json");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();
    let staging_report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();
    let promotion_report =
        ZrPackDeltaInstaller::promote_staged_pack(&staged_path, &base_path, Some(&backup_path))
            .unwrap();

    let receipt = ZrPackDeltaInstaller::write_install_receipt(
        &receipt_path,
        &staging_report,
        &promotion_report,
    )
    .unwrap();
    let read_receipt = ZrPackDeltaInstaller::read_install_receipt(&receipt_path).unwrap();

    assert_eq!(receipt, read_receipt);
    assert_eq!(receipt.base_pack, base_path);
    assert_eq!(receipt.delta_pack, delta_path);
    assert_eq!(receipt.staged_pack, staged_path);
    assert_eq!(receipt.installed_pack, promotion_report.installed_pack);
    assert_eq!(receipt.backup_pack, Some(backup_path));
    assert_eq!(receipt.target_manifest, target_reader.manifest().clone());
    assert_eq!(receipt.installed_manifest, target_reader.manifest().clone());
    assert_eq!(receipt.staged_size, staging_report.staged_size);
    assert_eq!(receipt.installed_size, promotion_report.installed_size);
    assert!(receipt.delta_apply_verified);
    assert_eq!(receipt.promotion_method, ZrPackPromotionMethod::Renamed);
    assert!(receipt.promoted);
    assert_eq!(
        receipt.format_version,
        ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION
    );
    assert!(fs::read_to_string(&receipt_path)
        .unwrap()
        .contains("\"format_version\""));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_receipt_records_copy_fallback_promotion_method() {
    let root = unique_pack_temp_dir("delta-receipt-copy-fallback");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let receipt_path = root.join("receipts").join("assets.install.json");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();
    let staging_report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();
    let promotion_report =
        ZrPackDeltaInstaller::promote_staged_pack_with_forced_staged_rename_failure(
            &staged_path,
            &base_path,
            Some(&backup_path),
        )
        .unwrap();

    let receipt = ZrPackDeltaInstaller::write_install_receipt(
        &receipt_path,
        &staging_report,
        &promotion_report,
    )
    .unwrap();
    let read_receipt = ZrPackDeltaInstaller::read_install_receipt(&receipt_path).unwrap();

    assert_eq!(
        promotion_report.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert_eq!(
        receipt.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert_eq!(
        read_receipt.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert!(fs::read_to_string(&receipt_path)
        .unwrap()
        .contains("\"promotion_method\""));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_rejects_receipt_for_mismatched_reports() {
    let root = unique_pack_temp_dir("delta-receipt-mismatch");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let receipt_path = root.join("receipts").join("assets.install.json");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();
    let staging_report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();
    let mut promotion_report =
        ZrPackDeltaInstaller::promote_staged_pack(&staged_path, &base_path, Some(&backup_path))
            .unwrap();
    promotion_report.staged_pack = root.join("staging").join("other.zrpack");

    let error = ZrPackDeltaInstaller::write_install_receipt(
        &receipt_path,
        &staging_report,
        &promotion_report,
    )
    .unwrap_err();

    match error {
        ZrPackDeltaInstallError::ReceiptReportMismatch(message) => {
            assert!(message.contains("staged pack"));
        }
        other => panic!("unexpected install receipt error: {other:?}"),
    }
    assert!(!receipt_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn unreferenced_asset_trimmed_and_reported() {
    let report = ZrPackTrimPlanner::trim(
        ZrPackTrimConfig::new(["scenes/main.zscene"]),
        [
            ZrPackTrimInputAsset::new("scenes/main.zscene").with_dependency("textures/hero.png"),
            ZrPackTrimInputAsset::new("textures/hero.png"),
            ZrPackTrimInputAsset::new("textures/unused.png"),
        ],
    );

    assert_eq!(
        report.included_assets,
        ["scenes/main.zscene", "textures/hero.png"]
    );
    assert_eq!(report.trimmed_asset_count(), 1);
    assert_eq!(report.trimmed_assets[0].path, "textures/unused.png");
    assert_eq!(
        report.trimmed_assets[0].reason,
        ZrPackTrimReason::Unreferenced
    );
    assert_eq!(
        report.diagnostics,
        ["trimmed asset textures/unused.png: unreferenced"]
    );
}

#[test]
fn asset_filter_trim_is_reported() {
    let report = ZrPackTrimPlanner::trim(
        ZrPackTrimConfig::new(["scenes/main.zscene"]).with_asset_filter("shipping"),
        [
            ZrPackTrimInputAsset::new("scenes/main.zscene")
                .with_dependency("textures/hero.png")
                .with_label("shipping"),
            ZrPackTrimInputAsset::new("textures/hero.png"),
            ZrPackTrimInputAsset::new("textures/loading.png").with_label("shipping"),
        ],
    );

    assert_eq!(report.included_assets, ["scenes/main.zscene"]);
    assert_eq!(report.trimmed_asset_count(), 2);
    assert_eq!(report.trimmed_assets[0].path, "textures/hero.png");
    assert_eq!(
        report.trimmed_assets[0].reason,
        ZrPackTrimReason::AssetFilterMismatch("shipping".to_string())
    );
    assert_eq!(report.trimmed_assets[1].path, "textures/loading.png");
    assert_eq!(
        report.trimmed_assets[1].reason,
        ZrPackTrimReason::Unreferenced
    );
    assert_eq!(
        report.diagnostics,
        [
            "trimmed asset textures/hero.png: asset_filter shipping did not match",
            "trimmed asset textures/loading.png: unreferenced"
        ]
    );
}

#[test]
fn duplicate_trim_input_path_is_reported() {
    let report = ZrPackTrimPlanner::trim(
        ZrPackTrimConfig::new(["scenes/main.zscene"]),
        [
            ZrPackTrimInputAsset::new("scenes/main.zscene"),
            ZrPackTrimInputAsset::new("scenes/main.zscene"),
        ],
    );

    assert_eq!(report.included_assets, ["scenes/main.zscene"]);
    assert_eq!(report.duplicate_assets, ["scenes/main.zscene"]);
    assert!(report.has_duplicate_assets());
    assert_eq!(
        report.diagnostics,
        ["asset scenes/main.zscene is duplicated in trim input"]
    );
}

fn pack_asset_entry(path: impl Into<String>) -> ZrPackAssetEntry {
    pack_asset_entry_with_payload(path, b"data")
}

fn pack_asset_entry_with_payload(path: impl Into<String>, payload: &[u8]) -> ZrPackAssetEntry {
    ZrPackAssetEntry::new(
        path,
        zrpack_content_hash(payload),
        u64::try_from(payload.len()).unwrap(),
    )
}

fn chunks_for_asset_entries(assets: &[ZrPackAssetEntry]) -> Vec<ZrChunkEntry> {
    let mut unique_chunks = std::collections::BTreeMap::new();
    for asset in assets {
        unique_chunks
            .entry(asset.chunk_hash)
            .or_insert_with(|| u32::try_from(asset.size).unwrap());
    }
    let mut offset = ZRPACK_TEST_HEADER_SIZE as u64;
    unique_chunks
        .into_iter()
        .map(|(hash, size)| {
            let entry = ZrChunkEntry::new(hash, offset, size);
            offset += u64::from(size);
            entry
        })
        .collect()
}

fn total_chunk_size(chunks: &[ZrChunkEntry]) -> u64 {
    chunks.iter().map(|chunk| u64::from(chunk.size)).sum()
}

fn payload_bytes_for_chunks(chunks: &[ZrChunkEntry]) -> Vec<u8> {
    vec![0; usize::try_from(total_chunk_size(chunks)).unwrap()]
}

fn malformed_pack_bytes_with_assets(assets: Vec<ZrPackAssetEntry>) -> Vec<u8> {
    malformed_pack_bytes(pack_document_manifest_with_assets(assets))
}

fn malformed_pack_bytes(manifest: ZrPackDocumentManifest) -> Vec<u8> {
    let payload = payload_bytes_for_chunks(&manifest.pack.chunks);
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let manifest_offset = (ZRPACK_TEST_HEADER_SIZE + payload.len()) as u64;
    let manifest_size = manifest_bytes.len() as u64;
    let mut bytes = vec![0; ZRPACK_TEST_HEADER_SIZE];
    bytes.extend_from_slice(&payload);
    bytes.extend_from_slice(&manifest_bytes);
    bytes[0..4].copy_from_slice(&ZRPACK_MAGIC);
    bytes[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    bytes[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    bytes[16..24].copy_from_slice(&manifest_size.to_le_bytes());
    bytes
}

fn delta_manifest_with_assets(
    base_assets: Vec<ZrPackAssetEntry>,
    target_assets: Vec<ZrPackAssetEntry>,
    changed_assets: Vec<ZrPackAssetEntry>,
    removed_assets: Vec<String>,
) -> ZrPackDeltaDocumentManifest {
    let chunks = chunks_for_asset_entries(&changed_assets);
    ZrPackDeltaDocumentManifest {
        format_version: ZRPACK_FORMAT_VERSION,
        base: pack_document_manifest_with_assets(base_assets),
        target: pack_document_manifest_with_assets(target_assets),
        chunks,
        changed_assets,
        removed_assets,
    }
}

fn malformed_delta_bytes(manifest: ZrPackDeltaDocumentManifest) -> Vec<u8> {
    let payload = payload_bytes_for_chunks(&manifest.chunks);
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let manifest_offset = (ZRPACK_TEST_HEADER_SIZE + payload.len()) as u64;
    let manifest_size = manifest_bytes.len() as u64;
    let mut bytes = vec![0; ZRPACK_TEST_HEADER_SIZE];
    bytes.extend_from_slice(&payload);
    bytes.extend_from_slice(&manifest_bytes);
    bytes[0..4].copy_from_slice(&ZRPACK_DELTA_MAGIC);
    bytes[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    bytes[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    bytes[16..24].copy_from_slice(&manifest_size.to_le_bytes());
    bytes
}

fn bytes_with_manifest_gap(mut bytes: Vec<u8>, gap: &[u8]) -> Vec<u8> {
    let manifest_offset =
        usize::try_from(u64::from_le_bytes(bytes[8..16].try_into().unwrap())).unwrap();
    bytes.splice(manifest_offset..manifest_offset, gap.iter().copied());
    let new_manifest_offset = u64::try_from(manifest_offset + gap.len()).unwrap();
    bytes[8..16].copy_from_slice(&new_manifest_offset.to_le_bytes());
    bytes
}

fn bytes_with_manifest_trailing_bytes(mut bytes: Vec<u8>, trailing_bytes: &[u8]) -> Vec<u8> {
    bytes.extend_from_slice(trailing_bytes);
    bytes
}

fn pack_document_manifest_with_assets(assets: Vec<ZrPackAssetEntry>) -> ZrPackDocumentManifest {
    let chunks = chunks_for_asset_entries(&assets);
    let total_size = total_chunk_size(&chunks);
    ZrPackDocumentManifest::new(
        ZrPackManifest {
            version: ZRPACK_FORMAT_VERSION,
            chunks,
            total_size,
        },
        assets,
    )
}

const ZRPACK_TEST_HEADER_SIZE: usize = 24;

fn unique_pack_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "zircon-pack-{label}-{}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

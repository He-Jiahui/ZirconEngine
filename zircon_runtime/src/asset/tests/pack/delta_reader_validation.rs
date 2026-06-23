use super::*;

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

use super::*;

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

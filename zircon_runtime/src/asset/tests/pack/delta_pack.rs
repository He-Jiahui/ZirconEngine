use super::*;

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
fn runtime04_pack_reader_repeated_delta_reads_preserve_alias_and_byte_parity() {
    let reused = vec![0x19; 16 * 1_024];
    let changed = vec![0x82; 16 * 1_024];
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("a/changed.bin", b"before".to_vec()),
        ZrPackInputAsset::new("a/empty.bin", Vec::new()),
        ZrPackInputAsset::new("a/removed.bin", b"removed".to_vec()),
        ZrPackInputAsset::new("a/reused.bin", reused.clone()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("a/changed.bin", changed.clone()),
        ZrPackInputAsset::new("a/empty.bin", Vec::new()),
        ZrPackInputAsset::new("a/reused.bin", reused.clone()),
        ZrPackInputAsset::new("b/changed-alias.bin", changed.clone()),
        ZrPackInputAsset::new("b/reused-alias.bin", reused.clone()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes.as_slice()).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    assert_eq!(
        delta.changed_assets,
        ["a/changed.bin", "b/changed-alias.bin"]
    );
    assert_eq!(delta.removed_assets, ["a/removed.bin"]);
    assert_eq!(
        delta.reused_assets,
        ["a/empty.bin", "a/reused.bin", "b/reused-alias.bin"]
    );
    assert_eq!(delta.manifest.chunks.len(), 1);
    let delta_reader = ZrPackDeltaReader::from_bytes(delta.bytes.as_slice()).unwrap();

    for _ in 0..3 {
        let repeated_delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
        assert_eq!(repeated_delta.bytes, delta.bytes);
        let applied = delta_reader.apply_to_base(&base_reader).unwrap();
        assert_eq!(&applied.manifest, target_reader.manifest());
        assert_eq!(applied.bytes, target.bytes);
        let applied_reader = ZrPackReader::from_bytes(applied.bytes).unwrap();
        for path in ["a/reused.bin", "b/reused-alias.bin"] {
            assert_eq!(applied_reader.read_asset(path).unwrap(), reused);
        }
        for path in ["a/changed.bin", "b/changed-alias.bin"] {
            let mut bytes = applied_reader.read_asset(path).unwrap();
            assert_eq!(bytes, changed);
            bytes.fill(0);
            assert_eq!(applied_reader.read_asset(path).unwrap(), changed);
        }
        assert!(applied_reader.read_asset("a/empty.bin").unwrap().is_empty());
        assert_eq!(
            applied_reader.read_asset("a/removed.bin"),
            Err(ZrPackError::AssetNotFound("a/removed.bin".to_string()))
        );
    }
}

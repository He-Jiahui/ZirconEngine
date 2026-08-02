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
    assert!(
        delta_reader
            .manifest()
            .target
            .asset("meshes/reused-alias.bin")
            .is_some()
    );
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
    assert!(
        applied_reader
            .read_asset("meshes/reused-source.bin")
            .is_err()
    );
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

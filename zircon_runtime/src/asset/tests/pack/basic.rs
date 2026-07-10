use super::*;

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
fn pack_manifest_chunk_plan_round_trips_from_asset_owner() {
    let manifest = ZrPackManifest::new(1, 12)
        .with_chunk(ZrChunkEntry::new([1; 32], 0, 4))
        .with_chunk(ZrChunkEntry::new([2; 32], 4, 8));

    assert_eq!(manifest.covered_bytes(), 12);
    assert!(manifest.is_complete_byte_plan());
    assert_eq!(manifest.chunks[1].end_offset(), Some(12));
    let json = serde_json::to_value(&manifest).unwrap();
    assert_eq!(
        serde_json::from_value::<ZrPackManifest>(json).unwrap(),
        manifest
    );
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

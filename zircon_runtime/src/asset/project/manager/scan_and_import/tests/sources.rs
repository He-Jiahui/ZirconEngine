use std::fs;

use super::*;

#[test]
fn generated_source_snapshot_moves_primary_bytes_without_cloning() {
    let mut source = AssetImportSource {
        asset_root: PathBuf::from("."),
        path: PathBuf::from("generated.zcube"),
        uri: AssetUri::parse("res://generated.zcube").unwrap(),
        meta_path: PathBuf::from("generated.zcube.zmeta"),
        unit: AssetSourceUnit::Single,
        included_files: Vec::new(),
        included_paths: Vec::new(),
        compound_root: None,
        source_snapshot: Some(AssetImportSourceSnapshot {
            source_bytes: vec![1, 2, 3, 4],
            source_mtime_unix_ms: 17,
            source_file_snapshots: BTreeMap::new(),
        }),
        snapshot_complete: true,
    };
    let original = source
        .source_snapshot
        .as_ref()
        .expect("source snapshot")
        .source_bytes
        .as_ptr();

    let bytes = take_source_bytes_for_import(&mut source).unwrap();

    assert_eq!(bytes, [1, 2, 3, 4]);
    assert_eq!(bytes.as_ptr(), original);
    assert!(source
        .source_snapshot
        .as_ref()
        .expect("source snapshot metadata remains available")
        .source_bytes
        .is_empty());
}

#[test]
fn compound_snapshot_reuses_one_generation_for_digest_and_auxiliary_sources() {
    let root = std::env::temp_dir().join(format!(
        "zircon-compound-source-snapshot-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let compound_root = root.join("shader");
    fs::create_dir_all(&compound_root).unwrap();
    let meta_path = root.join("shader.zmeta");
    let descriptor_path = compound_root.join("shader.zshader");
    let source_path = compound_root.join("main.wgsl");
    fs::write(&meta_path, b"meta").unwrap();
    let descriptor = b"kind = \"include\"\nversion = 2\nimport_path = \"project::snapshot\"\nwgsl_files = [\"main.wgsl\"]\n";
    fs::write(&descriptor_path, descriptor).unwrap();
    fs::write(&source_path, b"source").unwrap();
    let included_paths = vec![descriptor_path.clone(), source_path.clone()];
    let mut source = AssetImportSource {
        asset_root: root.clone(),
        path: meta_path,
        uri: AssetUri::parse("res://shader").unwrap(),
        meta_path: root.join("shader.zmeta"),
        unit: AssetSourceUnit::Compound,
        included_files: Vec::new(),
        included_paths,
        compound_root: Some(compound_root),
        source_snapshot: None,
        snapshot_complete: false,
    };

    materialize_compound_source_snapshot(&mut source).unwrap();
    let digest = source_digest_for_import(
        &source.source_snapshot.as_ref().unwrap().source_bytes,
        &source
            .source_snapshot
            .as_ref()
            .unwrap()
            .source_file_snapshots,
        source.compound_root.as_deref(),
    );
    fs::write(&descriptor_path, b"changed descriptor").unwrap();
    fs::write(&source_path, b"changed source").unwrap();

    assert_eq!(
        source
            .source_snapshot
            .as_ref()
            .unwrap()
            .source_file_snapshots
            .get(&descriptor_path)
            .unwrap(),
        descriptor
    );
    assert_eq!(
        source
            .source_snapshot
            .as_ref()
            .unwrap()
            .source_file_snapshots
            .get(&source_path)
            .unwrap(),
        b"source"
    );
    let primary = take_source_bytes_for_import(&mut source).unwrap();
    assert_eq!(primary, b"meta");
    assert_eq!(digest.len(), 16);
    let snapshots = source.take_source_file_snapshots();
    assert_eq!(snapshots.len(), 2);
    let mut changed_snapshots = snapshots.clone();
    changed_snapshots.insert(source_path, b"changed source".to_vec());
    assert_ne!(
        source_digest_for_import(
            &primary,
            &changed_snapshots,
            source.compound_root.as_deref()
        ),
        digest,
        "an auxiliary-only change must invalidate restore"
    );
    assert!(source.source_file_snapshots().is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn gltf_generation_snapshot_digest_tracks_auxiliary_only_changes() {
    let root = std::env::temp_dir().join(format!(
        "zircon-gltf-generation-snapshot-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let asset_root = root.join("assets");
    fs::create_dir_all(&asset_root).unwrap();
    let source_path = asset_root.join("model.gltf");
    let buffer_path = asset_root.join("mesh.bin");
    fs::write(
        &source_path,
        br#"{"asset":{"version":"2.0"},"buffers":[{"uri":"mesh.bin","byteLength":4}]}"#,
    )
    .unwrap();
    fs::write(&buffer_path, [1, 2, 3, 4]).unwrap();
    let new_source = || AssetImportSource {
        asset_root: asset_root.clone(),
        path: source_path.clone(),
        uri: AssetUri::parse("res://model.gltf").unwrap(),
        meta_path: source_path.with_extension("gltf.zmeta"),
        unit: AssetSourceUnit::Single,
        included_files: Vec::new(),
        included_paths: Vec::new(),
        compound_root: None,
        source_snapshot: None,
        snapshot_complete: false,
    };
    let mut first = new_source();
    materialize_compound_source_snapshot(&mut first).unwrap();
    let captured = first.source_snapshot.as_ref().unwrap();
    let first_digest = source_digest_for_import(
        &captured.source_bytes,
        &captured.source_file_snapshots,
        Some(&asset_root),
    );
    fs::write(&buffer_path, [5, 6, 7, 8]).unwrap();
    assert_eq!(
        captured.source_file_snapshots.values().next().unwrap(),
        &[1, 2, 3, 4]
    );
    let mut second = new_source();
    materialize_compound_source_snapshot(&mut second).unwrap();
    let changed = second.source_snapshot.as_ref().unwrap();
    assert_eq!(captured.source_bytes, changed.source_bytes);
    assert_ne!(
        first_digest,
        source_digest_for_import(
            &changed.source_bytes,
            &changed.source_file_snapshots,
            Some(&asset_root)
        )
    );
    let _ = fs::remove_dir_all(root);
}

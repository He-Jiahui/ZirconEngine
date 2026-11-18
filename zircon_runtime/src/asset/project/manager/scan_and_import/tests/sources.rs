use std::fs;

use crate::asset::{AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, AssetKind};

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

#[derive(Debug)]
struct DeclaredSources(AssetImporterDescriptor, Vec<PathBuf>);

impl AssetImporterHandler for DeclaredSources {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.0
    }

    fn auxiliary_source_paths(
        &self,
        _source_path: &Path,
        primary: &[u8],
    ) -> Result<Vec<PathBuf>, AssetImportError> {
        assert_eq!(primary, b"supplied primary");
        Ok(self.1.clone())
    }

    fn import(&self, _: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        unreachable!("snapshot discovery does not execute the importer")
    }
}

#[test]
fn declared_sources_use_current_supplied_bytes_and_do_not_fill_a_complete_snapshot_from_disk() {
    let temp = std::env::temp_dir().join(format!(
        "zircon-declared-sources-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(temp.join("manifests/layers")).unwrap();
    let root = ProjectPaths::resolve_existing_path(&temp).unwrap();
    let parent = root.join("manifests");
    let supplied_path = parent.join("layers").join("tile.bin");
    let disk_only_path = parent.join("layers").join("other.bin");
    fs::write(&supplied_path, b"earlier disk version").unwrap();
    fs::write(&disk_only_path, b"ordinary on-disk input").unwrap();
    let mut source = AssetImportSource {
        asset_root: root,
        path: parent.join("source.fixture"),
        uri: AssetUri::parse("res://manifests/source.fixture").unwrap(),
        meta_path: parent.join("source.fixture.zmeta"),
        unit: AssetSourceUnit::Single,
        included_files: Vec::new(),
        included_paths: Vec::new(),
        compound_root: None,
        source_snapshot: None,
        snapshot_complete: false,
    }
    .with_source_snapshot(AssetImportSourceSnapshot {
        source_bytes: b"supplied primary".to_vec(),
        source_mtime_unix_ms: 23,
        source_file_snapshots: BTreeMap::from([(supplied_path.clone(), b"current".to_vec())]),
    });
    let mut handler = DeclaredSources(
        AssetImporterDescriptor::new("fixture", "fixture", AssetKind::Data, 1),
        vec!["./layers/tile.bin".into(), "layers/./tile.bin".into()],
    );
    #[cfg(windows)]
    handler.1.push("LAYERS/TILE.BIN".into());
    snapshot_handler_auxiliary_sources(&mut source, Some(&handler), true).unwrap();
    let admitted = source.source_file_snapshots();
    assert_eq!(admitted.len(), 1);
    assert_eq!(admitted[&supplied_path].as_slice(), b"current");
    assert_eq!(source_mtime_unix_ms_for_import(&source).unwrap(), 23);

    handler.1.push("./source.fixture".into());
    let error = snapshot_handler_auxiliary_sources(&mut source, Some(&handler), true)
        .expect_err("the primary source is not a companion input");
    assert!(
        error.to_string().contains("names the primary source"),
        "{error}"
    );
    assert_eq!(source.source_file_snapshots(), admitted);
    let _ = handler.1.pop();

    handler.1.push("layers/other.bin".into());
    let error = snapshot_handler_auxiliary_sources(&mut source, Some(&handler), true)
        .expect_err("a complete transaction snapshot cannot acquire a missing input from disk");
    assert!(error.to_string().contains("other.bin"), "{error}");
    assert_eq!(source.source_file_snapshots(), admitted);

    handler.1.push("layers/missing.bin".into());
    let error = snapshot_handler_auxiliary_sources(&mut source, Some(&handler), false)
        .expect_err("a missing input must discard additions captured during this attempt");
    assert!(error.to_string().contains("missing.bin"), "{error}");
    assert_eq!(source.source_file_snapshots(), admitted);
    let _ = handler.1.pop();
    handler.1.push("./layers/./other.bin".into());
    #[cfg(windows)]
    handler.1.push("LAYERS/OTHER.BIN".into());

    // When only the primary/existing inputs were supplied, finish the capture using the resolver.
    snapshot_handler_auxiliary_sources(&mut source, Some(&handler), false).unwrap();
    let captured = source.source_file_snapshots();
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[&supplied_path].as_slice(), b"current");
    assert_eq!(
        captured[&disk_only_path].as_slice(),
        b"ordinary on-disk input"
    );
    fs::remove_dir_all(temp).unwrap();
}

use std::fs;

use crate::asset::project::{AssetMetaError, ProjectManifest, ProjectPaths};
use crate::asset::{AssetKind, AssetUuid};
use zircon_runtime_interface::project::RelPath;

use super::*;

#[test]
fn compound_discovery_keeps_identity_members_and_final_source_order_with_mixed_sidecars() {
    let root = std::env::temp_dir().join(format!(
        "zircon-compound-discovery-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Compound Discovery",
        AssetUri::parse("res://package").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let asset_root = paths.asset_root(&RelPath::project_assets());
    let manager = ProjectManager::open(&root).unwrap();

    for index in 0..32 {
        fs::write(
            asset_root.join(format!("invalid_{index:02}.zmeta")),
            b"not valid metadata",
        )
        .unwrap();
    }
    let malformed_root = asset_root.join("malformed");
    fs::create_dir_all(&malformed_root).unwrap();
    fs::write(asset_root.join("malformed.zmeta"), b"not valid metadata").unwrap();
    fs::write(malformed_root.join("member.json"), b"{}").unwrap();
    let ordinary_meta = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://single.json").unwrap(),
        AssetKind::Data,
    );
    ordinary_meta
        .save(asset_root.join("single.json.zmeta"))
        .unwrap();

    let mismatched_root = asset_root.join("mismatched");
    fs::create_dir_all(&mismatched_root).unwrap();
    let mut mismatched_meta = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://different").unwrap(),
        AssetKind::Data,
    );
    mismatched_meta.unit = AssetSourceUnit::Compound;
    mismatched_meta
        .save(asset_root.join("mismatched.zmeta"))
        .unwrap();

    let compound_root = asset_root.join("package");
    fs::create_dir_all(compound_root.join("nested")).unwrap();
    let first_member = compound_root.join("b.wgsl");
    let second_member = compound_root.join("nested").join("a.wgsl");
    fs::write(&second_member, b"second").unwrap();
    fs::write(&first_member, b"first").unwrap();
    let compound_uri = AssetUri::parse("res://package").unwrap();
    let meta_path = asset_root.join("package.zmeta");
    let mut compound_meta =
        AssetMetaDocument::new(AssetUuid::new(), compound_uri.clone(), AssetKind::Data);
    compound_meta.unit = AssetSourceUnit::Compound;
    compound_meta.save(&meta_path).unwrap();
    fs::write(asset_root.join("zz-ordinary.json"), b"{}").unwrap();

    let mut observation = ProjectGenerationObservation::new();
    let compounds = manager
        .collect_compound_sources_for_root(&asset_root, None, &mut observation)
        .unwrap();
    assert_eq!(compounds.len(), 1);
    let compound = &compounds[0];
    assert_eq!(compound.uri, compound_uri);
    assert_eq!(compound.path, meta_path);
    assert_eq!(
        compound.compound_root.as_deref(),
        Some(compound_root.as_path())
    );
    assert_eq!(compound.included_paths, vec![first_member, second_member]);
    assert_eq!(
        compound.included_files,
        vec![
            AssetUri::parse("res://package/b.wgsl").unwrap(),
            AssetUri::parse("res://package/nested/a.wgsl").unwrap(),
        ]
    );

    let mut observation = ProjectGenerationObservation::new();
    let sources = manager.collect_import_sources(&mut observation).unwrap();
    assert_eq!(
        sources
            .into_iter()
            .map(|source| source.uri)
            .collect::<Vec<_>>(),
        vec![
            AssetUri::parse("res://malformed/member.json").unwrap(),
            AssetUri::parse("res://package").unwrap(),
            AssetUri::parse("res://zz-ordinary.json").unwrap(),
        ]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn compound_discovery_skips_oversize_sidecar_before_parsing_it() {
    let root = std::env::temp_dir().join(format!(
        "zircon-compound-oversize-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Oversize Sidecar",
        AssetUri::parse("res://oversize").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let manager = ProjectManager::open(&root).unwrap();
    let asset_root = paths.asset_root(&RelPath::project_assets());
    let meta_path = asset_root.join("oversize.zmeta");
    fs::File::create(&meta_path)
        .unwrap()
        .set_len(crate::asset::project::meta::MAX_ASSET_META_DOCUMENT_BYTES as u64 + 1)
        .unwrap();

    let mut observation = ProjectGenerationObservation::new();
    let error = observation
        .load_metadata_document(&meta_path)
        .expect_err("the discovery reader must enforce the sidecar budget");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("byte limit"), "{error}");

    let compounds = manager
        .collect_compound_sources_for_root(&asset_root, None, &mut observation)
        .unwrap();
    assert!(compounds.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn full_generation_inventory_rejects_oversize_sidecar_paired_with_a_directory() {
    let root = std::env::temp_dir().join(format!(
        "zircon-compound-paired-oversize-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Paired Oversize Sidecar",
        AssetUri::parse("res://package").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let manager = ProjectManager::open(&root).unwrap();
    let asset_root = paths.asset_root(&RelPath::project_assets());
    let compound_root = asset_root.join("package");
    fs::create_dir_all(&compound_root).unwrap();
    fs::write(compound_root.join("member.json"), b"{}").unwrap();
    fs::File::create(asset_root.join("package.zmeta"))
        .unwrap()
        .set_len(crate::asset::project::meta::MAX_ASSET_META_DOCUMENT_BYTES as u64 + 1)
        .unwrap();

    let mut observation = ProjectGenerationObservation::new();
    let error = match manager.collect_import_sources(&mut observation) {
        Ok(sources) => panic!(
            "oversize package must not reinterpret members as independent sources: {:?}",
            sources
                .into_iter()
                .map(|source| source.uri)
                .collect::<Vec<_>>()
        ),
        Err(error) => error,
    };
    let source = match error {
        AssetImportError::Io(source) => source,
        other => panic!("expected typed oversized sidecar I/O error, got {other}"),
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|source| source.downcast_ref::<AssetMetaError>()),
        Some(AssetMetaError::DocumentTooLarge { .. })
    ));

    let _ = fs::remove_dir_all(root);
}

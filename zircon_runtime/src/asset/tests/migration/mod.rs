mod project_commandlet;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::asset::project::{AssetMetaDocument, AssetMetaEntry, AssetSourceUnit};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn migration_report_formats_into_one_output_buffer() {
    let source = include_str!("../../migration/report.rs");

    assert!(source.contains("let mut output = String::new()"));
    assert!(!source.contains("let mut lines = vec!"));
    assert!(!source.contains("lines.join"));
}

fn fixture_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon_migrate_assets_{label}_{}_{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn write_manifest(root: &Path, asset_roots: &[&str]) {
    let roots = asset_roots
        .iter()
        .map(|root| format!("\"{root}\""))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        root.join("zircon-project.toml"),
        format!(
            "name = \"Migration Fixture\"\nformat_version = 2\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [{roots}]\nlibrary_version = 1\n"
        ),
    )
    .unwrap();
}

fn write_registered_source(
    project_root: &Path,
    asset_root: &str,
    relative: &str,
    guid: AssetUuid,
    kind: AssetKind,
) {
    let source = project_root.join(asset_root).join(relative);
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, b"fixture source").unwrap();
    let uri = AssetUri::parse(&format!("res://{relative}")).unwrap();
    let meta = AssetMetaDocument::new(guid, uri, kind);
    meta.save(source.with_file_name(format!(
        "{}.zmeta",
        source.file_name().unwrap().to_string_lossy()
    )))
    .unwrap();
}

fn write_registered_subasset(
    project_root: &Path,
    asset_root: &str,
    relative: &str,
    root_guid: AssetUuid,
    sub_guid: AssetUuid,
    label: &str,
) {
    let source = project_root.join(asset_root).join(relative);
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, b"compound fixture source").unwrap();
    let root_uri = AssetUri::parse(&format!("res://{relative}")).unwrap();
    let mut meta = AssetMetaDocument::new(root_guid, root_uri, AssetKind::Model);
    meta.entries.push(AssetMetaEntry {
        uuid: sub_guid,
        url: AssetUri::parse(&format!("res://{relative}#{label}")).unwrap(),
        asset_kind: AssetKind::Mesh,
        artifact_locator: None,
        dependencies: Vec::new(),
        tags: Default::default(),
    });
    let sidecar = source.with_file_name(format!(
        "{}.zmeta",
        source.file_name().unwrap().to_string_lossy()
    ));
    meta.save(&sidecar).unwrap();

    let persisted = AssetMetaDocument::load(sidecar).unwrap();
    assert_eq!(persisted.unit, AssetSourceUnit::Single);
    assert_eq!(persisted.entries.len(), 1);
    assert_eq!(persisted.entries[0].uuid, sub_guid);
    assert_eq!(persisted.entries[0].url.label(), Some(label));
}

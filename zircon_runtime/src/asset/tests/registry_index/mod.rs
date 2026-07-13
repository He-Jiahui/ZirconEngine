mod dependency_extractors;
mod incremental;
mod persistence;
mod queries;
mod scan_safety;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::{AssetMetaDocument, ProjectPaths};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn unique_root(label: &str) -> PathBuf {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_registry_{label}_{id}"))
}

fn write_asset(
    root: &std::path::Path,
    relative: &str,
    uuid: AssetUuid,
    kind: AssetKind,
    dependencies: Vec<AssetUri>,
) -> PathBuf {
    write_asset_with_tags(root, relative, uuid, kind, dependencies, BTreeSet::new())
}

fn write_asset_with_tags(
    root: &std::path::Path,
    relative: &str,
    uuid: AssetUuid,
    kind: AssetKind,
    dependencies: Vec<AssetUri>,
    tags: BTreeSet<String>,
) -> PathBuf {
    let source = root.join(relative);
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, b"source").unwrap();
    let mut meta = AssetMetaDocument::new(uuid, uri(&format!("res://{relative}")), kind);
    meta.source_digest = format!("digest-{relative}");
    meta.dependencies = dependencies;
    meta.tags = tags;
    meta.save(source.with_file_name(format!(
        "{}.zmeta",
        source.file_name().unwrap().to_string_lossy()
    )))
    .unwrap();
    source
}

fn registry_root(project_root: &std::path::Path) -> PathBuf {
    ProjectPaths::from_root(project_root)
        .unwrap()
        .registry_root()
        .to_path_buf()
}

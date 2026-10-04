use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const ASSET_CATALOG_LIMIT: usize = 256;
const PROJECT_ASSET_DIRS: &[&str] = &["Assets", "assets"];
pub const SELECTED_PROJECT_ASSET_SOURCE: &str = "Selected Project";
pub const PROJECT_ASSET_SOURCE: &str = "Project";
const ENGINE_ASSET_ROOTS: &[(&str, &[&str])] = &[
    ("Editor", &["zircon_editor", "assets"]),
    ("Runtime", &["zircon_runtime", "assets"]),
];
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];

/// Hub 资产视图使用的只读目录项；路径是后续打开操作的目标，来源标签用于筛选而非资产身份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetCatalogEntry {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub size_bytes: u64,
    pub path: PathBuf,
}

#[derive(Clone)]
struct RankedAssetCatalogEntry {
    root_rank: usize,
    entry: AssetCatalogEntry,
}

pub fn discover_asset_catalog<P, R>(
    project_roots: P,
    repo_roots: R,
) -> Result<Vec<AssetCatalogEntry>, HubError>
where
    P: IntoIterator<Item = PathBuf>,
    R: IntoIterator<Item = PathBuf>,
{
    discover_asset_catalog_for_scope(None, project_roots, repo_roots)
}

/// 在所选项目、近期项目和源码引擎之间建立有优先级的资产视图。
/// 调用方应按当前选择传入项目根，并在选择或引擎变化后重新发现；结果上限用于界面展示。
pub fn discover_asset_catalog_for_scope<P, R>(
    selected_project_root: Option<PathBuf>,
    project_roots: P,
    repo_roots: R,
) -> Result<Vec<AssetCatalogEntry>, HubError>
where
    P: IntoIterator<Item = PathBuf>,
    R: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_roots = HashSet::new();

    if let Some(project_root) = selected_project_root {
        collect_project_asset_roots(
            SELECTED_PROJECT_ASSET_SOURCE,
            &project_root,
            0,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    let mut project_root_rank = 0;
    for project_root in project_roots {
        collect_project_asset_roots(
            PROJECT_ASSET_SOURCE,
            &project_root,
            project_root_rank,
            &mut visited_roots,
            &mut entries,
        )?;
        project_root_rank += 1;
    }

    for (root_rank, repo_root) in repo_roots.into_iter().enumerate() {
        for (label, segments) in ENGINE_ASSET_ROOTS {
            let root = segments
                .iter()
                .fold(repo_root.clone(), |path, segment| path.join(segment));
            collect_asset_root(label, &root, root_rank, &mut visited_roots, &mut entries)?;
        }
    }

    retain_top_ranked_entries(&mut entries);
    Ok(entries.into_iter().map(|ranked| ranked.entry).collect())
}

fn collect_project_asset_roots(
    source: &str,
    project_root: &Path,
    root_rank: usize,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<RankedAssetCatalogEntry>,
) -> Result<(), HubError> {
    for asset_dir in PROJECT_ASSET_DIRS {
        let root = project_root.join(asset_dir);
        collect_asset_root(source, &root, root_rank, visited_roots, entries)?;
    }
    Ok(())
}

fn source_priority(source: &str) -> u8 {
    match source {
        SELECTED_PROJECT_ASSET_SOURCE => 0,
        PROJECT_ASSET_SOURCE => 1,
        _ => 2,
    }
}

fn ranked_asset_order(left: &RankedAssetCatalogEntry, right: &RankedAssetCatalogEntry) -> Ordering {
    source_priority(&left.entry.source)
        .cmp(&source_priority(&right.entry.source))
        .then_with(|| left.root_rank.cmp(&right.root_rank))
        .then_with(|| left.entry.source.cmp(&right.entry.source))
        .then_with(|| left.entry.kind.cmp(&right.entry.kind))
        .then_with(|| left.entry.name.cmp(&right.entry.name))
        .then_with(|| left.entry.path.cmp(&right.entry.path))
}

// 先保留高优先级前缀，再排序，以免数量上限把当前项目资产挤出 Hub 视图。
fn retain_top_ranked_entries(entries: &mut Vec<RankedAssetCatalogEntry>) {
    if entries.len() > ASSET_CATALOG_LIMIT {
        entries.select_nth_unstable_by(ASSET_CATALOG_LIMIT, ranked_asset_order);
        entries.truncate(ASSET_CATALOG_LIMIT);
    }
    entries.sort_by(ranked_asset_order);
}

fn collect_asset_root(
    source: &str,
    root: &Path,
    root_rank: usize,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<RankedAssetCatalogEntry>,
) -> Result<(), HubError> {
    if !root.is_dir() {
        return Ok(());
    }
    let root_key = project_filesystem_path_key(root);
    if !visited_roots.insert(root_key) {
        return Ok(());
    }
    collect_asset_files(source, root, root_rank, entries)
}

fn collect_asset_files(
    source: &str,
    directory: &Path,
    root_rank: usize,
    entries: &mut Vec<RankedAssetCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_asset_files(source, &path, root_rank, entries)?;
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            entries.push(RankedAssetCatalogEntry {
                root_rank,
                entry: AssetCatalogEntry {
                    name: path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("asset")
                        .to_string(),
                    kind: asset_kind(&path),
                    source: source.to_string(),
                    size_bytes: metadata.len(),
                    path,
                },
            });
        }
    }
    Ok(())
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn asset_kind(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name.ends_with(".ui.toml") || file_name.ends_with(".v2.ui.toml") {
        return "ui".to_string();
    }
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" | "jpg" | "jpeg" | "webp" | "svg" => "image",
        "glb" | "gltf" | "obj" | "fbx" => "model",
        "wav" | "ogg" | "mp3" | "flac" => "audio",
        "wgsl" | "glsl" | "hlsl" => "shader",
        "toml" | "json" | "ron" => "data",
        "zircon" | "scene" => "scene",
        "" => "file",
        other => other,
    }
    .to_string()
}

#[cfg(test)]
#[path = "tests/catalog.rs"]
mod tests;

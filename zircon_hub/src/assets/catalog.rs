use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;

const ASSET_CATALOG_LIMIT: usize = 256;
const PROJECT_ASSET_DIRS: &[&str] = &["Assets", "assets"];
pub const SELECTED_PROJECT_ASSET_SOURCE: &str = "Selected Project";
pub const PROJECT_ASSET_SOURCE: &str = "Project";
const ENGINE_ASSET_ROOTS: &[(&str, &[&str])] = &[
    ("Editor", &["zircon_editor", "assets"]),
    ("Runtime", &["zircon_runtime", "assets"]),
];
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetCatalogEntry {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub size_bytes: u64,
    pub path: PathBuf,
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
            &mut visited_roots,
            &mut entries,
        )?;
    }

    for project_root in project_roots {
        collect_project_asset_roots(
            PROJECT_ASSET_SOURCE,
            &project_root,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    for repo_root in repo_roots {
        for (label, segments) in ENGINE_ASSET_ROOTS {
            let root = segments
                .iter()
                .fold(repo_root.clone(), |path, segment| path.join(segment));
            collect_asset_root(label, &root, &mut visited_roots, &mut entries)?;
        }
    }

    entries.sort_by(|left, right| {
        source_priority(&left.source)
            .cmp(&source_priority(&right.source))
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.path.cmp(&right.path))
    });
    entries.truncate(ASSET_CATALOG_LIMIT);
    Ok(entries)
}

fn collect_project_asset_roots(
    source: &str,
    project_root: &Path,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<AssetCatalogEntry>,
) -> Result<(), HubError> {
    for asset_dir in PROJECT_ASSET_DIRS {
        let root = project_root.join(asset_dir);
        collect_asset_root(source, &root, visited_roots, entries)?;
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

fn collect_asset_root(
    source: &str,
    root: &Path,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<AssetCatalogEntry>,
) -> Result<(), HubError> {
    if !root.is_dir() {
        return Ok(());
    }
    let root_key = normalized_path_key(root);
    if !visited_roots.insert(root_key) {
        return Ok(());
    }
    collect_asset_files(source, root, entries)
}

fn collect_asset_files(
    source: &str,
    directory: &Path,
    entries: &mut Vec<AssetCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_asset_files(source, &path, entries)?;
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            entries.push(AssetCatalogEntry {
                name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("asset")
                    .to_string(),
                kind: asset_kind(&path),
                source: source.to_string(),
                size_bytes: metadata.len(),
                path,
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

fn normalized_path_key(path: &Path) -> String {
    let value = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    if cfg!(target_os = "windows") {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn discover_asset_catalog_reads_project_and_engine_assets() {
        let project_root = temp_dir("asset-project");
        let repo_root = temp_dir("asset-repo");
        fs::create_dir_all(project_root.join("Assets").join("textures")).unwrap();
        fs::write(
            project_root
                .join("Assets")
                .join("textures")
                .join("diffuse.png"),
            "image",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("zircon_editor").join("assets").join("icons")).unwrap();
        fs::write(
            repo_root
                .join("zircon_editor")
                .join("assets")
                .join("icons")
                .join("add.svg"),
            "svg",
        )
        .unwrap();

        let entries = discover_asset_catalog([project_root.clone()], [repo_root.clone()]).unwrap();
        fs::remove_dir_all(project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.source == "Project" && entry.kind == "image"));
        assert!(entries
            .iter()
            .any(|entry| entry.source == "Editor" && entry.name == "add.svg"));
    }

    #[test]
    fn discover_asset_catalog_skips_transient_directories() {
        let project_root = temp_dir("asset-skip");
        fs::create_dir_all(project_root.join("Assets").join("target")).unwrap();
        fs::write(
            project_root.join("Assets").join("target").join("cache.png"),
            "cache",
        )
        .unwrap();

        let entries =
            discover_asset_catalog([project_root.clone()], Vec::<PathBuf>::new()).unwrap();
        fs::remove_dir_all(project_root).unwrap();

        assert!(entries.is_empty());
    }

    #[test]
    fn discover_asset_catalog_labels_selected_project_assets() {
        let selected_project_root = temp_dir("asset-selected");
        let other_project_root = temp_dir("asset-other");
        fs::create_dir_all(selected_project_root.join("Assets")).unwrap();
        fs::write(
            selected_project_root.join("Assets").join("hero.glb"),
            "model",
        )
        .unwrap();
        fs::create_dir_all(other_project_root.join("assets")).unwrap();
        fs::write(
            other_project_root.join("assets").join("ambient.ogg"),
            "audio",
        )
        .unwrap();

        let entries = discover_asset_catalog_for_scope(
            Some(selected_project_root.clone()),
            [selected_project_root.clone(), other_project_root.clone()],
            Vec::<PathBuf>::new(),
        )
        .unwrap();
        fs::remove_dir_all(selected_project_root).unwrap();
        fs::remove_dir_all(other_project_root).unwrap();

        assert!(entries.iter().any(|entry| {
            entry.name == "hero.glb" && entry.source == SELECTED_PROJECT_ASSET_SOURCE
        }));
        assert!(entries
            .iter()
            .any(|entry| entry.name == "ambient.ogg" && entry.source == PROJECT_ASSET_SOURCE));
        assert!(!entries
            .iter()
            .any(|entry| entry.name == "hero.glb" && entry.source == PROJECT_ASSET_SOURCE));
    }

    #[test]
    fn discover_asset_catalog_orders_selected_project_assets_first() {
        let selected_project_root = temp_dir("asset-selected-first");
        let other_project_root = temp_dir("asset-other-first");
        let repo_root = temp_dir("asset-repo-first");
        fs::create_dir_all(selected_project_root.join("Assets")).unwrap();
        fs::write(
            selected_project_root.join("Assets").join("hero.glb"),
            "model",
        )
        .unwrap();
        fs::create_dir_all(other_project_root.join("assets")).unwrap();
        fs::write(
            other_project_root.join("assets").join("ambient.ogg"),
            "audio",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("zircon_runtime").join("assets")).unwrap();
        fs::write(
            repo_root
                .join("zircon_runtime")
                .join("assets")
                .join("runtime.svg"),
            "svg",
        )
        .unwrap();

        let entries = discover_asset_catalog_for_scope(
            Some(selected_project_root.clone()),
            [selected_project_root.clone(), other_project_root.clone()],
            [repo_root.clone()],
        )
        .unwrap();
        fs::remove_dir_all(selected_project_root).unwrap();
        fs::remove_dir_all(other_project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries[0].source, SELECTED_PROJECT_ASSET_SOURCE);
        assert_eq!(entries[1].source, PROJECT_ASSET_SOURCE);
        assert!(entries[2..]
            .iter()
            .all(|entry| entry.source != SELECTED_PROJECT_ASSET_SOURCE));
    }

    fn temp_dir(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("zircon-hub-{label}-{now}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}

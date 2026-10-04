use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetUri, FontAsset};
use zircon_runtime::core::resource::ResourceScheme;
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::ui::surface::{UiRenderExtract, UiVisualAssetRef};

const DEFAULT_FONT_URI: &str = "res://fonts/default.font.toml";

#[cfg(test)]
mod tests {
    use super::{font_asset_uri, DEFAULT_FONT_URI};

    #[test]
    fn native_capture_accepts_checked_in_project_font_manifests() {
        assert_eq!(
            font_asset_uri(DEFAULT_FONT_URI).unwrap().to_string(),
            DEFAULT_FONT_URI
        );
        assert_eq!(
            font_asset_uri("res://fonts/editor-ui.font.toml")
                .unwrap()
                .to_string(),
            "res://fonts/editor-ui.font.toml"
        );
    }

    #[test]
    fn native_capture_rejects_font_manifests_outside_the_project_font_root() {
        assert!(font_asset_uri("res://textures/not-a-font.toml").is_err());
        assert!(font_asset_uri("file:///tmp/editor.font.toml").is_err());
        assert!(font_asset_uri("res://fonts/../secrets.font.toml").is_err());
    }
}

fn font_asset_uri(value: &str) -> Result<AssetUri, String> {
    if !value.starts_with("res://fonts/") || value.contains("..") || !value.ends_with(".font.toml")
    {
        return Err(format!(
            "native capture font asset must be a project manifest under res://fonts/: {value}"
        ));
    }
    let uri =
        AssetUri::parse(value).map_err(|error| format!("invalid font asset {value}: {error}"))?;
    if uri.scheme() != ResourceScheme::Res || uri.label().is_some() {
        return Err(format!(
            "unsupported native capture font asset scheme in {value}"
        ));
    }
    Ok(uri)
}
pub(super) struct PreparedAssets {
    pub manager: Arc<ProjectAssetManager>,
    pub fingerprints: Vec<(String, String)>,
}

pub(super) fn prepare_asset_manager(
    repo_root: &Path,
    source_path: &Path,
    dependency_paths: &[PathBuf],
    work_root: &Path,
    extract: &UiRenderExtract,
) -> Result<PreparedAssets, String> {
    let paths = ProjectPaths::from_root(work_root).map_err(|error| error.to_string())?;
    paths
        .ensure_layout(&[RelPath::project_assets()])
        .map_err(|error| error.to_string())?;

    let asset_root = paths.asset_root(&RelPath::project_assets());
    let mut fingerprints = copy_default_font(repo_root, &asset_root)?;
    let mut font_uris = BTreeSet::from([DEFAULT_FONT_URI.to_string()]);

    let mut image_uris = Vec::new();
    for command in &extract.list.commands {
        if let Some(font) = command.style.font.as_deref() {
            let _ = font_asset_uri(font)?;
            font_uris.insert(font.to_string());
        }
        let text = command
            .text
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if text.contains("<img") || text.contains("[img") {
            return Err(
                "rich inline image markup is unsupported by native capture asset preparation"
                    .to_string(),
            );
        }
        let Some(UiVisualAssetRef::Image(source)) = command.image.as_ref() else {
            continue;
        };
        let uri = AssetUri::parse(source)
            .map_err(|error| format!("invalid image URI {source}: {error}"))?;
        if uri.scheme() != ResourceScheme::Res || uri.label().is_some() {
            return Err(format!(
                "unsupported native capture image scheme in {source}"
            ));
        }
        let relative = uri
            .path()
            .trim_start_matches('/')
            .replace('/', std::path::MAIN_SEPARATOR_STR);
        if relative.is_empty() || relative.contains("..") {
            return Err(format!("unsafe native capture image path in {source}"));
        }
        let mut candidates = nearest_assets_dir(source_path)
            .into_iter()
            .map(|p| p.join(&relative))
            .collect::<Vec<_>>();
        let suffix = Path::new("assets").join(&relative);
        candidates.extend(
            dependency_paths
                .iter()
                .filter(|p| p.ends_with(&suffix))
                .cloned(),
        );
        let mut canonical_candidates = Vec::new();
        for candidate in candidates.iter().filter(|p| p.is_file()) {
            canonical_candidates.push(contained_source(repo_root, candidate)?);
        }
        canonical_candidates.sort();
        canonical_candidates.dedup();
        candidates = canonical_candidates;
        let source_file = match candidates.as_slice() {
            [p] => p,
            [] => return Err(format!("missing native capture image source for {source}")),
            _ => {
                return Err(format!(
                    "ambiguous native capture image source for {source}"
                ));
            }
        };
        let canonical = contained_source(repo_root, source_file)?;
        image::open(&canonical).map_err(|e| format!("image failed to decode ({source}): {e}"))?;
        let destination = asset_root.join(&relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let bytes = fs::read(&canonical).map_err(|e| e.to_string())?;
        fs::write(&destination, &bytes).map_err(|error| error.to_string())?;
        fingerprints.push((
            canonical.to_string_lossy().into_owned(),
            format!("{:x}", Sha256::digest(&bytes)),
        ));
        image_uris.push(uri);
    }

    let manifest_uri = image_uris
        .first()
        .cloned()
        .unwrap_or_else(|| AssetUri::parse(DEFAULT_FONT_URI).expect("default font URI"));
    ProjectManifest::new("ZuiNativeVisualAcceptance", manifest_uri, 1)
        .save(paths.manifest_path())
        .map_err(|error| error.to_string())?;

    let manager = Arc::new(ProjectAssetManager::default());
    manager
        .open_project(work_root.to_string_lossy().as_ref())
        .map_err(|error| error.to_string())?;

    for uri in image_uris {
        let id = manager
            .resolve_asset_id(&uri)
            .ok_or_else(|| format!("image was not imported: {uri}"))?;
        manager
            .load_texture_asset(id)
            .map_err(|error| format!("image failed to decode ({uri}): {error}"))?;
    }
    for font in font_uris {
        let font_uri = font_asset_uri(&font)?;
        let font_id = manager
            .resolve_asset_id(&font_uri)
            .ok_or_else(|| format!("font was not imported: {font}"))?;
        manager
            .load_font_asset(font_id)
            .map_err(|error| format!("font failed to load ({font}): {error}"))?;
    }

    Ok(PreparedAssets {
        manager,
        fingerprints,
    })
}

fn copy_default_font(repo_root: &Path, asset_root: &Path) -> Result<Vec<(String, String)>, String> {
    let source = repo_root
        .join("zircon_runtime")
        .join("assets")
        .join("fonts");
    let destination = asset_root.join("fonts");
    fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
    let mut fingerprints = Vec::new();
    copy_font_directory(repo_root, &source, &destination, &mut fingerprints)?;
    let manifest = fs::read_to_string(destination.join("default.font.toml"))
        .map_err(|error| error.to_string())?;
    FontAsset::from_toml_str(&manifest).map_err(|error| error.to_string())?;
    Ok(fingerprints)
}

fn contained_source(repo_root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root = repo_root.canonicalize().map_err(|e| e.to_string())?;
    let source = path.canonicalize().map_err(|e| e.to_string())?;
    if !source.starts_with(&root) {
        return Err(format!(
            "asset source escapes repository: {}",
            path.display()
        ));
    }
    Ok(source)
}

fn copy_font_directory(
    repo_root: &Path,
    source: &Path,
    destination: &Path,
    fingerprints: &mut Vec<(String, String)>,
) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    let mut entries = fs::read_dir(source)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = contained_source(repo_root, &entry.path())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_symlink() {
            return Err(format!("symlink in font package: {}", path.display()));
        }
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy_font_directory(repo_root, &path, &target, fingerprints)?;
        } else {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            fs::write(target, &bytes).map_err(|e| e.to_string())?;
            fingerprints.push((
                path.to_string_lossy().into_owned(),
                format!("{:x}", Sha256::digest(&bytes)),
            ));
        }
    }
    Ok(())
}

fn nearest_assets_dir(source_path: &Path) -> Option<PathBuf> {
    let mut current = source_path.parent();
    while let Some(path) = current {
        let candidate = path.join("assets");
        if candidate.is_dir() {
            return Some(candidate);
        }
        current = path.parent();
    }
    None
}

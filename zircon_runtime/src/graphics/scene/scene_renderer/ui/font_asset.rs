use std::path::{Path, PathBuf};

use crate::asset::{AssetUri, FontAsset, ProjectAssetManager};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoadedUiFontManifest {
    pub(crate) source_path: PathBuf,
    pub(crate) family: Option<String>,
    pub(crate) render_mode: Option<UiTextRenderMode>,
}

#[allow(dead_code)]
pub(crate) fn load_ui_font_manifest(asset_ref: &str) -> Option<LoadedUiFontManifest> {
    load_ui_font_manifest_with_asset_manager(asset_ref, None)
}

pub(crate) fn load_ui_font_manifest_with_asset_manager(
    asset_ref: &str,
    asset_manager: Option<&ProjectAssetManager>,
) -> Option<LoadedUiFontManifest> {
    if asset_ref.ends_with(".toml") {
        if let Some(manifest) =
            asset_manager.and_then(|manager| load_project_ui_font_manifest(manager, asset_ref))
        {
            return Some(manifest);
        }

        let manifest_path = resolve_font_asset_path(asset_ref)?;
        let manifest = std::fs::read_to_string(&manifest_path).ok()?;
        let manifest = FontAsset::from_toml_str(&manifest).ok()?;
        let source_path =
            resolve_manifest_source_path(asset_ref, &manifest_path, manifest.source.as_str())?;
        return Some(LoadedUiFontManifest {
            source_path,
            family: manifest.family,
            render_mode: manifest.render_mode,
        });
    }

    let source_path = resolve_font_asset_path(asset_ref)?;
    Some(LoadedUiFontManifest {
        source_path,
        family: None,
        render_mode: None,
    })
}

fn load_project_ui_font_manifest(
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> Option<LoadedUiFontManifest> {
    if !asset_ref.starts_with("res://") {
        return None;
    }

    let uri = AssetUri::parse(asset_ref).ok()?;
    let project = asset_manager.current_project_manager()?;
    let manifest_id = asset_manager.resolve_asset_id(&uri)?;
    let manifest = asset_manager.load_font_asset(manifest_id).ok()?;
    let manifest_path = project.paths().assets_root().join(uri.path());
    let source_path = resolve_manifest_source_path_with_allowed_root(
        &manifest_path,
        manifest.source.as_str(),
        project.paths().assets_root(),
    )?;
    Some(LoadedUiFontManifest {
        source_path,
        family: manifest.family,
        render_mode: manifest.render_mode,
    })
}

fn resolve_font_asset_path(asset_ref: &str) -> Option<PathBuf> {
    if let Some(relative) = asset_ref.strip_prefix("res://") {
        return Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(relative),
        );
    }
    let path = PathBuf::from(asset_ref);
    path.is_absolute()
        .then_some(path)
        .or(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join(asset_ref)))
}

fn resolve_manifest_source_path(
    asset_ref: &str,
    manifest_path: &Path,
    source: &str,
) -> Option<PathBuf> {
    let allowed_root = if asset_ref.starts_with("res://") {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    } else {
        manifest_path.parent()?.to_path_buf()
    };
    resolve_manifest_source_path_with_allowed_root(manifest_path, source, allowed_root)
}

fn resolve_manifest_source_path_with_allowed_root(
    manifest_path: &Path,
    source: &str,
    allowed_root: impl AsRef<Path>,
) -> Option<PathBuf> {
    let source = source.trim();
    if source.is_empty() {
        return None;
    }

    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return None;
    }

    let resolved = manifest_path.parent()?.join(&source_path);
    let canonical_allowed_root = std::fs::canonicalize(allowed_root).ok()?;
    let canonical_resolved = std::fs::canonicalize(&resolved).ok()?;

    canonical_resolved
        .starts_with(&canonical_allowed_root)
        .then_some(resolved)
}

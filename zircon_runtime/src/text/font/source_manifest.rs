use std::path::{Path, PathBuf};

use crate::asset::project::AssetMetaDocument;
use crate::asset::{AssetUri, AssetUuid, FontAsset, ProjectAssetManager};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadedTextFontSource {
    pub(crate) source_path: PathBuf,
    pub(crate) asset: Option<FontAsset>,
    pub(crate) family: Option<String>,
    pub(crate) face_index: u32,
    pub(crate) asset_uuid: Option<AssetUuid>,
}

pub(crate) fn load_text_font_source(
    asset_ref: &str,
    asset_manager: Option<&ProjectAssetManager>,
) -> Option<LoadedTextFontSource> {
    if asset_ref.ends_with(".toml") {
        if let Some(source) =
            asset_manager.and_then(|manager| load_project_font_source(manager, asset_ref))
        {
            return Some(source);
        }

        let manifest_path = resolve_font_asset_path(asset_ref)?;
        let manifest = std::fs::read_to_string(&manifest_path).ok()?;
        let manifest = FontAsset::from_toml_str(&manifest).ok()?;
        let source_path =
            resolve_manifest_source_path(asset_ref, &manifest_path, manifest.source.as_str())?;
        let family = manifest.family.clone();
        let face_index = manifest.face_index;
        let asset_uuid = font_manifest_asset_uuid(&manifest_path);
        return Some(LoadedTextFontSource {
            source_path,
            asset: Some(manifest),
            family,
            face_index,
            asset_uuid,
        });
    }

    Some(LoadedTextFontSource {
        source_path: resolve_font_asset_path(asset_ref)?,
        asset: None,
        family: None,
        face_index: 0,
        asset_uuid: None,
    })
}

fn load_project_font_source(
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> Option<LoadedTextFontSource> {
    if !asset_ref.starts_with("res://") {
        return None;
    }

    let uri = AssetUri::parse(asset_ref).ok()?;
    let project = asset_manager.current_project_manager()?;
    let manifest_id = asset_manager.resolve_asset_id(&uri)?;
    let manifest = asset_manager.load_font_asset(manifest_id).ok()?;
    let manifest_path = project.source_path_for_uri(&uri).ok()?;
    let asset_root = project
        .project_asset_roots()
        .iter()
        .find(|root| manifest_path.starts_with(root))?;
    let source_path = resolve_manifest_source_path_with_allowed_root(
        &manifest_path,
        manifest.source.as_str(),
        asset_root,
    )?;
    let family = manifest.family.clone();
    let face_index = manifest.face_index;
    let asset_uuid = font_manifest_asset_uuid(&manifest_path);
    Some(LoadedTextFontSource {
        source_path,
        asset: Some(manifest),
        family,
        face_index,
        asset_uuid,
    })
}

fn font_manifest_asset_uuid(manifest_path: &Path) -> Option<AssetUuid> {
    let file_name = manifest_path.file_name()?.to_str()?;
    let meta_path = manifest_path.with_file_name(format!("{file_name}.zmeta"));
    AssetMetaDocument::load(meta_path)
        .ok()
        .map(|meta| meta.uuid)
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

#[cfg(test)]
mod tests;

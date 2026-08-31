use std::path::{Path, PathBuf};

use crate::asset::project::AssetMetaDocument;
use crate::asset::{
    AssetUri, AssetUuid, FontAsset, FontBlobArtifact, ProjectAssetManager, runtime_asset_path,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadedTextFontSource {
    pub(crate) source_path: PathBuf,
    pub(crate) cooked_blob: Option<FontBlobArtifact>,
    pub(crate) asset: Option<FontAsset>,
    pub(crate) family: Option<String>,
    pub(crate) face_index: u32,
    pub(crate) asset_uuid: Option<AssetUuid>,
}

/// Stable, bounded diagnostics for a font reference before bytes enter `FontDatabase`.
///
/// These categories intentionally omit host IO/parser details: callers cache this value and may
/// surface it in runtime diagnostics without retaining platform error objects per entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum FontLoadError {
    #[error("font asset reference is invalid")]
    InvalidAssetReference,
    #[error("project font URI is invalid")]
    InvalidProjectUri,
    #[error("project font asset is unavailable")]
    ProjectAssetUnavailable,
    #[error("project font artifact does not contain its cooked font payload")]
    ProjectCookedPayloadUnavailable,
    #[error("font manifest could not be read: {0:?}")]
    ManifestReadFailed(FontLoadIoFailure),
    #[error("font manifest could not be parsed")]
    ManifestParseFailed,
    #[error("font manifest source is empty")]
    EmptyManifestSource,
    #[error("font manifest source must be relative")]
    AbsoluteManifestSource,
    #[error("font manifest allowed root is unavailable: {0:?}")]
    AllowedRootUnavailable(FontLoadIoFailure),
    #[error("font manifest source is unavailable: {0:?}")]
    ManifestSourceUnavailable(FontLoadIoFailure),
    #[error("font manifest source escapes its allowed root")]
    SourceOutsideAllowedRoot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontLoadIoFailure {
    NotFound,
    PermissionDenied,
    Other,
}

impl From<std::io::ErrorKind> for FontLoadIoFailure {
    fn from(kind: std::io::ErrorKind) -> Self {
        match kind {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            _ => Self::Other,
        }
    }
}

pub(crate) fn load_text_font_source(
    asset_ref: &str,
    asset_manager: Option<&ProjectAssetManager>,
) -> Result<LoadedTextFontSource, FontLoadError> {
    if asset_ref.ends_with(".toml") {
        if asset_ref.starts_with("res://") {
            if let Some(asset_manager) = asset_manager {
                if asset_manager.current_project_manager().is_some() {
                    return load_project_font_source(asset_manager, asset_ref);
                }
            }
        }

        let manifest_path =
            resolve_font_asset_path(asset_ref).ok_or(FontLoadError::InvalidAssetReference)?;
        let manifest = std::fs::read_to_string(&manifest_path)
            .map_err(|error| FontLoadError::ManifestReadFailed(error.kind().into()))?;
        let manifest =
            FontAsset::from_toml_str(&manifest).map_err(|_| FontLoadError::ManifestParseFailed)?;
        let source_path =
            resolve_manifest_source_path(asset_ref, &manifest_path, manifest.source.as_str())?;
        let family = manifest.family.clone();
        let face_index = manifest.face_index;
        let asset_uuid = font_manifest_asset_uuid(&manifest_path);
        return Ok(LoadedTextFontSource {
            source_path,
            cooked_blob: None,
            asset: Some(manifest),
            family,
            face_index,
            asset_uuid,
        });
    }

    Ok(LoadedTextFontSource {
        source_path: resolve_font_asset_path(asset_ref)
            .ok_or(FontLoadError::InvalidAssetReference)?,
        cooked_blob: None,
        asset: None,
        family: None,
        face_index: 0,
        asset_uuid: None,
    })
}

fn load_project_font_source(
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> Result<LoadedTextFontSource, FontLoadError> {
    if !asset_ref.starts_with("res://") {
        return Err(FontLoadError::InvalidProjectUri);
    }

    let uri = AssetUri::parse(asset_ref).map_err(|_| FontLoadError::InvalidProjectUri)?;
    let project = asset_manager
        .current_project_manager()
        .ok_or(FontLoadError::ProjectAssetUnavailable)?;
    let registry_entry = project
        .asset_registry()
        .entry_by_path(&uri)
        .ok_or(FontLoadError::ProjectAssetUnavailable)?;
    let asset_uuid = registry_entry.uuid();
    let manifest_id = project
        .asset_registry()
        .resolve_asset_id_by_uuid(asset_uuid)
        .map_err(|_| FontLoadError::ProjectAssetUnavailable)?;
    let manifest = asset_manager
        .load_font_asset(manifest_id)
        .map_err(|_| FontLoadError::ProjectAssetUnavailable)?;
    let cooked_blob = manifest
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.cooked_blob.clone())
        .ok_or(FontLoadError::ProjectCookedPayloadUnavailable)?;
    // The artifact URI is a stable logical source key only. Its source bytes
    // enter FontDatabase from the cooked payload below, never via filesystem
    // reopening of `FontAsset::source`.
    let source_path = cooked_font_asset_source_key(&uri);
    let family = manifest.family.clone();
    let face_index = manifest.face_index;
    Ok(LoadedTextFontSource {
        source_path,
        cooked_blob: Some(cooked_blob),
        asset: Some(manifest),
        family,
        face_index,
        asset_uuid: Some(asset_uuid),
    })
}

pub(super) fn cooked_font_asset_source_key(uri: &AssetUri) -> PathBuf {
    Path::new("cooked-font").join(uri.path())
}

fn font_manifest_asset_uuid(manifest_path: &Path) -> Option<AssetUuid> {
    let file_name = manifest_path.file_name()?.to_str()?;
    let meta_path = manifest_path.with_file_name(format!("{file_name}.zmeta"));
    AssetMetaDocument::load(meta_path)
        .ok()
        .map(|meta| meta.uuid)
}

fn resolve_font_asset_path(asset_ref: &str) -> Option<PathBuf> {
    resolve_font_asset_path_with(asset_ref, runtime_font_asset_path)
}

fn runtime_font_asset_path(relative: &Path) -> PathBuf {
    runtime_asset_path(relative)
}

fn resolve_font_asset_path_with(
    asset_ref: &str,
    resolve_runtime_asset: impl FnOnce(&Path) -> PathBuf,
) -> Option<PathBuf> {
    if let Some(relative) = asset_ref.strip_prefix("res://") {
        return Some(resolve_runtime_asset(Path::new(relative)));
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
) -> Result<PathBuf, FontLoadError> {
    let allowed_root = if asset_ref.starts_with("res://") {
        runtime_asset_path("")
    } else {
        manifest_path
            .parent()
            .ok_or(FontLoadError::InvalidAssetReference)?
            .to_path_buf()
    };
    resolve_manifest_source_path_with_allowed_root(manifest_path, source, allowed_root)
}

fn resolve_manifest_source_path_with_allowed_root(
    manifest_path: &Path,
    source: &str,
    allowed_root: impl AsRef<Path>,
) -> Result<PathBuf, FontLoadError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(FontLoadError::EmptyManifestSource);
    }

    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return Err(FontLoadError::AbsoluteManifestSource);
    }

    let resolved = manifest_path
        .parent()
        .ok_or(FontLoadError::InvalidAssetReference)?
        .join(&source_path);
    let canonical_allowed_root = std::fs::canonicalize(allowed_root)
        .map_err(|error| FontLoadError::AllowedRootUnavailable(error.kind().into()))?;
    let canonical_resolved = std::fs::canonicalize(&resolved)
        .map_err(|error| FontLoadError::ManifestSourceUnavailable(error.kind().into()))?;
    canonical_resolved
        .starts_with(&canonical_allowed_root)
        .then_some(resolved)
        .ok_or(FontLoadError::SourceOutsideAllowedRoot)
}

#[cfg(test)]
mod tests;

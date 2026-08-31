use std::path::{Component, Path, PathBuf};

use image::RgbaImage;
use zircon_runtime::asset::{AssetImportContext, AssetImportError, AssetReference, AssetUri};

pub(crate) struct DecodedManifestImage {
    pub(crate) reference: AssetReference,
    pub(crate) rgba: RgbaImage,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ResolvedManifestSource {
    pub(crate) path: PathBuf,
    pub(crate) uri: AssetUri,
}

pub(crate) fn decode_manifest_image(
    context: &AssetImportContext,
    source: &str,
) -> Result<DecodedManifestImage, AssetImportError> {
    let resolved = resolve_manifest_source(context, source)?;
    let bytes = std::fs::read(&resolved.path).map_err(|error| {
        AssetImportError::Parse(format!(
            "read texture manifest source {}: {error}",
            resolved.path.display()
        ))
    })?;
    let rgba = image::load_from_memory(&bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode texture manifest source {}: {error}",
            resolved.path.display()
        ))
    })?;
    Ok(DecodedManifestImage {
        reference: AssetReference::from_locator(resolved.uri),
        rgba: rgba.to_rgba8(),
    })
}

pub(crate) fn resolve_manifest_source(
    context: &AssetImportContext,
    source: &str,
) -> Result<ResolvedManifestSource, AssetImportError> {
    let relative = validated_relative_source(source)?;
    let parent = context.source_path.parent().ok_or_else(|| {
        AssetImportError::Parse(format!(
            "texture manifest {} has no source directory",
            context.source_path.display()
        ))
    })?;
    let filesystem_path = parent.join(relative);
    let uri_parent = Path::new(context.uri.path())
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let uri_path = uri_parent
        .join(relative)
        .to_string_lossy()
        .replace('\\', "/");
    let uri = AssetUri::new(context.uri.scheme(), uri_path, None).map_err(|error| {
        AssetImportError::Parse(format!(
            "invalid texture manifest source `{source}`: {error}"
        ))
    })?;
    Ok(ResolvedManifestSource {
        path: filesystem_path,
        uri,
    })
}

fn validated_relative_source(source: &str) -> Result<&Path, AssetImportError> {
    let path = Path::new(source);
    let valid = !source.trim().is_empty()
        && !source.contains("://")
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
    if !valid {
        return Err(AssetImportError::Parse(format!(
            "texture manifest source must be a project-relative path without parent traversal: `{source}`"
        )));
    }
    Ok(path)
}

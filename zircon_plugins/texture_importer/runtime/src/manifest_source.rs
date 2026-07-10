use std::path::{Component, Path, PathBuf};

use image::RgbaImage;
use zircon_runtime::asset::{AssetImportContext, AssetImportError, AssetReference, AssetUri};

pub(crate) struct DecodedManifestImage {
    pub(crate) reference: AssetReference,
    pub(crate) rgba: RgbaImage,
}

pub(crate) fn decode_manifest_image(
    context: &AssetImportContext,
    source: &str,
) -> Result<DecodedManifestImage, AssetImportError> {
    let path = manifest_source_path(context, source)?;
    let bytes = std::fs::read(&path).map_err(|error| {
        AssetImportError::Parse(format!(
            "read texture manifest source {}: {error}",
            path.display()
        ))
    })?;
    let rgba = image::load_from_memory(&bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode texture manifest source {}: {error}",
            path.display()
        ))
    })?;
    Ok(DecodedManifestImage {
        reference: AssetReference::from_locator(manifest_source_uri(context, source)?),
        rgba: rgba.to_rgba8(),
    })
}

fn manifest_source_path(
    context: &AssetImportContext,
    source: &str,
) -> Result<PathBuf, AssetImportError> {
    let relative = validated_relative_source(source)?;
    let parent = context.source_path.parent().ok_or_else(|| {
        AssetImportError::Parse(format!(
            "texture manifest {} has no source directory",
            context.source_path.display()
        ))
    })?;
    Ok(parent.join(relative))
}

fn manifest_source_uri(
    context: &AssetImportContext,
    source: &str,
) -> Result<AssetUri, AssetImportError> {
    if source.contains("://") {
        return AssetUri::parse(source).map_err(|error| {
            AssetImportError::Parse(format!(
                "invalid texture manifest source URI `{source}`: {error}"
            ))
        });
    }
    let relative = validated_relative_source(source)?;
    let manifest_uri = context.uri.to_string();
    let (scheme, _) = manifest_uri.split_once("://").ok_or_else(|| {
        AssetImportError::Parse(format!("invalid texture manifest URI `{manifest_uri}`"))
    })?;
    let parent = Path::new(context.uri.path())
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let path = parent.join(relative).to_string_lossy().replace('\\', "/");
    AssetUri::parse(&format!("{scheme}://{path}")).map_err(|error| {
        AssetImportError::Parse(format!(
            "invalid texture manifest source `{source}`: {error}"
        ))
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

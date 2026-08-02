mod parse_sfnt;

use std::path::{Path, PathBuf};

use crate::asset::assets::{FontAsset, ImportedAsset, decode_font_source};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri};

pub(crate) fn import_font_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let mut asset = FontAsset::from_toml_str(&document).map_err(AssetImportError::FontDocument)?;
    let source_path = resolve_manifest_source_path(&context.source_path, &asset.source)?;
    let source_bytes =
        std::fs::read(&source_path).map_err(|source| AssetImportError::FontSourceIo {
            path: source_path.clone(),
            source,
        })?;
    let source =
        decode_font_source(source_bytes).map_err(|source| AssetImportError::FontSourceDecode {
            path: source_path.clone(),
            source,
        })?;
    let metadata = parse_sfnt::parse_font_metadata(&source).map_err(|source| {
        AssetImportError::FontMetadata {
            path: source_path.clone(),
            source,
        }
    })?;

    apply_parsed_defaults(&mut asset, metadata);

    let dependency = dependency_uri_for_source(&context.uri, &asset.source);
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Font(asset));
    if let Some(dependency) = dependency {
        outcome = outcome.with_dependency(dependency);
    }
    Ok(outcome)
}

fn apply_parsed_defaults(asset: &mut FontAsset, metadata: crate::asset::assets::FontAssetMetadata) {
    if asset.family_members.is_empty() {
        asset.family_members = metadata
            .faces
            .iter()
            .filter_map(|face| face.family_member())
            .collect();
    }
    if asset.variable_instances.is_empty() {
        asset.variable_instances = metadata
            .faces
            .iter()
            .find(|face| face.face_index == asset.face_index)
            .map(|face| face.named_instances.clone())
            .unwrap_or_default();
    }
    if asset.family.is_none() {
        asset.family = metadata
            .faces
            .iter()
            .find(|face| face.face_index == asset.face_index)
            .and_then(|face| face.family.clone())
            .or_else(|| metadata.faces.first().and_then(|face| face.family.clone()));
    }
    asset.metadata = Some(metadata);
}

fn resolve_manifest_source_path(
    manifest_path: &Path,
    source: &str,
) -> Result<PathBuf, AssetImportError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(AssetImportError::FontSourcePath {
            manifest_path: manifest_path.to_path_buf(),
            reason: "source is empty",
        });
    }

    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return Err(AssetImportError::FontSourcePath {
            manifest_path: manifest_path.to_path_buf(),
            reason: "source must be relative to the manifest",
        });
    }

    Ok(manifest_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(source_path))
}

fn dependency_uri_for_source(manifest_uri: &AssetUri, source: &str) -> Option<AssetUri> {
    if !manifest_uri.to_string().starts_with("res://") {
        return None;
    }
    let parent = Path::new(manifest_uri.path()).parent()?;
    let dependency_path = parent
        .join(source.trim())
        .to_string_lossy()
        .replace('\\', "/");
    AssetUri::parse(&format!("res://{dependency_path}")).ok()
}

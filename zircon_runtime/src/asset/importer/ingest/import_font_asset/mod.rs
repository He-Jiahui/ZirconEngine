mod parse_sfnt;

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::asset::assets::{
    decode_font_source, validate_font_source_file_len, FontAsset, FontBlobArtifact, ImportedAsset,
};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri};

pub(crate) fn import_font_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let mut asset = FontAsset::from_toml_str(document).map_err(AssetImportError::FontDocument)?;
    let source_path = resolve_manifest_source_path(&context.source_path, &asset.source)?;
    let source_size = std::fs::metadata(&source_path)
        .map_err(|source| AssetImportError::FontSourceIo {
            path: source_path.clone(),
            source,
        })?
        .len();
    validate_font_source_file_len(source_size).map_err(|source| {
        AssetImportError::FontSourceBudget {
            path: source_path.clone(),
            source,
        }
    })?;
    let source_bytes =
        std::fs::read(&source_path).map_err(|source| AssetImportError::FontSourceIo {
            path: source_path.clone(),
            source,
        })?;
    let source_sha256: [u8; 32] = Sha256::digest(&source_bytes).into();
    let source_resource_path = std::fs::canonicalize(&source_path).ok();
    let source = decode_font_source(source_bytes).map_err(|source| match source {
        crate::asset::assets::FontSourceDecodeError::Budget(source) => {
            AssetImportError::FontSourceBudget {
                path: source_path.clone(),
                source,
            }
        }
        source => AssetImportError::FontSourceDecode {
            path: source_path.clone(),
            source,
        },
    })?;
    let metadata = parse_sfnt::parse_font_metadata(&source).map_err(|source| {
        AssetImportError::FontMetadata {
            path: source_path.clone(),
            source,
        }
    })?;
    let source_sha256 = source_resource_path.as_ref().map(|_| source_sha256);
    let cooked_blob = FontBlobArtifact::from_decoded_source_file(
        source.source_format(),
        source.into_bytes(),
        source_resource_path,
        source_sha256,
    );

    apply_parsed_defaults(&mut asset, metadata, cooked_blob);

    let dependency = dependency_uri_for_source(&context.uri, &asset.source);
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Font(asset));
    if let Some(dependency) = dependency {
        outcome = outcome.with_dependency(dependency);
    }
    Ok(outcome)
}

fn apply_parsed_defaults(
    asset: &mut FontAsset,
    mut metadata: crate::asset::assets::FontAssetMetadata,
    cooked_blob: FontBlobArtifact,
) {
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
    metadata.cooked_blob = Some(cooked_blob);
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

#[cfg(test)]
#[path = "tests/mod_plugins07_font_manifest_source_tests.rs"]
mod plugins07_font_manifest_source_tests;

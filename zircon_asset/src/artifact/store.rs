use std::fs;
use std::path::PathBuf;

use crate::{
    AssetImportError, AssetKind, AssetMetadata, AssetUri, AssetUriScheme, ImportedAsset,
    ProjectPaths,
};

#[derive(Clone, Debug, Default)]
pub struct ArtifactStore;

impl ArtifactStore {
    pub fn write(
        &self,
        paths: &ProjectPaths,
        metadata: &AssetMetadata,
        asset: &ImportedAsset,
    ) -> Result<AssetUri, AssetImportError> {
        let relative_path = format!(
            "{}/{}.json",
            asset_kind_directory(metadata.kind),
            metadata.id()
        );
        let artifact_uri = AssetUri::parse(&format!("lib://{relative_path}"))?;
        let artifact_path = resolve_library_path(paths, &artifact_uri)?;
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serde_json::to_vec_pretty(asset)?;
        fs::write(&artifact_path, payload)?;
        Ok(artifact_uri)
    }

    pub fn read(
        &self,
        paths: &ProjectPaths,
        artifact_uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let artifact_path = resolve_library_path(paths, artifact_uri)?;
        let document = fs::read_to_string(artifact_path)?;
        Ok(serde_json::from_str(&document)?)
    }
}

fn resolve_library_path(
    paths: &ProjectPaths,
    artifact_uri: &AssetUri,
) -> Result<PathBuf, AssetImportError> {
    if artifact_uri.scheme() != AssetUriScheme::Library {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "artifact uri must use lib:// scheme: {artifact_uri}"
        )));
    }
    Ok(paths.library_root().join(artifact_uri.path()))
}

fn asset_kind_directory(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Texture => "textures",
        AssetKind::Shader => "shaders",
        AssetKind::Material => "materials",
        AssetKind::Scene => "scenes",
        AssetKind::Model => "models",
        AssetKind::UiLayout => "ui/layouts",
        AssetKind::UiWidget => "ui/widgets",
        AssetKind::UiStyle => "ui/styles",
    }
}

use crate::asset::{ArtifactStore, AssetId, AssetImportError, AssetUri, ImportedAsset};

use super::super::ProjectPaths;

use super::ProjectManager;

impl ProjectManager {
    pub fn load_artifact(&self, uri: &AssetUri) -> Result<ImportedAsset, AssetImportError> {
        self.load_artifact_with_raw_payload_limit(uri, u64::MAX)
    }

    pub(crate) fn load_artifact_with_raw_payload_limit(
        &self,
        uri: &AssetUri,
        max_raw_payload_bytes: u64,
    ) -> Result<ImportedAsset, AssetImportError> {
        let metadata = self.registry.get_by_locator(uri).ok_or_else(|| {
            if let Some((source_uri, label)) = split_labeled_uri(uri) {
                if self.registry.get_by_locator(&source_uri).is_some() {
                    return AssetImportError::MissingAssetLabel { source_uri, label };
                }
            }
            AssetImportError::Parse(format!("missing asset metadata for source uri {uri}"))
        })?;
        let artifact_uri = metadata.artifact_locator().ok_or_else(|| {
            AssetImportError::Parse(format!("missing artifact uri for source uri {uri}"))
        })?;
        self.artifact_store
            .read_with_raw_payload_limit(&self.paths, artifact_uri, max_raw_payload_bytes)
            .map_err(|error| match error {
                error @ AssetImportError::ArtifactRawPayloadLimitExceeded { .. } => error,
                error => AssetImportError::Parse(format!(
                    "read artifact {artifact_uri} for source uri {uri}: {error}"
                )),
            })
    }

    pub fn load_artifact_by_id(&self, id: AssetId) -> Result<ImportedAsset, AssetImportError> {
        self.prepare_artifact_read_by_id(id)?.read()
    }

    pub(crate) fn prepare_artifact_read_by_id(
        &self,
        id: AssetId,
    ) -> Result<PreparedProjectArtifactRead, AssetImportError> {
        let metadata = self.registry.get(id).ok_or_else(|| {
            AssetImportError::Parse(format!("missing asset metadata for asset id {id}"))
        })?;
        let artifact_uri = metadata.artifact_locator().cloned().ok_or_else(|| {
            AssetImportError::Parse(format!("missing artifact uri for asset id {id}"))
        })?;
        Ok(PreparedProjectArtifactRead {
            artifact_store: self.artifact_store.clone(),
            paths: self.paths.clone(),
            artifact_uri,
            asset_id: id,
        })
    }
}

pub(crate) struct PreparedProjectArtifactRead {
    artifact_store: ArtifactStore,
    paths: ProjectPaths,
    artifact_uri: AssetUri,
    asset_id: AssetId,
}

impl PreparedProjectArtifactRead {
    pub(crate) fn read(self) -> Result<ImportedAsset, AssetImportError> {
        self.artifact_store
            .read(&self.paths, &self.artifact_uri)
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "read artifact {} for asset id {}: {error}",
                    self.artifact_uri, self.asset_id
                ))
            })
    }
}

fn split_labeled_uri(uri: &AssetUri) -> Option<(AssetUri, String)> {
    let label = uri.label()?.to_string();
    let source_text = uri.to_string().split_once('#')?.0.to_string();
    AssetUri::parse(&source_text)
        .ok()
        .map(|source_uri| (source_uri, label))
}

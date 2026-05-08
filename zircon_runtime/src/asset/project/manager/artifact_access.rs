use crate::asset::{AssetId, AssetImportError, AssetUri, ImportedAsset};

use super::ProjectManager;

impl ProjectManager {
    pub fn load_artifact(&self, uri: &AssetUri) -> Result<ImportedAsset, AssetImportError> {
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
        self.artifact_store.read(&self.paths, artifact_uri)
    }

    pub fn load_artifact_by_id(&self, id: AssetId) -> Result<ImportedAsset, AssetImportError> {
        let metadata = self.registry.get(id).ok_or_else(|| {
            AssetImportError::Parse(format!("missing asset metadata for asset id {id}"))
        })?;
        let artifact_uri = metadata.artifact_locator().ok_or_else(|| {
            AssetImportError::Parse(format!("missing artifact uri for asset id {id}"))
        })?;
        self.artifact_store.read(&self.paths, artifact_uri)
    }
}

fn split_labeled_uri(uri: &AssetUri) -> Option<(AssetUri, String)> {
    let label = uri.label()?.to_string();
    let source_text = uri.to_string().split_once('#')?.0.to_string();
    AssetUri::parse(&source_text)
        .ok()
        .map(|source_uri| (source_uri, label))
}

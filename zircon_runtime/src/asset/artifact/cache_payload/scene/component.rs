use serde::{Deserialize, Serialize};

use super::super::ArtifactCacheJsonValue;
use crate::asset::{AssetImportError, AssetReference, SceneComponentAssetRecord};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneComponentAsset {
    type_id: String,
    schema_id: String,
    schema_version: u32,
    provider_id: String,
    payload: ArtifactCacheJsonValue,
    references: Vec<AssetReference>,
}

impl From<&SceneComponentAssetRecord> for ArtifactCacheSceneComponentAsset {
    fn from(row: &SceneComponentAssetRecord) -> Self {
        Self {
            type_id: row.type_id.clone(),
            schema_id: row.schema_id.clone(),
            schema_version: row.schema_version,
            provider_id: row.provider_id.clone(),
            payload: ArtifactCacheJsonValue::from_json(&row.payload),
            references: row.references.clone(),
        }
    }
}

impl ArtifactCacheSceneComponentAsset {
    pub(super) fn into_asset(self) -> Result<SceneComponentAssetRecord, AssetImportError> {
        let record = SceneComponentAssetRecord {
            type_id: self.type_id,
            schema_id: self.schema_id,
            schema_version: self.schema_version,
            provider_id: self.provider_id,
            payload: self.payload.into_json()?,
            references: self.references,
        };
        record.validate_references().map_err(|error| {
            AssetImportError::Parse(format!(
                "reject cached scene component row with malformed asset references: {error}"
            ))
        })?;
        Ok(record)
    }
}

mod entity;
mod physics;
mod rendering;
mod script;

use serde::{Deserialize, Serialize};

use self::entity::ArtifactCacheSceneEntityAsset;
use crate::asset::{AssetImportError, SceneAsset};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::asset::artifact) struct ArtifactCacheSceneAsset {
    entities: Vec<ArtifactCacheSceneEntityAsset>,
}

impl From<&SceneAsset> for ArtifactCacheSceneAsset {
    fn from(asset: &SceneAsset) -> Self {
        Self {
            entities: asset
                .entities
                .iter()
                .map(ArtifactCacheSceneEntityAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneAsset {
    pub(super) fn into_asset(self) -> Result<SceneAsset, AssetImportError> {
        Ok(SceneAsset {
            entities: self
                .entities
                .into_iter()
                .map(ArtifactCacheSceneEntityAsset::into_asset)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

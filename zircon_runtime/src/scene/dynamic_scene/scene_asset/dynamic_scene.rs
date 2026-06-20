use crate::asset::{project::ProjectManager, AssetUri, SceneAsset};
use crate::scene::serializer::SceneAssetSerializer;

use super::error::scene_asset_error;
use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};

impl DynamicScene {
    pub fn from_scene_asset(
        project: &ProjectManager,
        asset: &SceneAsset,
    ) -> Result<Self, DynamicSceneError> {
        let world = SceneAssetSerializer::instantiate_world(project, asset)
            .map_err(|error| scene_asset_error("instantiate scene asset", error))?;
        Self::from_world(&world)
    }

    pub fn from_scene_asset_uri(
        project: &ProjectManager,
        uri: &AssetUri,
    ) -> Result<Self, DynamicSceneError> {
        let world = SceneAssetSerializer::load_world(project, uri)
            .map_err(|error| scene_asset_error(format!("load scene asset {uri}"), error))?;
        Self::from_world(&world)
    }
}

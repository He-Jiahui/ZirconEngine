use crate::asset::{project::ProjectManager, AssetUri, SceneAsset};
use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError, PreparedDynamicSceneSpawn};

impl PreparedDynamicSceneSpawn {
    pub fn from_scene_asset(
        project: &ProjectManager,
        asset: &SceneAsset,
    ) -> Result<Self, DynamicSceneError> {
        Self::new(DynamicScene::from_scene_asset(project, asset)?)
    }

    pub fn from_scene_asset_uri(
        project: &ProjectManager,
        uri: &AssetUri,
    ) -> Result<Self, DynamicSceneError> {
        Self::new(DynamicScene::from_scene_asset_uri(project, uri)?)
    }
}

use crate::asset::{project::ProjectManager, AssetUri, SceneAsset};
use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError, PreparedDynamicSceneSpawn};

impl PreparedDynamicSceneSpawn {
    /// 获取并检查场景载荷；目标世界的类型兼容性与实体重映射仍在后续生成预检时验证。
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

use crate::asset::assets::SceneAsset;
use crate::asset::project::ProjectManager;
use crate::core::resource::ResourceLocator;
use crate::scene::world::{SceneProjectError, World};

#[derive(Debug, Default)]
pub struct SceneAssetSerializer;

impl SceneAssetSerializer {
    pub fn load_world(
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<World, SceneProjectError> {
        World::load_scene_from_uri(project, uri)
    }

    pub(crate) fn load_world_with_raw_payload_limit(
        project: &ProjectManager,
        uri: &ResourceLocator,
        max_raw_payload_bytes: u64,
    ) -> Result<World, SceneProjectError> {
        World::load_scene_from_uri_with_raw_payload_limit(project, uri, max_raw_payload_bytes)
    }

    pub fn instantiate_world(
        project: &ProjectManager,
        asset: &SceneAsset,
    ) -> Result<World, SceneProjectError> {
        World::from_scene_asset(project, asset)
    }

    pub fn serialize_world(
        project: &ProjectManager,
        world: &World,
    ) -> Result<SceneAsset, SceneProjectError> {
        world.to_scene_asset(project)
    }
}

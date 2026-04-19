//! Scene asset serializer boundary between `SceneAsset` and `World`.

use zircon_asset::assets::SceneAsset;
use zircon_asset::project::ProjectManager;
use zircon_resource::ResourceLocator;

use crate::world::{SceneProjectError, World};

#[derive(Debug, Default)]
pub struct SceneAssetSerializer;

impl SceneAssetSerializer {
    pub fn load_world(
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<World, SceneProjectError> {
        World::load_scene_from_uri(project, uri)
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

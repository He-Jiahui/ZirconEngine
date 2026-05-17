use serde::{Deserialize, Serialize};

use crate::scene::World;

use super::{DynamicScene, DynamicSceneError, EntityRemap};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenePatch {
    pub scene: DynamicScene,
}

impl ScenePatch {
    pub fn from_scene(scene: DynamicScene) -> Self {
        Self { scene }
    }

    pub fn from_world(world: &World) -> Result<Self, DynamicSceneError> {
        DynamicScene::from_world(world).map(Self::from_scene)
    }

    pub fn apply(&self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        self.scene.spawn_into(world)
    }
}

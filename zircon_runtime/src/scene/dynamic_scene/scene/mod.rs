use serde::{Deserialize, Serialize};

use crate::scene::World;

use super::{
    DynamicEntity, DynamicResource, DynamicSceneError, EntityRemap, ScenePatchPreviewReport,
};

mod capture;
mod spawn;
mod validation;

pub const DYNAMIC_SCENE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicScene {
    pub format_version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_types: Vec<crate::plugin::ComponentTypeDescriptor>,
    #[serde(default)]
    pub entities: Vec<DynamicEntity>,
    #[serde(default)]
    pub resources: Vec<DynamicResource>,
}

impl DynamicScene {
    pub fn empty() -> Self {
        Self {
            format_version: DYNAMIC_SCENE_FORMAT_VERSION,
            component_types: Vec::new(),
            entities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_world(world: &World) -> Result<Self, DynamicSceneError> {
        capture::dynamic_scene_from_world(world)
    }

    pub fn spawn_into(&self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        spawn::spawn_scene_into(self, world)
    }

    pub fn preview_spawn_into(
        &self,
        world: &World,
    ) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
        spawn::preview_scene_spawn_into(self, world)
    }

    pub fn ensure_supported(&self) -> Result<(), DynamicSceneError> {
        validation::ensure_scene_supported(self)
    }
}

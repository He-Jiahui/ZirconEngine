use serde::{Deserialize, Serialize};
use zircon_runtime_interface::serialization::PayloadHeader;

use crate::scene::World;

use super::{
    DynamicEntity, DynamicResource, DynamicSceneError, EntityRemap, ScenePatchPreviewReport,
};

mod capture;
mod spawn;
mod validation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicScene {
    #[serde(
        skip,
        default = "crate::scene::dynamic_scene::document::current_dynamic_scene_header"
    )]
    pub(super) payload_header: PayloadHeader,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_types: Vec<crate::core::framework::scene::ComponentTypeDescriptor>,
    #[serde(default)]
    pub entities: Vec<DynamicEntity>,
    #[serde(default)]
    pub resources: Vec<DynamicResource>,
}

impl DynamicScene {
    pub fn empty() -> Self {
        Self {
            payload_header: crate::scene::dynamic_scene::document::current_dynamic_scene_header(),
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

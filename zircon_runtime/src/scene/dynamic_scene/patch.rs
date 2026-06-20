use serde::{Deserialize, Serialize};

use crate::scene::World;

use crate::scene::EntityId;

use super::{DynamicScene, DynamicSceneError, EntityRemap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewEntityRemap {
    pub source_entity: EntityId,
    pub target_entity: EntityId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewComponentType {
    pub type_id: String,
    pub plugin_id: String,
    pub display_name: String,
    pub already_registered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewResource {
    pub type_path: String,
    pub already_present: bool,
    pub can_create_on_apply: bool,
    pub field_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewReport {
    pub component_type_count: usize,
    pub existing_component_type_count: usize,
    pub new_component_type_count: usize,
    pub component_instance_count: usize,
    pub entity_count: usize,
    pub resource_count: usize,
    pub target_entity_count: usize,
    pub preserved_entity_count: usize,
    pub remapped_entity_count: usize,
    pub component_types: Vec<ScenePatchPreviewComponentType>,
    pub resources: Vec<ScenePatchPreviewResource>,
    pub entity_remaps: Vec<ScenePatchPreviewEntityRemap>,
}

impl ScenePatchPreviewReport {
    pub fn has_entity_remaps(&self) -> bool {
        self.remapped_entity_count > 0
    }

    pub fn has_new_component_types(&self) -> bool {
        self.new_component_type_count > 0
    }

    pub fn new_component_types(&self) -> impl Iterator<Item = &ScenePatchPreviewComponentType> {
        self.component_types
            .iter()
            .filter(|component_type| !component_type.already_registered)
    }

    pub fn resources_requiring_creation(&self) -> impl Iterator<Item = &ScenePatchPreviewResource> {
        self.resources
            .iter()
            .filter(|resource| !resource.already_present && resource.can_create_on_apply)
    }
}

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

    pub fn preview_apply(
        &self,
        world: &World,
    ) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
        self.scene.preview_spawn_into(world)
    }

    pub fn apply(&self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        self.scene.spawn_into(world)
    }
}

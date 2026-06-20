use crate::scene::{EntityRemap, World};

use super::super::{DynamicScene, DynamicSceneError};

/// A validated dynamic scene payload that is ready to apply on the main world.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedDynamicSceneSpawn {
    scene: DynamicScene,
    component_type_count: usize,
    entity_count: usize,
    resource_count: usize,
}

impl PreparedDynamicSceneSpawn {
    pub fn new(scene: DynamicScene) -> Result<Self, DynamicSceneError> {
        scene.ensure_supported()?;
        Ok(Self {
            component_type_count: scene.component_types.len(),
            entity_count: scene.entities.len(),
            resource_count: scene.resources.len(),
            scene,
        })
    }

    pub fn scene(&self) -> &DynamicScene {
        &self.scene
    }

    pub fn into_scene(self) -> DynamicScene {
        self.scene
    }

    pub fn component_type_count(&self) -> usize {
        self.component_type_count
    }

    pub fn entity_count(&self) -> usize {
        self.entity_count
    }

    pub fn resource_count(&self) -> usize {
        self.resource_count
    }

    pub fn spawn_into(self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        self.scene.spawn_into(world)
    }
}

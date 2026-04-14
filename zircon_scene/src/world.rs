//! ECS world state, project I/O, and render extraction.

mod bootstrap;
mod derived_state;
mod hierarchy;
mod project_io;
mod query;
mod records;
mod render;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::{
    Active, CameraComponent, DirectionalLight, Hierarchy, LocalTransform, MeshRenderer, Name,
    NodeKind, SceneNode, Schedule, WorldTransform,
};
use crate::EntityId;

pub use project_io::SceneProjectError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World {
    entities: Vec<EntityId>,
    #[serde(default)]
    kinds: HashMap<EntityId, NodeKind>,
    names: HashMap<EntityId, Name>,
    hierarchy: HashMap<EntityId, Hierarchy>,
    local_transforms: HashMap<EntityId, LocalTransform>,
    world_transforms: HashMap<EntityId, WorldTransform>,
    cameras: HashMap<EntityId, CameraComponent>,
    mesh_renderers: HashMap<EntityId, MeshRenderer>,
    directional_lights: HashMap<EntityId, DirectionalLight>,
    active: HashMap<EntityId, Active>,
    next_id: EntityId,
    active_camera: EntityId,
    selected_entity: Option<EntityId>,
    #[serde(skip, default)]
    schedule: Schedule,
    #[serde(skip, default)]
    node_cache: Vec<SceneNode>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

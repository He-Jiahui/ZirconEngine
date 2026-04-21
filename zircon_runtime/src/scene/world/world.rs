use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, MeshRenderer, Mobility, Name, NodeKind, PointLight,
    RenderLayerMask, RigidBodyComponent, SceneNode, Schedule, SpotLight, WorldMatrix,
};
use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub(super) entities: Vec<EntityId>,
    #[serde(default)]
    pub(super) kinds: HashMap<EntityId, NodeKind>,
    pub(super) names: HashMap<EntityId, Name>,
    pub(super) hierarchy: HashMap<EntityId, Hierarchy>,
    pub(super) local_transforms: HashMap<EntityId, LocalTransform>,
    #[serde(skip, default)]
    pub(super) world_matrices: HashMap<EntityId, WorldMatrix>,
    pub(super) cameras: HashMap<EntityId, CameraComponent>,
    pub(super) mesh_renderers: HashMap<EntityId, MeshRenderer>,
    pub(super) directional_lights: HashMap<EntityId, DirectionalLight>,
    #[serde(default)]
    pub(super) point_lights: HashMap<EntityId, PointLight>,
    #[serde(default)]
    pub(super) spot_lights: HashMap<EntityId, SpotLight>,
    #[serde(default)]
    pub(super) rigid_bodies: HashMap<EntityId, RigidBodyComponent>,
    #[serde(default)]
    pub(super) colliders: HashMap<EntityId, ColliderComponent>,
    #[serde(default)]
    pub(super) joints: HashMap<EntityId, JointComponent>,
    #[serde(default)]
    pub(super) animation_skeletons: HashMap<EntityId, AnimationSkeletonComponent>,
    #[serde(default)]
    pub(super) animation_players: HashMap<EntityId, AnimationPlayerComponent>,
    #[serde(default)]
    pub(super) animation_sequence_players: HashMap<EntityId, AnimationSequencePlayerComponent>,
    #[serde(default)]
    pub(super) animation_graph_players: HashMap<EntityId, AnimationGraphPlayerComponent>,
    #[serde(default)]
    pub(super) animation_state_machine_players:
        HashMap<EntityId, AnimationStateMachinePlayerComponent>,
    #[serde(default, rename = "active")]
    pub(super) active_self: HashMap<EntityId, ActiveSelf>,
    #[serde(skip, default)]
    pub(super) active_in_hierarchy: HashMap<EntityId, ActiveInHierarchy>,
    #[serde(default)]
    pub(super) render_layer_masks: HashMap<EntityId, RenderLayerMask>,
    #[serde(default)]
    pub(super) mobility: HashMap<EntityId, Mobility>,
    pub(super) next_id: EntityId,
    pub(super) active_camera: EntityId,
    #[serde(skip, default)]
    pub(super) schedule: Schedule,
    #[serde(skip, default)]
    pub(super) node_cache: Vec<SceneNode>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

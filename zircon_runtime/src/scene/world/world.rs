use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, MeshRenderer, Mobility, Name, NodeKind, PointLight,
    RenderLayerMask, RigidBodyComponent, SceneNode, SpotLight, WorldMatrix,
};
use crate::scene::ecs::{
    ChangeTick, CommandQueue, ComponentRegistry, ComponentStorage, EntityRegistry, EventStore,
    RemovedComponentEvents, ResourceRegistry, ResourceStore, Schedule,
};
use crate::scene::reflect::TypeRegistry;
use crate::scene::EntityId;

use super::{dirty_state::DerivedStateDirty, ComponentTypeRegistry};

#[derive(Clone, Debug, PartialEq, Serialize)]
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
    #[serde(default)]
    pub(super) dynamic_components: HashMap<EntityId, HashMap<String, serde_json::Value>>,
    #[serde(skip, default)]
    pub(super) component_types: ComponentTypeRegistry,
    #[serde(skip, default)]
    pub(super) type_registry: TypeRegistry,
    pub(super) next_id: EntityId,
    pub(super) active_camera: EntityId,
    #[serde(skip, default)]
    pub(super) schedule: Schedule,
    #[serde(skip, default)]
    pub(super) entity_registry: EntityRegistry,
    #[serde(skip, default)]
    pub(super) component_registry: ComponentRegistry,
    #[serde(skip, default)]
    pub(super) component_storage: ComponentStorage,
    #[serde(skip, default)]
    pub(super) removed_component_events: RemovedComponentEvents,
    #[serde(skip, default)]
    pub(super) resource_registry: ResourceRegistry,
    #[serde(skip, default)]
    pub(super) resources: ResourceStore,
    #[serde(skip, default)]
    pub(super) events: EventStore,
    #[serde(skip, default)]
    pub(super) command_queue: CommandQueue,
    #[serde(skip, default = "default_change_tick")]
    pub(super) change_tick: ChangeTick,
    #[serde(skip, default)]
    pub(super) active_change_tick: Option<ChangeTick>,
    #[serde(skip, default)]
    pub(super) node_cache: Vec<SceneNode>,
    #[serde(skip, default)]
    pub(super) derived_state_dirty: DerivedStateDirty,
}

#[derive(Deserialize)]
struct WorldPersistentState {
    entities: Vec<EntityId>,
    #[serde(default)]
    kinds: HashMap<EntityId, NodeKind>,
    names: HashMap<EntityId, Name>,
    hierarchy: HashMap<EntityId, Hierarchy>,
    local_transforms: HashMap<EntityId, LocalTransform>,
    cameras: HashMap<EntityId, CameraComponent>,
    mesh_renderers: HashMap<EntityId, MeshRenderer>,
    directional_lights: HashMap<EntityId, DirectionalLight>,
    #[serde(default)]
    point_lights: HashMap<EntityId, PointLight>,
    #[serde(default)]
    spot_lights: HashMap<EntityId, SpotLight>,
    #[serde(default)]
    rigid_bodies: HashMap<EntityId, RigidBodyComponent>,
    #[serde(default)]
    colliders: HashMap<EntityId, ColliderComponent>,
    #[serde(default)]
    joints: HashMap<EntityId, JointComponent>,
    #[serde(default)]
    animation_skeletons: HashMap<EntityId, AnimationSkeletonComponent>,
    #[serde(default)]
    animation_players: HashMap<EntityId, AnimationPlayerComponent>,
    #[serde(default)]
    animation_sequence_players: HashMap<EntityId, AnimationSequencePlayerComponent>,
    #[serde(default)]
    animation_graph_players: HashMap<EntityId, AnimationGraphPlayerComponent>,
    #[serde(default)]
    animation_state_machine_players: HashMap<EntityId, AnimationStateMachinePlayerComponent>,
    #[serde(default, rename = "active")]
    active_self: HashMap<EntityId, ActiveSelf>,
    #[serde(default)]
    render_layer_masks: HashMap<EntityId, RenderLayerMask>,
    #[serde(default)]
    mobility: HashMap<EntityId, Mobility>,
    #[serde(default)]
    dynamic_components: HashMap<EntityId, HashMap<String, serde_json::Value>>,
    next_id: EntityId,
    active_camera: EntityId,
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state = WorldPersistentState::deserialize(deserializer)?;
        let mut world = Self {
            entities: state.entities,
            kinds: state.kinds,
            names: state.names,
            hierarchy: state.hierarchy,
            local_transforms: state.local_transforms,
            world_matrices: HashMap::new(),
            cameras: state.cameras,
            mesh_renderers: state.mesh_renderers,
            directional_lights: state.directional_lights,
            point_lights: state.point_lights,
            spot_lights: state.spot_lights,
            rigid_bodies: state.rigid_bodies,
            colliders: state.colliders,
            joints: state.joints,
            animation_skeletons: state.animation_skeletons,
            animation_players: state.animation_players,
            animation_sequence_players: state.animation_sequence_players,
            animation_graph_players: state.animation_graph_players,
            animation_state_machine_players: state.animation_state_machine_players,
            active_self: state.active_self,
            active_in_hierarchy: HashMap::new(),
            render_layer_masks: state.render_layer_masks,
            mobility: state.mobility,
            dynamic_components: state.dynamic_components,
            component_types: Default::default(),
            type_registry: Default::default(),
            next_id: state.next_id,
            active_camera: state.active_camera,
            schedule: Default::default(),
            entity_registry: Default::default(),
            component_registry: Default::default(),
            component_storage: Default::default(),
            removed_component_events: Default::default(),
            resource_registry: Default::default(),
            resources: Default::default(),
            events: Default::default(),
            command_queue: Default::default(),
            change_tick: default_change_tick(),
            active_change_tick: None,
            node_cache: Vec::new(),
            derived_state_dirty: Default::default(),
        };
        crate::scene::reflect::register_builtin_reflection(&mut world);
        world.rebuild_entity_registry();
        world.rebuild_typed_component_presence();
        Ok(world)
    }
}

fn default_change_tick() -> ChangeTick {
    ChangeTick::INITIAL
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub(in crate::scene) fn type_registry_for_reflection(&self) -> &TypeRegistry {
        &self.type_registry
    }

    pub(in crate::scene) fn type_registry_mut_for_reflection(&mut self) -> &mut TypeRegistry {
        &mut self.type_registry
    }
}

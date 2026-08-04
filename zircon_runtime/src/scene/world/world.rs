use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeSet, HashMap};

use super::{
    compiled_binding::SceneBindingGenerations, derived_state::NODE_KIND_ORDINAL_COUNT,
    dirty_state::DerivedStateDirty, generation::WorldGeneration, ComponentTypeRegistry,
};
use crate::scene::components::{
    ActiveSelf, AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name,
    NodeKind, PointLight, PostProcessSettingsComponent, PostProcessVolumeComponent, RectLight,
    RenderLayerMask, RigidBodyComponent, SceneNode, SpotLight, Sprite2dComponent,
};
use crate::scene::ecs::{
    ArchetypeIndex, ChangeTick, CommandQueue, ComponentLifecycleEvent, ComponentRegistry,
    ComponentStorage, DeferredCommandError, EcsFramePerformanceDiagnostics, EntityRegistry,
    EventStore, MessageStore, ObserverStore, RemovedComponentEvents, ResourceRegistry,
    ResourceStore, Schedule,
};
use crate::scene::event_mirror::RuntimeEventMirrorRegistry;
use crate::scene::inspection::WorldInspectionArtifactCache;
use crate::scene::reflect::TypeRegistry;
use crate::scene::EntityId;
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct QueryCacheRevision(u64);

impl QueryCacheRevision {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    pub(super) fn advance(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

impl PartialEq for QueryCacheRevision {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct World {
    pub(super) entities: Vec<EntityId>,
    #[serde(default)]
    pub(super) kinds: HashMap<EntityId, NodeKind>,
    #[serde(skip, default)]
    pub(super) node_kind_ordinals: [usize; NODE_KIND_ORDINAL_COUNT],
    pub(super) names: HashMap<EntityId, Name>,
    pub(super) hierarchy: HashMap<EntityId, Hierarchy>,
    pub(super) local_transforms: HashMap<EntityId, LocalTransform>,
    pub(super) cameras: HashMap<EntityId, CameraComponent>,
    pub(super) mesh_renderers: HashMap<EntityId, MeshRenderer>,
    #[serde(default)]
    pub(super) sprite_2d: HashMap<EntityId, Sprite2dComponent>,
    #[serde(default)]
    pub(super) mesh_2d: HashMap<EntityId, Mesh2dComponent>,
    #[serde(default)]
    pub(super) ambient_lights: HashMap<EntityId, AmbientLight>,
    pub(super) directional_lights: HashMap<EntityId, DirectionalLight>,
    #[serde(default)]
    pub(super) point_lights: HashMap<EntityId, PointLight>,
    #[serde(default)]
    pub(super) rect_lights: HashMap<EntityId, RectLight>,
    #[serde(default)]
    pub(super) spot_lights: HashMap<EntityId, SpotLight>,
    #[serde(skip, default)]
    pub(super) post_process_settings: HashMap<EntityId, PostProcessSettingsComponent>,
    #[serde(skip, default)]
    pub(super) post_process_volumes: HashMap<EntityId, PostProcessVolumeComponent>,
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
    #[serde(default)]
    pub(super) render_layer_masks: HashMap<EntityId, RenderLayerMask>,
    #[serde(default)]
    pub(super) mobility: HashMap<EntityId, Mobility>,
    #[serde(default)]
    pub(super) dynamic_components: HashMap<EntityId, HashMap<String, serde_json::Value>>,
    #[serde(skip, default)]
    pub(super) dynamic_component_generations: HashMap<String, u64>,
    #[serde(skip, default)]
    pub(super) component_types: ComponentTypeRegistry,
    #[serde(skip, default)]
    pub(super) type_registry: TypeRegistry,
    #[serde(skip, default)]
    pub(super) vm_catalog_type_paths: BTreeSet<String>,
    #[serde(skip, default)]
    pub(super) vm_dynamic_type_paths: BTreeSet<String>,
    pub(super) next_id: EntityId,
    pub(super) active_camera: EntityId,
    #[serde(skip, default)]
    pub(super) schedule: Schedule,
    #[serde(skip, default)]
    pub(super) archetype_index: ArchetypeIndex,
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
    pub(super) event_mirrors: RuntimeEventMirrorRegistry,
    #[serde(skip, default)]
    pub(super) messages: MessageStore,
    #[serde(skip, default)]
    pub(super) observers: ObserverStore,
    #[serde(skip, default)]
    pub(super) staged_lifecycle_events: Vec<ComponentLifecycleEvent>,
    #[serde(skip, default)]
    pub(super) record_staged_lifecycle_events: bool,
    #[serde(skip, default)]
    pub(super) command_queue: CommandQueue,
    #[serde(skip, default)]
    pub(super) deferred_command_errors: Vec<DeferredCommandError>,
    #[serde(skip, default)]
    pub(super) ecs_frame_performance_diagnostics: EcsFramePerformanceDiagnostics,
    #[serde(skip, default)]
    pub(super) query_cache_revision: QueryCacheRevision,
    #[serde(skip, default)]
    pub(super) world_generation: WorldGeneration,
    #[serde(skip, default)]
    pub(super) scene_binding_generations: SceneBindingGenerations,
    #[serde(skip, default = "default_change_tick")]
    pub(super) change_tick: ChangeTick,
    #[serde(skip, default)]
    pub(super) last_change_tick: ChangeTick,
    #[serde(skip, default)]
    pub(super) active_change_tick: Option<ChangeTick>,
    #[serde(skip, default)]
    pub(super) node_cache: Vec<SceneNode>,
    #[serde(skip, default)]
    pub(in crate::scene) inspection_artifact_cache: WorldInspectionArtifactCache,
    #[serde(skip, default)]
    pub(super) derived_state_dirty: DerivedStateDirty,
}

impl Clone for World {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            entities: self.entities.clone(),
            kinds: self.kinds.clone(),
            node_kind_ordinals: self.node_kind_ordinals,
            names: self.names.clone(),
            hierarchy: self.hierarchy.clone(),
            local_transforms: self.local_transforms.clone(),
            cameras: self.cameras.clone(),
            mesh_renderers: self.mesh_renderers.clone(),
            sprite_2d: self.sprite_2d.clone(),
            mesh_2d: self.mesh_2d.clone(),
            ambient_lights: self.ambient_lights.clone(),
            directional_lights: self.directional_lights.clone(),
            point_lights: self.point_lights.clone(),
            rect_lights: self.rect_lights.clone(),
            spot_lights: self.spot_lights.clone(),
            post_process_settings: self.post_process_settings.clone(),
            post_process_volumes: self.post_process_volumes.clone(),
            rigid_bodies: self.rigid_bodies.clone(),
            colliders: self.colliders.clone(),
            joints: self.joints.clone(),
            animation_skeletons: self.animation_skeletons.clone(),
            animation_players: self.animation_players.clone(),
            animation_sequence_players: self.animation_sequence_players.clone(),
            animation_graph_players: self.animation_graph_players.clone(),
            animation_state_machine_players: self.animation_state_machine_players.clone(),
            active_self: self.active_self.clone(),
            render_layer_masks: self.render_layer_masks.clone(),
            mobility: self.mobility.clone(),
            dynamic_components: self.dynamic_components.clone(),
            dynamic_component_generations: self.dynamic_component_generations.clone(),
            component_types: self.component_types.clone(),
            type_registry: self.type_registry.clone(),
            vm_catalog_type_paths: self.vm_catalog_type_paths.clone(),
            vm_dynamic_type_paths: self.vm_dynamic_type_paths.clone(),
            next_id: self.next_id,
            active_camera: self.active_camera,
            schedule: self.schedule.clone(),
            archetype_index: Default::default(),
            entity_registry: Default::default(),
            component_registry: self.component_registry.clone(),
            component_storage: Default::default(),
            removed_component_events: self.removed_component_events.clone(),
            resource_registry: self.resource_registry.clone(),
            resources: self.resources.clone(),
            events: self.events.clone(),
            event_mirrors: self.event_mirrors.clone(),
            messages: self.messages.clone(),
            observers: self.observers.clone(),
            staged_lifecycle_events: Vec::new(),
            record_staged_lifecycle_events: true,
            command_queue: self.command_queue.clone(),
            deferred_command_errors: self.deferred_command_errors.clone(),
            ecs_frame_performance_diagnostics: self.ecs_frame_performance_diagnostics.clone(),
            query_cache_revision: self.query_cache_revision,
            world_generation: self.world_generation.clone(),
            scene_binding_generations: self.scene_binding_generations.clone(),
            change_tick: self.change_tick,
            last_change_tick: self.last_change_tick,
            active_change_tick: self.active_change_tick,
            node_cache: self.node_cache.clone(),
            inspection_artifact_cache: self.inspection_artifact_cache.clone(),
            derived_state_dirty: Default::default(),
        };
        cloned.rebuild_entity_registry();
        cloned.rebuild_component_storage_projection();
        cloned.record_staged_lifecycle_events = self.record_staged_lifecycle_events;
        cloned.staged_lifecycle_events = self.staged_lifecycle_events.clone();
        cloned
    }
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
    #[serde(default)]
    sprite_2d: HashMap<EntityId, Sprite2dComponent>,
    #[serde(default)]
    mesh_2d: HashMap<EntityId, Mesh2dComponent>,
    #[serde(default)]
    ambient_lights: HashMap<EntityId, AmbientLight>,
    directional_lights: HashMap<EntityId, DirectionalLight>,
    #[serde(default)]
    point_lights: HashMap<EntityId, PointLight>,
    #[serde(default)]
    rect_lights: HashMap<EntityId, RectLight>,
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
            node_kind_ordinals: Default::default(),
            names: state.names,
            hierarchy: state.hierarchy,
            local_transforms: state.local_transforms,
            cameras: state.cameras,
            mesh_renderers: state.mesh_renderers,
            sprite_2d: state.sprite_2d,
            mesh_2d: state.mesh_2d,
            ambient_lights: state.ambient_lights,
            directional_lights: state.directional_lights,
            point_lights: state.point_lights,
            rect_lights: state.rect_lights,
            spot_lights: state.spot_lights,
            post_process_settings: HashMap::new(),
            post_process_volumes: HashMap::new(),
            rigid_bodies: state.rigid_bodies,
            colliders: state.colliders,
            joints: state.joints,
            animation_skeletons: state.animation_skeletons,
            animation_players: state.animation_players,
            animation_sequence_players: state.animation_sequence_players,
            animation_graph_players: state.animation_graph_players,
            animation_state_machine_players: state.animation_state_machine_players,
            active_self: state.active_self,
            render_layer_masks: state.render_layer_masks,
            mobility: state.mobility,
            dynamic_components: state.dynamic_components,
            dynamic_component_generations: HashMap::new(),
            component_types: Default::default(),
            type_registry: Default::default(),
            vm_catalog_type_paths: Default::default(),
            vm_dynamic_type_paths: Default::default(),
            next_id: state.next_id,
            active_camera: state.active_camera,
            schedule: Default::default(),
            archetype_index: Default::default(),
            entity_registry: Default::default(),
            component_registry: Default::default(),
            component_storage: Default::default(),
            removed_component_events: Default::default(),
            resource_registry: Default::default(),
            resources: Default::default(),
            events: Default::default(),
            event_mirrors: Default::default(),
            messages: Default::default(),
            observers: Default::default(),
            staged_lifecycle_events: Vec::new(),
            record_staged_lifecycle_events: false,
            command_queue: Default::default(),
            deferred_command_errors: Vec::new(),
            ecs_frame_performance_diagnostics: Default::default(),
            query_cache_revision: QueryCacheRevision::default(),
            world_generation: WorldGeneration::default(),
            scene_binding_generations: SceneBindingGenerations::default(),
            change_tick: default_change_tick(),
            last_change_tick: ChangeTick::ZERO,
            active_change_tick: None,
            node_cache: Vec::new(),
            inspection_artifact_cache: Default::default(),
            derived_state_dirty: Default::default(),
        };
        crate::scene::reflect::register_builtin_reflection(&mut world);
        world.rebuild_entity_registry();
        world.rebuild_typed_component_presence();
        world.rebuild_node_kind_ordinals();
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

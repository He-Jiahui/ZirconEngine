use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use super::{
    ComponentTypeRegistry,
    compiled_binding::{CompiledScenePropertyAccessDiagnostics, SceneBindingGenerations},
    derived_state::NODE_KIND_ORDINAL_COUNT,
    dirty_state::DerivedStateDirty,
    entity_id_allocator::EntityIdAllocator,
    generation::{LifecycleVisibilityRevision, WorldGeneration},
    hierarchy_topology::HierarchyTopology,
};
use crate::scene::components::{
    ActiveSelf, AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name,
    NodeKind, PointLight, RectLight, RenderLayerMask, RigidBodyComponent, SceneNode, SpotLight,
    Sprite2dComponent,
};
use crate::scene::ecs::{
    ArchetypeIndex, ChangeTick, CommandQueue, ComponentLifecycleEvent, ComponentRegistry,
    ComponentStorage, DeferredCommandError, DeferredCommandTarget, DeferredEntityRef,
    DeferredSpawnToken, EcsFramePerformanceDiagnostics, EntityRegistry, EventStore, MessageStore,
    ObserverStore, RemovedComponentEvents, ResourceRegistry, ResourceStore, Schedule,
};
use crate::scene::event_mirror::RuntimeEventMirrorRegistry;
use crate::scene::inspection::SubscriptionTable;
use crate::scene::inspection::WorldInspectionArtifactCache;
use crate::scene::reflect::TypeRegistry;
use crate::scene::{EntityId, SceneError};
use zircon_runtime_interface::world_sync::WorldFact;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ArchetypeAssignmentCounter(u64);

impl ArchetypeAssignmentCounter {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    pub(super) fn record_assignment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

impl PartialEq for ArchetypeAssignmentCounter {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Runtime-only sink owned by the live LevelSystem session.
///
/// The sink deliberately does not participate in persistent-world equality. A clone or decoded
/// world receives its default empty sink, so staging mutations cannot enter a live session's
/// invalidation queue.
#[derive(Debug, Default)]
pub(super) struct WorldSyncSubscriptionSink(Option<Arc<Mutex<SubscriptionTable>>>);

impl PartialEq for WorldSyncSubscriptionSink {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl WorldSyncSubscriptionSink {
    fn attach(&mut self, subscriptions: Arc<Mutex<SubscriptionTable>>) {
        self.0 = Some(subscriptions);
    }

    fn record(&self, world: &World, fact: WorldFact) {
        let Some(subscriptions) = self.0.as_ref().map(Arc::clone) else {
            return;
        };
        subscriptions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_fact(world, fact);
    }

    fn invalidate_component_type(&self, type_name: &str) {
        let Some(subscriptions) = self.0.as_ref().map(Arc::clone) else {
            return;
        };
        subscriptions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .invalidate_component_type(type_name);
    }
}

#[derive(Debug, PartialEq)]
pub struct World {
    pub(super) entities: Vec<EntityId>,
    pub(super) entity_dense_rows: HashMap<EntityId, usize>,
    pub(super) kinds: HashMap<EntityId, NodeKind>,
    pub(super) node_kind_ordinals: [usize; NODE_KIND_ORDINAL_COUNT],
    pub(super) dynamic_components: HashMap<EntityId, HashMap<String, serde_json::Value>>,
    pub(super) dynamic_component_generations: HashMap<String, u64>,
    pub(super) component_types: ComponentTypeRegistry,
    pub(super) type_registry: TypeRegistry,
    pub(super) vm_catalog_type_paths: BTreeSet<String>,
    pub(super) vm_dynamic_type_paths: BTreeSet<String>,
    pub(super) entity_id_allocator: EntityIdAllocator,
    pub(super) active_camera: EntityId,
    pub(super) schedule: Schedule,
    pub(super) archetype_index: ArchetypeIndex,
    pub(super) stable_query_order: super::query_order::StableQueryOrderIndex,
    pub(super) hierarchy_mutation_index: HierarchyTopology,
    pub(super) entity_registry: EntityRegistry,
    pub(super) component_registry: ComponentRegistry,
    pub(super) component_storage: ComponentStorage,
    pub(super) removed_component_events: RemovedComponentEvents,
    pub(super) resource_registry: ResourceRegistry,
    pub(super) resources: ResourceStore,
    pub(super) events: EventStore,
    pub(super) event_mirrors: RuntimeEventMirrorRegistry,
    pub(super) messages: MessageStore,
    pub(super) observers: ObserverStore,
    pub(super) world_sync_subscriptions: WorldSyncSubscriptionSink,
    pub(super) staged_lifecycle_events: Vec<ComponentLifecycleEvent>,
    pub(super) record_staged_lifecycle_events: bool,
    pub(super) command_queue: CommandQueue,
    pub(super) deferred_command_errors: Vec<DeferredCommandError>,
    pub(super) deferred_direct_spawn_ordinal: u32,
    pub(super) deferred_direct_system_ordinal: u32,
    pub(super) deferred_spawn_resolutions: BTreeMap<DeferredSpawnToken, EntityId>,
    pub(super) published_deferred_spawns: BTreeSet<DeferredSpawnToken>,
    pub(super) ecs_frame_performance_diagnostics: EcsFramePerformanceDiagnostics,
    pub(super) archetype_assignment_counter: ArchetypeAssignmentCounter,
    pub(super) lifecycle_visibility_revision: LifecycleVisibilityRevision,
    pub(super) world_generation: WorldGeneration,
    pub(super) scene_binding_generations: SceneBindingGenerations,
    pub(super) compiled_scene_property_access_diagnostics: CompiledScenePropertyAccessDiagnostics,
    pub(super) change_tick: ChangeTick,
    pub(super) last_change_tick: ChangeTick,
    pub(super) active_change_tick: Option<ChangeTick>,
    pub(super) node_cache: Vec<SceneNode>,
    pub(super) node_cache_rows: HashMap<EntityId, usize>,
    pub(super) node_cache_topology_generation: u64,
    pub(in crate::scene) inspection_artifact_cache: WorldInspectionArtifactCache,
    pub(super) derived_state_dirty: DerivedStateDirty,
}

impl Clone for World {
    fn clone(&self) -> Self {
        crate::profile_scope!("runtime", "scene", "world_clone");
        let persistent_entity_core = self.persistent_entity_core_component_snapshot();
        let persistent_scene_render = self.persistent_scene_render_component_snapshot();
        let runtime_only_post_process = self.runtime_only_post_process_component_snapshot();
        let persistent_physics = self.persistent_physics_component_snapshot();
        let persistent_lighting = self.persistent_lighting_component_snapshot();
        let persistent_render_2d = self.persistent_render_2d_component_snapshot();
        let persistent_animation_runtime = self.persistent_animation_runtime_component_snapshot();
        let world_generation = self
            .world_generation
            .advanced_by(self.derived_state_dirty.pending_component_mutation_count());
        let stable_entities = self.stable_entity_ids().collect::<Vec<_>>();
        let mut cloned = Self {
            entities: self.entities.clone(),
            entity_dense_rows: Default::default(),
            kinds: self.kinds.clone(),
            node_kind_ordinals: self.node_kind_ordinals,
            dynamic_components: self.dynamic_components.clone(),
            dynamic_component_generations: self.dynamic_component_generations.clone(),
            component_types: self.component_types.clone(),
            type_registry: self.type_registry.clone(),
            vm_catalog_type_paths: self.vm_catalog_type_paths.clone(),
            vm_dynamic_type_paths: self.vm_dynamic_type_paths.clone(),
            entity_id_allocator: self.entity_id_allocator,
            active_camera: self.active_camera,
            schedule: self.schedule.clone(),
            archetype_index: Default::default(),
            stable_query_order: Default::default(),
            hierarchy_mutation_index: Default::default(),
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
            world_sync_subscriptions: Default::default(),
            staged_lifecycle_events: Vec::new(),
            record_staged_lifecycle_events: true,
            command_queue: self.command_queue.clone(),
            deferred_command_errors: self.deferred_command_errors.clone(),
            deferred_direct_spawn_ordinal: self.deferred_direct_spawn_ordinal,
            deferred_direct_system_ordinal: self.deferred_direct_system_ordinal,
            deferred_spawn_resolutions: Default::default(),
            published_deferred_spawns: Default::default(),
            ecs_frame_performance_diagnostics: self.ecs_frame_performance_diagnostics.clone(),
            archetype_assignment_counter: Default::default(),
            lifecycle_visibility_revision: self.lifecycle_visibility_revision,
            world_generation,
            scene_binding_generations: self.scene_binding_generations.clone(),
            compiled_scene_property_access_diagnostics: Default::default(),
            change_tick: self.change_tick,
            last_change_tick: self.last_change_tick,
            active_change_tick: self.active_change_tick,
            node_cache: self.node_cache.clone(),
            node_cache_rows: self.node_cache_rows.clone(),
            node_cache_topology_generation: self.node_cache_topology_generation,
            inspection_artifact_cache: self
                .inspection_artifact_cache
                .clone_for_world_generation(world_generation.get()),
            derived_state_dirty: Default::default(),
        };
        {
            crate::profile_scope!("runtime", "scene", "world_projection_rebuild");
            cloned.rebuild_entity_registry_with_stable_order(stable_entities);
            cloned.rebuild_component_storage_projection_with_owned_components(
                persistent_entity_core,
                persistent_scene_render,
                runtime_only_post_process,
                persistent_physics,
                persistent_lighting,
                persistent_render_2d,
                persistent_animation_runtime,
            );
        }
        cloned.record_staged_lifecycle_events = self.record_staged_lifecycle_events;
        cloned.staged_lifecycle_events = self.staged_lifecycle_events.clone();
        cloned
    }
}

#[derive(Deserialize)]
pub(super) struct WorldPersistentState {
    entities: Vec<EntityId>,
    #[serde(default)]
    kinds: HashMap<EntityId, NodeKind>,
    names: HashMap<EntityId, Name>,
    hierarchy: HashMap<EntityId, Hierarchy>,
    pub(super) local_transforms: HashMap<EntityId, LocalTransform>,
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

#[derive(Debug)]
pub(super) enum WorldPersistentStateError {
    OrphanComponent {
        entity: EntityId,
        component: &'static str,
    },
    Scene(SceneError),
}

impl WorldPersistentState {
    pub(super) fn first_orphan_component(&self) -> Option<(EntityId, &'static str)> {
        let known_entities = self.entities.iter().copied().collect::<BTreeSet<_>>();

        macro_rules! check_map {
            ($field:ident, $component:literal) => {
                if let Some(entity) = self
                    .$field
                    .keys()
                    .find(|entity| !known_entities.contains(entity))
                {
                    return Some((*entity, $component));
                }
            };
        }

        check_map!(kinds, "kinds");
        check_map!(names, "names");
        check_map!(hierarchy, "hierarchy");
        check_map!(local_transforms, "local_transforms");
        check_map!(cameras, "cameras");
        check_map!(mesh_renderers, "mesh_renderers");
        check_map!(sprite_2d, "sprite_2d");
        check_map!(mesh_2d, "mesh_2d");
        check_map!(ambient_lights, "ambient_lights");
        check_map!(directional_lights, "directional_lights");
        check_map!(point_lights, "point_lights");
        check_map!(rect_lights, "rect_lights");
        check_map!(spot_lights, "spot_lights");
        check_map!(rigid_bodies, "rigid_bodies");
        check_map!(colliders, "colliders");
        check_map!(joints, "joints");
        check_map!(animation_skeletons, "animation_skeletons");
        check_map!(animation_players, "animation_players");
        check_map!(animation_sequence_players, "animation_sequence_players");
        check_map!(animation_graph_players, "animation_graph_players");
        check_map!(
            animation_state_machine_players,
            "animation_state_machine_players"
        );
        check_map!(active_self, "active_self");
        check_map!(render_layer_masks, "render_layer_masks");
        check_map!(mobility, "mobility");
        check_map!(dynamic_components, "dynamic_components");

        None
    }
}

#[derive(Serialize)]
struct WorldPersistentStateRef<'a> {
    entities: Vec<EntityId>,
    kinds: &'a HashMap<EntityId, NodeKind>,
    names: HashMap<EntityId, Name>,
    hierarchy: HashMap<EntityId, Hierarchy>,
    local_transforms: HashMap<EntityId, LocalTransform>,
    cameras: HashMap<EntityId, CameraComponent>,
    mesh_renderers: HashMap<EntityId, MeshRenderer>,
    sprite_2d: HashMap<EntityId, Sprite2dComponent>,
    mesh_2d: HashMap<EntityId, Mesh2dComponent>,
    ambient_lights: HashMap<EntityId, AmbientLight>,
    directional_lights: HashMap<EntityId, DirectionalLight>,
    point_lights: HashMap<EntityId, PointLight>,
    rect_lights: HashMap<EntityId, RectLight>,
    spot_lights: HashMap<EntityId, SpotLight>,
    rigid_bodies: HashMap<EntityId, RigidBodyComponent>,
    colliders: HashMap<EntityId, ColliderComponent>,
    joints: HashMap<EntityId, JointComponent>,
    animation_skeletons: HashMap<EntityId, AnimationSkeletonComponent>,
    animation_players: HashMap<EntityId, AnimationPlayerComponent>,
    animation_sequence_players: HashMap<EntityId, AnimationSequencePlayerComponent>,
    animation_graph_players: HashMap<EntityId, AnimationGraphPlayerComponent>,
    animation_state_machine_players: HashMap<EntityId, AnimationStateMachinePlayerComponent>,
    #[serde(rename = "active")]
    active_self: HashMap<EntityId, ActiveSelf>,
    render_layer_masks: HashMap<EntityId, RenderLayerMask>,
    mobility: HashMap<EntityId, Mobility>,
    dynamic_components: &'a HashMap<EntityId, HashMap<String, serde_json::Value>>,
    next_id: EntityId,
    active_camera: EntityId,
}

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let persistent_entity_core = self.persistent_entity_core_component_snapshot();
        let persistent_scene_render = self.persistent_scene_render_component_snapshot();
        let persistent_physics = self.persistent_physics_component_snapshot();
        let persistent_lighting = self.persistent_lighting_component_snapshot();
        let persistent_render_2d = self.persistent_render_2d_component_snapshot();
        let persistent_animation_runtime = self.persistent_animation_runtime_component_snapshot();
        WorldPersistentStateRef {
            entities: self.stable_entity_ids().collect(),
            kinds: &self.kinds,
            names: persistent_entity_core.names,
            hierarchy: persistent_entity_core.hierarchy,
            local_transforms: persistent_entity_core.local_transforms,
            cameras: persistent_scene_render.cameras,
            mesh_renderers: persistent_scene_render.mesh_renderers,
            sprite_2d: persistent_render_2d.sprite_2d,
            mesh_2d: persistent_render_2d.mesh_2d,
            ambient_lights: persistent_lighting.ambient_lights,
            directional_lights: persistent_lighting.directional_lights,
            point_lights: persistent_lighting.point_lights,
            rect_lights: persistent_lighting.rect_lights,
            spot_lights: persistent_lighting.spot_lights,
            rigid_bodies: persistent_physics.rigid_bodies,
            colliders: persistent_physics.colliders,
            joints: persistent_physics.joints,
            animation_skeletons: persistent_animation_runtime.skeletons,
            animation_players: persistent_animation_runtime.players,
            animation_sequence_players: persistent_animation_runtime.sequence_players,
            animation_graph_players: persistent_animation_runtime.graph_players,
            animation_state_machine_players: persistent_animation_runtime.state_machine_players,
            active_self: persistent_entity_core.active_self,
            render_layer_masks: persistent_scene_render.render_layer_masks,
            mobility: persistent_scene_render.mobility,
            dynamic_components: &self.dynamic_components,
            next_id: self.entity_id_allocator.next_id(),
            active_camera: self.active_camera,
        }
        .serialize(serializer)
    }
}

impl World {
    pub(super) fn from_persistent_state(
        state: WorldPersistentState,
    ) -> Result<Self, WorldPersistentStateError> {
        if let Some(orphan) = state.first_orphan_component() {
            return Err(WorldPersistentStateError::OrphanComponent {
                entity: orphan.0,
                component: orphan.1,
            });
        }
        let entity_id_allocator = EntityIdAllocator::from_persisted_next(state.next_id)
            .map_err(WorldPersistentStateError::Scene)?;
        let persistent_entity_core =
            Self::persistent_entity_core_component_snapshot_from_serialized_maps(
                state.names,
                state.hierarchy,
                state.local_transforms,
                state.active_self,
            );
        let persistent_scene_render =
            Self::persistent_scene_render_component_snapshot_from_serialized_maps(
                state.render_layer_masks,
                state.cameras,
                state.mesh_renderers,
                state.mobility,
            );
        let persistent_physics = Self::persistent_physics_component_snapshot_from_serialized_maps(
            state.rigid_bodies,
            state.colliders,
            state.joints,
        );
        let persistent_lighting = Self::persistent_lighting_component_snapshot_from_serialized_maps(
            state.ambient_lights,
            state.directional_lights,
            state.point_lights,
            state.rect_lights,
            state.spot_lights,
        );
        let persistent_render_2d =
            Self::persistent_render_2d_component_snapshot_from_serialized_maps(
                state.sprite_2d,
                state.mesh_2d,
            );
        let persistent_animation_runtime =
            Self::persistent_animation_runtime_component_snapshot_from_serialized_maps(
                state.animation_skeletons,
                state.animation_players,
                state.animation_sequence_players,
                state.animation_graph_players,
                state.animation_state_machine_players,
            );
        let mut world = Self {
            entities: state.entities,
            entity_dense_rows: Default::default(),
            kinds: state.kinds,
            node_kind_ordinals: Default::default(),
            dynamic_components: state.dynamic_components,
            dynamic_component_generations: HashMap::new(),
            component_types: Default::default(),
            type_registry: Default::default(),
            vm_catalog_type_paths: Default::default(),
            vm_dynamic_type_paths: Default::default(),
            entity_id_allocator,
            active_camera: state.active_camera,
            schedule: Default::default(),
            archetype_index: Default::default(),
            stable_query_order: Default::default(),
            hierarchy_mutation_index: Default::default(),
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
            world_sync_subscriptions: Default::default(),
            staged_lifecycle_events: Vec::new(),
            record_staged_lifecycle_events: false,
            command_queue: Default::default(),
            deferred_command_errors: Vec::new(),
            deferred_direct_spawn_ordinal: 0,
            deferred_direct_system_ordinal: 0,
            deferred_spawn_resolutions: Default::default(),
            published_deferred_spawns: Default::default(),
            ecs_frame_performance_diagnostics: Default::default(),
            archetype_assignment_counter: Default::default(),
            lifecycle_visibility_revision: LifecycleVisibilityRevision::default(),
            world_generation: WorldGeneration::default(),
            scene_binding_generations: SceneBindingGenerations::default(),
            compiled_scene_property_access_diagnostics: Default::default(),
            change_tick: default_change_tick(),
            last_change_tick: ChangeTick::ZERO,
            active_change_tick: None,
            node_cache: Vec::new(),
            node_cache_rows: HashMap::new(),
            node_cache_topology_generation: 0,
            inspection_artifact_cache: Default::default(),
            derived_state_dirty: Default::default(),
        };
        crate::scene::reflect::register_builtin_reflection(&mut world);
        world.rebuild_entity_registry();
        world.component_registry = Default::default();
        let runtime_only_post_process = world.runtime_only_post_process_component_snapshot();
        world.rebuild_component_storage_projection_with_owned_components(
            persistent_entity_core,
            persistent_scene_render,
            runtime_only_post_process,
            persistent_physics,
            persistent_lighting,
            persistent_render_2d,
            persistent_animation_runtime,
        );
        world.rebuild_node_kind_ordinals();
        Ok(world)
    }
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state = WorldPersistentState::deserialize(deserializer)?;
        Self::from_persistent_state(state).map_err(|error| match error {
            WorldPersistentStateError::OrphanComponent { entity, component } => {
                serde::de::Error::custom(format!(
                    "persisted {component} component belongs to missing entity {entity}"
                ))
            }
            WorldPersistentStateError::Scene(source) => serde::de::Error::custom(source),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{World, WorldPersistentState, WorldPersistentStateError};
    use crate::scene::SceneError;

    #[test]
    fn persistent_state_retains_invalid_entity_allocator_diagnostics() {
        let world = World::empty();
        let mut state: WorldPersistentState =
            serde_json::from_value(serde_json::to_value(world).expect("world serializes"))
                .expect("serialized world decodes as persistent state");
        state.next_id = u64::MAX;

        assert!(matches!(
            World::from_persistent_state(state),
            Err(WorldPersistentStateError::Scene(
                SceneError::EntityIdExhausted { entity: u64::MAX }
            ))
        ));
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
    /// Attaches the live session-owned invalidation table to this authoritative world only.
    ///
    /// Cloned and deserialized worlds intentionally reset this sink so staging and snapshots
    /// cannot publish facts into the live level session.
    pub(in crate::scene) fn attach_world_sync_subscriptions(
        &mut self,
        subscriptions: Arc<Mutex<SubscriptionTable>>,
    ) {
        self.world_sync_subscriptions.attach(subscriptions);
    }

    pub(in crate::scene) fn record_world_fact(&self, fact: WorldFact) {
        self.world_sync_subscriptions.record(self, fact);
    }

    pub(in crate::scene) fn invalidate_world_component_type(&self, type_name: &str) {
        self.world_sync_subscriptions
            .invalidate_component_type(type_name);
    }

    pub(in crate::scene) fn type_registry_for_reflection(&self) -> &TypeRegistry {
        &self.type_registry
    }

    pub(in crate::scene) fn type_registry_mut_for_reflection(&mut self) -> &mut TypeRegistry {
        &mut self.type_registry
    }
}

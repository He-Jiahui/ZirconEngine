use std::any::TypeId;
use std::cell::Cell;

use crate::scene::EntityId;
use crate::scene::components::{
    ActiveSelf, AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name,
    NodeRecord, PointLight, RectLight, RenderLayerMask, RigidBodyComponent, SpotLight,
    Sprite2dComponent,
};
use crate::scene::ecs::{BundleStaging, Component, ComponentId, InternalEntity};

use super::{SceneError, SceneResult, World};

const MAX_BUNDLE_COMPONENTS: usize = 8;
const MAX_BUNDLE_COMPONENT_TYPES: usize = MAX_BUNDLE_COMPONENTS * 2;

#[derive(Clone, Copy)]
struct PreflightedBundleComponent {
    component_id: ComponentId,
    type_id: TypeId,
}

trait PendingBundleComponentValue {
    fn publish(
        self: Box<Self>,
        world: &mut World,
        entity: EntityId,
        component_id: ComponentId,
    ) -> SceneResult<bool>;
}

struct PendingBundleValue<T>
where
    T: Component,
{
    component: T,
}

impl<T> PendingBundleComponentValue for PendingBundleValue<T>
where
    T: Component,
{
    fn publish(
        self: Box<Self>,
        world: &mut World,
        entity: EntityId,
        component_id: ComponentId,
    ) -> SceneResult<bool> {
        world.insert_preflighted_bundle_component(entity, self.component, component_id)
    }
}

enum BundleTarget {
    Existing(InternalEntity),
    Spawn(NodeRecord),
    Committed(InternalEntity),
}

/// Holds the immutable checks and staged component identities for one bundle
/// mutation. No live world container changes until every component has staged.
pub(crate) struct BundleInsertionTransaction<'world> {
    world: &'world mut World,
    entity: EntityId,
    target: Option<BundleTarget>,
    spawn_next_id: Option<EntityId>,
    spawned_entity: bool,
    components: [Option<PreflightedBundleComponent>; MAX_BUNDLE_COMPONENTS],
    pending_values: [Option<Box<dyn PendingBundleComponentValue>>; MAX_BUNDLE_COMPONENTS],
    component_count: usize,
    committed_component_count: usize,
    added_component: bool,
    unregistered_component_types: [Option<TypeId>; MAX_BUNDLE_COMPONENT_TYPES],
    unregistered_component_count: usize,
    staged_hierarchy_parent: Option<Option<EntityId>>,
    staged_mobility: Option<Mobility>,
    final_state_validated: Cell<bool>,
    prior_lifecycle_staging: Option<bool>,
    lifecycle_start: usize,
}

impl<'world> BundleInsertionTransaction<'world> {
    pub(super) fn new(
        world: &'world mut World,
        entity: EntityId,
        internal_entity: InternalEntity,
    ) -> Self {
        Self {
            world,
            entity,
            target: Some(BundleTarget::Existing(internal_entity)),
            spawn_next_id: None,
            spawned_entity: false,
            components: [None; MAX_BUNDLE_COMPONENTS],
            pending_values: std::array::from_fn(|_| None),
            component_count: 0,
            committed_component_count: 0,
            added_component: false,
            unregistered_component_types: [None; MAX_BUNDLE_COMPONENT_TYPES],
            unregistered_component_count: 0,
            staged_hierarchy_parent: None,
            staged_mobility: None,
            final_state_validated: Cell::new(false),
            prior_lifecycle_staging: None,
            lifecycle_start: 0,
        }
    }

    pub(super) fn new_spawn(world: &'world mut World, record: NodeRecord) -> SceneResult<Self> {
        let entity = record.id;
        let spawn_next_id = entity
            .checked_add(1)
            .ok_or(SceneError::EntityIdExhausted { entity })?;
        let mut transaction = Self {
            world,
            entity,
            target: None,
            spawn_next_id: Some(spawn_next_id),
            spawned_entity: false,
            components: [None; MAX_BUNDLE_COMPONENTS],
            pending_values: std::array::from_fn(|_| None),
            component_count: 0,
            committed_component_count: 0,
            added_component: false,
            unregistered_component_types: [None; MAX_BUNDLE_COMPONENT_TYPES],
            unregistered_component_count: 0,
            staged_hierarchy_parent: None,
            staged_mobility: None,
            final_state_validated: Cell::new(false),
            prior_lifecycle_staging: None,
            lifecycle_start: 0,
        };
        transaction.reserve_node_record_component_types(&record)?;
        transaction.target = Some(BundleTarget::Spawn(record));
        Ok(transaction)
    }

    fn stage<T>(&mut self, component: &T) -> SceneResult<()>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        if self.components[..self.component_count]
            .iter()
            .flatten()
            .any(|preflight| preflight.type_id == type_id)
        {
            return Err(SceneError::Message(
                "bundle cannot contain duplicate component types".to_string(),
            ));
        }
        let staged_mobility = (component as &dyn std::any::Any)
            .downcast_ref::<Mobility>()
            .copied();
        if staged_mobility.is_none() {
            self.world
                .validate_fixed_component(self.entity, component)?;
        }
        let component_id = self.staged_component_id::<T>()?;
        self.world
            .component_storage
            .validate_insert::<T>(component_id, T::STORAGE_TYPE)?;
        let Some(component_slot) = self.components.get_mut(self.component_count) else {
            return Err(SceneError::Message(
                "bundle preflight exceeds the supported component width".to_string(),
            ));
        };
        *component_slot = Some(PreflightedBundleComponent {
            component_id,
            type_id,
        });
        self.component_count += 1;
        self.final_state_validated.set(false);
        if let Some(hierarchy) = (component as &dyn std::any::Any).downcast_ref::<Hierarchy>() {
            self.staged_hierarchy_parent = Some(hierarchy.parent);
        }
        if let Some(mobility) = staged_mobility {
            self.staged_mobility = Some(mobility);
        }
        Ok(())
    }

    fn validate_final_state(&self) -> SceneResult<()> {
        let parent = self.staged_hierarchy_parent.unwrap_or_else(|| {
            self.world
                .hierarchy
                .get(&self.entity)
                .and_then(|value| value.parent)
        });
        let mobility = self
            .staged_mobility
            .unwrap_or_else(|| self.world.mobility(self.entity).unwrap_or_default());
        self.world
            .validate_bundle_mobility_state(self.entity, parent, mobility)?;
        self.final_state_validated.set(true);
        Ok(())
    }

    fn commit<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        if !self.final_state_validated.get() {
            return Err(SceneError::BundleFinalStateNotValidated);
        }
        let Some(preflight) = self
            .components
            .get(self.committed_component_count)
            .copied()
            .flatten()
        else {
            return Err(SceneError::Message(
                "bundle commit exceeded its preflighted component count".to_string(),
            ));
        };
        if preflight.type_id != TypeId::of::<T>() {
            return Err(SceneError::Message(
                "bundle commit did not preserve staged component order".to_string(),
            ));
        }
        let Some(value_slot) = self.pending_values.get_mut(self.committed_component_count) else {
            return Err(SceneError::BundleCommitIncomplete {
                staged: self.component_count,
                committed: self.committed_component_count,
            });
        };
        *value_slot = Some(Box::new(PendingBundleValue { component }));
        self.committed_component_count += 1;
        Ok(())
    }

    pub(crate) fn finish(mut self) -> SceneResult<()> {
        let spawns_entity = matches!(self.target.as_ref(), Some(BundleTarget::Spawn(_)));
        if (self.component_count != 0 || spawns_entity) && !self.final_state_validated.get() {
            return Err(SceneError::BundleFinalStateNotValidated);
        }
        if self.committed_component_count != self.component_count {
            return Err(SceneError::BundleCommitIncomplete {
                staged: self.component_count,
                committed: self.committed_component_count,
            });
        }
        if self.pending_values[..self.component_count]
            .iter()
            .any(Option::is_none)
        {
            return Err(SceneError::BundleCommitIncomplete {
                staged: self.component_count,
                committed: self.committed_component_count,
            });
        }
        if self.component_count == 0 && !spawns_entity {
            return Ok(());
        }
        self.begin_commit()?;
        let Some(BundleTarget::Committed(committed_internal)) = self.target.as_ref() else {
            return Err(SceneError::Message(
                "bundle commit did not establish a live entity target".to_string(),
            ));
        };
        if self.world.internal_entity(self.entity) != Some(*committed_internal) {
            return Err(SceneError::missing_entity("commit bundle for", self.entity));
        }
        for component_index in 0..self.component_count {
            let component = self.pending_values[component_index].take().ok_or(
                SceneError::BundleCommitIncomplete {
                    staged: self.component_count,
                    committed: self.committed_component_count,
                },
            )?;
            let preflight =
                self.components[component_index].ok_or(SceneError::BundleCommitIncomplete {
                    staged: self.component_count,
                    committed: self.committed_component_count,
                })?;
            let added_component =
                component.publish(&mut *self.world, self.entity, preflight.component_id)?;
            self.added_component |= added_component;
        }
        let Some(prior_lifecycle_staging) = self.prior_lifecycle_staging.take() else {
            return Err(SceneError::Message(
                "nonempty bundle did not begin its commit".to_string(),
            ));
        };
        if self.added_component || self.spawned_entity {
            self.world.refresh_entity_archetype(self.entity);
            self.world.bump_query_cache_revision();
        }
        if self.spawned_entity {
            self.world.mark_derived_state_dirty();
            self.world
                .inspection_artifact_cache
                .mark_hierarchy_rows_dirty();
            self.world
                .advance_scene_binding_generations_for_new_descendant(self.entity);
        }
        self.world.advance_world_generation();
        self.world.record_staged_lifecycle_events = prior_lifecycle_staging;
        if prior_lifecycle_staging {
            return Ok(());
        }
        let lifecycle_events = self
            .world
            .staged_lifecycle_events
            .split_off(self.lifecycle_start);
        for event in lifecycle_events {
            self.world.dispatch_component_lifecycle(event);
        }
        Ok(())
    }

    fn begin_commit(&mut self) -> SceneResult<()> {
        if self.prior_lifecycle_staging.is_some() {
            return Ok(());
        }
        let target = self.target.take().ok_or_else(|| {
            SceneError::Message("bundle transaction has no pending target".to_string())
        })?;
        let spawn_next_id = match &target {
            BundleTarget::Spawn(_) => Some(self.spawn_next_id.ok_or_else(|| {
                SceneError::Message("bundle spawn did not retain its next entity id".to_string())
            })?),
            BundleTarget::Existing(internal) => {
                if self.world.internal_entity(self.entity) != Some(*internal) {
                    return Err(SceneError::missing_entity("commit bundle for", self.entity));
                }
                None
            }
            BundleTarget::Committed(_) => {
                return Err(SceneError::Message(
                    "bundle transaction was committed more than once".to_string(),
                ));
            }
        };
        self.lifecycle_start = self.world.staged_lifecycle_events.len();
        self.prior_lifecycle_staging = Some(std::mem::replace(
            &mut self.world.record_staged_lifecycle_events,
            true,
        ));
        let internal = match target {
            BundleTarget::Existing(internal) => internal,
            BundleTarget::Spawn(record) => {
                self.world
                    .insert_prevalidated_node_record_without_archetype(record);
                let next_id = spawn_next_id.expect("bundle spawn must retain its next entity id");
                self.world.next_id = self.world.next_id.max(next_id);
                self.world
                    .rebuild_fixed_component_presence_without_final_archetype(self.entity);
                self.spawned_entity = true;
                self.world
                    .internal_entity(self.entity)
                    .expect("prevalidated bundle spawn must register its entity")
            }
            BundleTarget::Committed(_) => {
                unreachable!("committed target was rejected before staging")
            }
        };
        self.target = Some(BundleTarget::Committed(internal));
        Ok(())
    }

    // This order must match `rebuild_fixed_component_presence_without_final_archetype`.
    fn reserve_node_record_component_types(&mut self, record: &NodeRecord) -> SceneResult<()> {
        self.reserve_component_type::<Name>()?;
        self.reserve_component_type::<Hierarchy>()?;
        self.reserve_component_type::<LocalTransform>()?;
        self.reserve_component_type::<ActiveSelf>()?;
        self.reserve_component_type::<RenderLayerMask>()?;
        if record.camera.is_some() {
            self.reserve_component_type::<CameraComponent>()?;
        }
        if record.mesh.is_some() {
            self.reserve_component_type::<MeshRenderer>()?;
        }
        if record.sprite_2d.is_some() {
            self.reserve_component_type::<Sprite2dComponent>()?;
        }
        if record.mesh_2d.is_some() {
            self.reserve_component_type::<Mesh2dComponent>()?;
        }
        if record.rigid_body.is_some() {
            self.reserve_component_type::<RigidBodyComponent>()?;
        }
        if record.collider.is_some() {
            self.reserve_component_type::<ColliderComponent>()?;
        }
        if record.joint.is_some() {
            self.reserve_component_type::<JointComponent>()?;
        }
        if record.animation_skeleton.is_some() {
            self.reserve_component_type::<AnimationSkeletonComponent>()?;
        }
        if record.animation_player.is_some() {
            self.reserve_component_type::<AnimationPlayerComponent>()?;
        }
        if record.animation_sequence_player.is_some() {
            self.reserve_component_type::<AnimationSequencePlayerComponent>()?;
        }
        if record.animation_graph_player.is_some() {
            self.reserve_component_type::<AnimationGraphPlayerComponent>()?;
        }
        if record.animation_state_machine_player.is_some() {
            self.reserve_component_type::<AnimationStateMachinePlayerComponent>()?;
        }
        if record.ambient_light.is_some() {
            self.reserve_component_type::<AmbientLight>()?;
        }
        if record.directional_light.is_some() {
            self.reserve_component_type::<DirectionalLight>()?;
        }
        if record.point_light.is_some() {
            self.reserve_component_type::<PointLight>()?;
        }
        if record.rect_light.is_some() {
            self.reserve_component_type::<RectLight>()?;
        }
        if record.spot_light.is_some() {
            self.reserve_component_type::<SpotLight>()?;
        }
        self.reserve_component_type::<Mobility>()
    }

    fn reserve_component_type<T>(&mut self) -> SceneResult<()>
    where
        T: Component,
    {
        if self.world.registered_component_id::<T>().is_some() {
            return Ok(());
        }
        self.reserve_unregistered_component_type(TypeId::of::<T>())
    }

    fn reserve_unregistered_component_type(&mut self, type_id: TypeId) -> SceneResult<()> {
        if self.unregistered_component_types[..self.unregistered_component_count]
            .contains(&Some(type_id))
        {
            return Ok(());
        }
        let Some(component_slot) = self
            .unregistered_component_types
            .get_mut(self.unregistered_component_count)
        else {
            return Err(SceneError::Message(
                "bundle preflight exceeds the supported component width".to_string(),
            ));
        };
        *component_slot = Some(type_id);
        self.unregistered_component_count += 1;
        Ok(())
    }

    fn staged_component_id<T>(&mut self) -> SceneResult<ComponentId>
    where
        T: Component,
    {
        if let Some(component_id) = self.world.registered_component_id::<T>() {
            return Ok(component_id);
        }

        let type_id = TypeId::of::<T>();
        self.reserve_unregistered_component_type(type_id)?;
        let Some(index) = self.unregistered_component_types[..self.unregistered_component_count]
            .iter()
            .position(|candidate| *candidate == Some(type_id))
        else {
            return Err(SceneError::Message(
                "bundle preflight did not reserve its component type".to_string(),
            ));
        };
        Ok(ComponentId::new(
            self.world.component_registry.descriptors().len() + index,
        ))
    }
}

impl BundleStaging for BundleInsertionTransaction<'_> {
    fn stage<T>(&mut self, component: &T) -> SceneResult<()>
    where
        T: Component,
    {
        Self::stage(self, component)
    }

    fn validate_final_state(&self) -> SceneResult<()> {
        Self::validate_final_state(self)
    }

    fn commit<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        Self::commit(self, component)
    }
}

use std::any::TypeId;
use std::cell::Cell;

use super::{SceneError, SceneResult, World};
use crate::scene::components::{Hierarchy, Mobility, NodeRecord};
use crate::scene::ecs::{
    ArchetypeSignature, BundleStaging, Component, ComponentId, ComponentTicks, InternalEntity,
    LifecycleEventKind, PreflightedComponentInsert, StorageType,
};
use crate::scene::EntityId;

mod deferred_bundle_commit;
mod deferred_bundle_removals;
mod deferred_bundle_staging;
mod staging;

use deferred_bundle_removals::PendingDeferredRemoval;

const MAX_BUNDLE_COMPONENTS: usize = 8;
const MAX_NODE_RECORD_COMPONENT_TYPES: usize = 23;
const MAX_BUNDLE_COMPONENT_TYPES: usize = MAX_BUNDLE_COMPONENTS + MAX_NODE_RECORD_COMPONENT_TYPES;

#[derive(Clone, Copy)]
struct PreflightedBundleComponent {
    component_id: ComponentId,
    storage_type: StorageType,
    type_id: TypeId,
}

#[derive(Clone, Copy)]
struct UnregisteredBundleComponentType {
    type_id: TypeId,
    register_component_id: fn(&mut World) -> ComponentId,
}

fn register_component_id<T>(world: &mut World) -> ComponentId
where
    T: Component,
{
    world.component_id::<T>()
}

trait PendingBundleComponentValue {
    fn type_id(&self) -> TypeId;

    fn component_id(&self) -> ComponentId;

    fn storage_type(&self) -> StorageType;

    fn rebind_preflight(&mut self, world: &World, component_id: ComponentId) -> SceneResult<()>;

    fn prepare_effect(
        &self,
        was_present: bool,
        previous_hierarchy_parent: Option<EntityId>,
    ) -> PendingBundleEffect;

    fn publish_value(
        self: Box<Self>,
        world: &mut World,
        internal: InternalEntity,
        tick: crate::scene::ecs::ChangeTick,
    ) -> PendingBundlePublication;
}

#[derive(Clone, Copy)]
struct PendingBundleEffect {
    component_id: ComponentId,
    was_present: bool,
    previous_hierarchy_parent: Option<EntityId>,
    current_hierarchy_parent: Option<Option<EntityId>>,
    apply_typed: fn(&mut World, EntityId, &PendingBundleEffect),
}

impl PendingBundleEffect {
    fn for_component<T>(
        component_id: ComponentId,
        component: &T,
        was_present: bool,
        previous_hierarchy_parent: Option<EntityId>,
    ) -> Self
    where
        T: Component,
    {
        Self {
            component_id,
            was_present,
            previous_hierarchy_parent,
            current_hierarchy_parent: World::hierarchy_parent_from_component(component),
            apply_typed: apply_pending_bundle_effect::<T>,
        }
    }

    fn apply(self, world: &mut World, entity: EntityId) {
        (self.apply_typed)(world, entity, &self);
    }
}

fn apply_pending_bundle_effect<T>(world: &mut World, entity: EntityId, effect: &PendingBundleEffect)
where
    T: Component,
{
    world.mark_preflighted_bundle_component_mutation::<T>(entity);
    world.mark_preflighted_bundle_component_scene_binding_replacement::<T>(
        entity,
        effect.previous_hierarchy_parent,
        effect.current_hierarchy_parent,
    );
    let lifecycle_kind = if effect.was_present {
        LifecycleEventKind::Replace
    } else {
        LifecycleEventKind::Add
    };
    world.trigger_component_lifecycle(lifecycle_kind, entity, effect.component_id);
    world.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, effect.component_id);
}

struct TableBundlePublication {
    component_id: ComponentId,
    value: Box<dyn std::any::Any + Send + Sync>,
    ticks: ComponentTicks,
}

enum PendingBundlePublication {
    Table(TableBundlePublication),
    Sparse { replaced: bool },
}

struct PendingBundleValue<T>
where
    T: Component,
{
    component: T,
    storage_preflight: PreflightedComponentInsert<T>,
}

impl<T> PendingBundleComponentValue for PendingBundleValue<T>
where
    T: Component,
{
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn component_id(&self) -> ComponentId {
        self.storage_preflight.component_id()
    }

    fn storage_type(&self) -> StorageType {
        T::STORAGE_TYPE
    }

    fn rebind_preflight(&mut self, world: &World, component_id: ComponentId) -> SceneResult<()> {
        self.storage_preflight = world
            .component_storage
            .preflight_insert::<T>(component_id, T::STORAGE_TYPE)?;
        Ok(())
    }

    fn prepare_effect(
        &self,
        was_present: bool,
        previous_hierarchy_parent: Option<EntityId>,
    ) -> PendingBundleEffect {
        PendingBundleEffect::for_component::<T>(
            self.storage_preflight.component_id(),
            &self.component,
            was_present,
            previous_hierarchy_parent,
        )
    }

    fn publish_value(
        self: Box<Self>,
        world: &mut World,
        internal: InternalEntity,
        tick: crate::scene::ecs::ChangeTick,
    ) -> PendingBundlePublication {
        let PendingBundleValue {
            component,
            storage_preflight,
        } = *self;
        match T::STORAGE_TYPE {
            StorageType::Table => PendingBundlePublication::Table(TableBundlePublication {
                component_id: storage_preflight.component_id(),
                value: Box::new(component),
                ticks: ComponentTicks::new(tick),
            }),
            StorageType::SparseSet => {
                let replaced = world.component_storage.insert_preflighted_at_tick(
                    storage_preflight,
                    internal,
                    component,
                    tick,
                );
                PendingBundlePublication::Sparse { replaced }
            }
        }
    }
}

struct PreparedBundleValue {
    component: Box<dyn PendingBundleComponentValue>,
    preflight: PreflightedBundleComponent,
}

enum BundleTarget {
    Existing(InternalEntity),
    Spawn(NodeRecord),
}

enum CommitInput {
    Existing(InternalEntity),
    Spawn {
        record: NodeRecord,
        next_id: EntityId,
    },
}

struct CommitBoundary {
    internal: InternalEntity,
    spawned_entity: bool,
    prior_lifecycle_staging: bool,
    lifecycle_start: usize,
}

/// Holds the immutable checks and staged component values for one bundle
/// mutation. No entity or component storage changes until every component has
/// staged and the final state has validated.
pub(crate) struct BundleInsertionTransaction<'world> {
    world: &'world mut World,
    entity: EntityId,
    target: Option<BundleTarget>,
    spawn_next_id: Option<EntityId>,
    components: [Option<PreflightedBundleComponent>; MAX_BUNDLE_COMPONENTS],
    pending_values: [Option<Box<dyn PendingBundleComponentValue>>; MAX_BUNDLE_COMPONENTS],
    component_count: usize,
    default_values: [Option<Box<dyn PendingBundleComponentValue>>; MAX_NODE_RECORD_COMPONENT_TYPES],
    default_value_count: usize,
    unregistered_component_types:
        [Option<UnregisteredBundleComponentType>; MAX_BUNDLE_COMPONENT_TYPES],
    unregistered_component_count: usize,
    staged_hierarchy_parent: Option<Option<EntityId>>,
    staged_mobility: Option<Mobility>,
    deferred_removals: [Option<PendingDeferredRemoval>; MAX_BUNDLE_COMPONENT_TYPES],
    deferred_removal_count: usize,
    final_state_validated: Cell<bool>,
    defer_final_state_validation: bool,
}

/// A deferred final-row transaction whose payload no longer borrows World.
/// It lets one queue barrier validate every structural target before the
/// barrier reattaches the artifacts for their infallible publication pass.
pub(crate) struct DeferredBundleTransactionArtifact {
    entity: EntityId,
    target: Option<BundleTarget>,
    spawn_next_id: Option<EntityId>,
    components: [Option<PreflightedBundleComponent>; MAX_BUNDLE_COMPONENTS],
    pending_values: [Option<Box<dyn PendingBundleComponentValue>>; MAX_BUNDLE_COMPONENTS],
    component_count: usize,
    default_values: [Option<Box<dyn PendingBundleComponentValue>>; MAX_NODE_RECORD_COMPONENT_TYPES],
    default_value_count: usize,
    unregistered_component_types:
        [Option<UnregisteredBundleComponentType>; MAX_BUNDLE_COMPONENT_TYPES],
    unregistered_component_count: usize,
    staged_hierarchy_parent: Option<Option<EntityId>>,
    staged_mobility: Option<Mobility>,
    deferred_removals: [Option<PendingDeferredRemoval>; MAX_BUNDLE_COMPONENT_TYPES],
    deferred_removal_count: usize,
    final_state_validated: Cell<bool>,
    defer_final_state_validation: bool,
}

impl<'world> BundleInsertionTransaction<'world> {
    pub(crate) fn into_deferred_artifact(self) -> DeferredBundleTransactionArtifact {
        DeferredBundleTransactionArtifact {
            entity: self.entity,
            target: self.target,
            spawn_next_id: self.spawn_next_id,
            components: self.components,
            pending_values: self.pending_values,
            component_count: self.component_count,
            default_values: self.default_values,
            default_value_count: self.default_value_count,
            unregistered_component_types: self.unregistered_component_types,
            unregistered_component_count: self.unregistered_component_count,
            staged_hierarchy_parent: self.staged_hierarchy_parent,
            staged_mobility: self.staged_mobility,
            deferred_removals: self.deferred_removals,
            deferred_removal_count: self.deferred_removal_count,
            final_state_validated: self.final_state_validated,
            defer_final_state_validation: self.defer_final_state_validation,
        }
    }

    pub(crate) fn from_deferred_artifact(
        world: &'world mut World,
        artifact: DeferredBundleTransactionArtifact,
    ) -> Self {
        Self {
            world,
            entity: artifact.entity,
            target: artifact.target,
            spawn_next_id: artifact.spawn_next_id,
            components: artifact.components,
            pending_values: artifact.pending_values,
            component_count: artifact.component_count,
            default_values: artifact.default_values,
            default_value_count: artifact.default_value_count,
            unregistered_component_types: artifact.unregistered_component_types,
            unregistered_component_count: artifact.unregistered_component_count,
            staged_hierarchy_parent: artifact.staged_hierarchy_parent,
            staged_mobility: artifact.staged_mobility,
            deferred_removals: artifact.deferred_removals,
            deferred_removal_count: artifact.deferred_removal_count,
            final_state_validated: artifact.final_state_validated,
            defer_final_state_validation: artifact.defer_final_state_validation,
        }
    }

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
            components: [None; MAX_BUNDLE_COMPONENTS],
            pending_values: std::array::from_fn(|_| None),
            component_count: 0,
            default_values: std::array::from_fn(|_| None),
            default_value_count: 0,
            unregistered_component_types: [None; MAX_BUNDLE_COMPONENT_TYPES],
            unregistered_component_count: 0,
            staged_hierarchy_parent: None,
            staged_mobility: None,
            deferred_removals: [None; MAX_BUNDLE_COMPONENT_TYPES],
            deferred_removal_count: 0,
            final_state_validated: Cell::new(false),
            defer_final_state_validation: false,
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
            components: [None; MAX_BUNDLE_COMPONENTS],
            pending_values: std::array::from_fn(|_| None),
            component_count: 0,
            default_values: std::array::from_fn(|_| None),
            default_value_count: 0,
            unregistered_component_types: [None; MAX_BUNDLE_COMPONENT_TYPES],
            unregistered_component_count: 0,
            staged_hierarchy_parent: None,
            staged_mobility: None,
            deferred_removals: [None; MAX_BUNDLE_COMPONENT_TYPES],
            deferred_removal_count: 0,
            final_state_validated: Cell::new(false),
            defer_final_state_validation: false,
        };
        transaction.stage_default_node_record_components(&record)?;
        transaction.target = Some(BundleTarget::Spawn(record));
        Ok(transaction)
    }

    fn final_archetype_signature(
        &self,
        spawns_entity: bool,
        default_values: &[Option<Box<dyn PendingBundleComponentValue>>;
             MAX_NODE_RECORD_COMPONENT_TYPES],
        prepared_values: &[Option<PreparedBundleValue>; MAX_BUNDLE_COMPONENTS],
        deferred_removals: &[Option<PendingDeferredRemoval>; MAX_BUNDLE_COMPONENT_TYPES],
        deferred_removal_count: usize,
    ) -> ArchetypeSignature {
        let mut signature = if spawns_entity {
            ArchetypeSignature::empty()
        } else {
            self.world
                .entity_archetype_signature(self.entity)
                .expect("existing bundle entity must have an archetype signature")
        };
        for default_value in default_values
            .iter()
            .take(self.default_value_count)
            .flatten()
        {
            signature = signature
                .with_component_added(default_value.component_id(), default_value.storage_type());
        }
        for prepared in prepared_values.iter().take(self.component_count).flatten() {
            signature = signature.with_component_added(
                prepared.preflight.component_id,
                prepared.preflight.storage_type,
            );
        }
        for removal in deferred_removals
            .iter()
            .take(deferred_removal_count)
            .flatten()
        {
            signature =
                signature.with_component_removed(removal.component_id(), removal.storage_type());
        }
        signature
    }

    fn validate_final_state(&self) -> SceneResult<()> {
        if self.defer_final_state_validation {
            return Ok(());
        }
        if self.has_deferred_removal::<Hierarchy>() || self.has_deferred_removal::<Mobility>() {
            self.final_state_validated.set(true);
            return Ok(());
        }
        let parent = self.staged_hierarchy_parent.unwrap_or_else(|| {
            self.world
                .get::<Hierarchy>(self.entity)
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

    pub(crate) fn finish(self) -> SceneResult<()> {
        self.finish_with_deferred_spawn(None)
    }

    fn validate_commit_invariants(&self) -> SceneResult<()> {
        for component_index in 0..self.component_count {
            if self.pending_values[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "staged component value is missing",
                });
            }
            if self.components[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "staged component preflight is missing",
                });
            }
        }
        for component_index in 0..self.default_value_count {
            if self.default_values[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "staged node record component value is missing",
                });
            }
        }
        for component_index in 0..self.unregistered_component_count {
            if self.unregistered_component_types[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "reserved component type is missing",
                });
            }
        }
        if self.target.is_none() {
            return Err(SceneError::BundleTransactionInvariant {
                reason: "pending entity target is missing",
            });
        }
        if matches!(self.target, Some(BundleTarget::Spawn(_))) && self.spawn_next_id.is_none() {
            return Err(SceneError::BundleTransactionInvariant {
                reason: "spawn allocator successor is missing",
            });
        }
        Ok(())
    }

    // Values are detached before descriptor materialization and publication.
    fn take_prepared_values(
        &mut self,
    ) -> SceneResult<[Option<PreparedBundleValue>; MAX_BUNDLE_COMPONENTS]> {
        let mut prepared_values = std::array::from_fn(|_| None);
        for component_index in 0..self.component_count {
            let component = self.pending_values[component_index].take().ok_or_else(|| {
                SceneError::BundleTransactionInvariant {
                    reason: "staged component value is missing",
                }
            })?;
            let preflight = self.components[component_index].take().ok_or_else(|| {
                SceneError::BundleTransactionInvariant {
                    reason: "staged component preflight is missing",
                }
            })?;
            prepared_values[component_index] = Some(PreparedBundleValue {
                component,
                preflight,
            });
        }
        Ok(prepared_values)
    }

    fn take_default_values(
        &mut self,
    ) -> SceneResult<[Option<Box<dyn PendingBundleComponentValue>>; MAX_NODE_RECORD_COMPONENT_TYPES]>
    {
        let mut values = std::array::from_fn(|_| None);
        for component_index in 0..self.default_value_count {
            values[component_index] =
                Some(self.default_values[component_index].take().ok_or_else(|| {
                    SceneError::BundleTransactionInvariant {
                        reason: "staged node record component value is missing",
                    }
                })?);
        }
        Ok(values)
    }

    fn prepare_commit(&mut self) -> SceneResult<CommitInput> {
        let target = self
            .target
            .take()
            .ok_or_else(|| SceneError::BundleTransactionInvariant {
                reason: "pending entity target is missing",
            })?;
        match target {
            BundleTarget::Spawn(record) => {
                let next_id =
                    self.spawn_next_id
                        .ok_or_else(|| SceneError::BundleTransactionInvariant {
                            reason: "spawn allocator successor is missing",
                        })?;
                Ok(CommitInput::Spawn { record, next_id })
            }
            BundleTarget::Existing(internal) => {
                if self.world.internal_entity(self.entity) != Some(internal) {
                    return Err(SceneError::missing_entity("commit bundle for", self.entity));
                }
                Ok(CommitInput::Existing(internal))
            }
        }
    }

    fn begin_commit(&mut self, input: CommitInput) -> CommitBoundary {
        let lifecycle_start = self.world.staged_lifecycle_events.len();
        let prior_lifecycle_staging =
            std::mem::replace(&mut self.world.record_staged_lifecycle_events, true);
        let (internal, spawned_entity) = match input {
            CommitInput::Existing(internal) => (internal, false),
            CommitInput::Spawn { record, next_id } => {
                let internal = self
                    .world
                    .register_prevalidated_node_identity_without_components(&record);
                self.world.next_id = self.world.next_id.max(next_id);
                (internal, true)
            }
        };
        CommitBoundary {
            internal,
            spawned_entity,
            prior_lifecycle_staging,
            lifecycle_start,
        }
    }

    pub(crate) fn new_deferred_spawn(
        world: &'world mut World,
        record: NodeRecord,
        include_default_components: bool,
    ) -> SceneResult<Self> {
        if include_default_components {
            let mut transaction = Self::new_spawn(world, record)?;
            transaction.defer_final_state_validation = true;
            return Ok(transaction);
        }
        let entity = record.id;
        let spawn_next_id = entity
            .checked_add(1)
            .ok_or(SceneError::EntityIdExhausted { entity })?;
        Ok(Self {
            world,
            entity,
            target: Some(BundleTarget::Spawn(record)),
            spawn_next_id: Some(spawn_next_id),
            components: [None; MAX_BUNDLE_COMPONENTS],
            pending_values: std::array::from_fn(|_| None),
            component_count: 0,
            default_values: std::array::from_fn(|_| None),
            default_value_count: 0,
            unregistered_component_types: [None; MAX_BUNDLE_COMPONENT_TYPES],
            unregistered_component_count: 0,
            staged_hierarchy_parent: None,
            staged_mobility: None,
            deferred_removals: [None; MAX_BUNDLE_COMPONENT_TYPES],
            deferred_removal_count: 0,
            final_state_validated: Cell::new(false),
            defer_final_state_validation: true,
        })
    }
}

impl DeferredBundleTransactionArtifact {
    pub(crate) fn preflight(&mut self, world: &World) -> SceneResult<()> {
        if self.defer_final_state_validation {
            self.defer_final_state_validation = false;
        }
        self.validate_final_state(world)?;
        self.validate_commit_invariants()?;
        self.rebind_component_preflights(world)?;
        Ok(())
    }

    pub(crate) fn publish_spawn(
        self,
        world: &mut World,
        deferred_spawn: Option<crate::scene::ecs::DeferredSpawnToken>,
    ) -> SceneResult<()> {
        BundleInsertionTransaction::from_deferred_artifact(world, self)
            .finish_with_deferred_spawn(deferred_spawn)
    }

    pub(crate) fn publish_despawn(self, world: &mut World) -> SceneResult<()> {
        BundleInsertionTransaction::from_deferred_artifact(world, self)
            .finish_with_deferred_despawn()
    }

    pub(crate) fn preflight_despawn(&self, world: &World) -> SceneResult<()> {
        match self.target.as_ref() {
            Some(BundleTarget::Spawn(_)) => Ok(()),
            Some(BundleTarget::Existing(_)) => world.preflight_deferred_despawn(self.entity),
            None => Err(SceneError::BundleTransactionInvariant {
                reason: "pending entity target is missing",
            }),
        }
    }

    pub(crate) fn preflight_batch_relationships(
        &self,
        despawned_entities: &std::collections::BTreeSet<EntityId>,
    ) -> SceneResult<()> {
        let Some(Some(parent)) = self.staged_hierarchy_parent else {
            return Ok(());
        };
        if despawned_entities.contains(&parent) {
            return Err(SceneError::MissingParent {
                child: self.entity,
                parent,
            });
        }
        Ok(())
    }

    pub(crate) fn materialize_component_reservations(&mut self, world: &mut World) {
        let mut resolved_ids = std::collections::BTreeMap::new();
        for reservation in self.unregistered_component_types[..self.unregistered_component_count]
            .iter()
            .flatten()
        {
            let component_id = (reservation.register_component_id)(world);
            resolved_ids.insert(reservation.type_id, component_id);
        }
        for component in self.components[..self.component_count].iter_mut().flatten() {
            if let Some(component_id) = resolved_ids.get(&component.type_id).copied() {
                component.component_id = component_id;
            }
        }
        for value in self.pending_values[..self.component_count]
            .iter_mut()
            .flatten()
        {
            let component_id = resolved_ids
                .get(&value.type_id())
                .copied()
                .unwrap_or_else(|| value.component_id());
            value
                .rebind_preflight(world, component_id)
                .expect("preflighted deferred component must retain its storage contract");
        }
        for value in self.default_values[..self.default_value_count]
            .iter_mut()
            .flatten()
        {
            let component_id = resolved_ids
                .get(&value.type_id())
                .copied()
                .unwrap_or_else(|| value.component_id());
            value
                .rebind_preflight(world, component_id)
                .expect("preflighted deferred default component must retain its storage contract");
        }
        self.unregistered_component_count = 0;
    }

    fn validate_final_state(&mut self, world: &World) -> SceneResult<()> {
        if self.has_deferred_removal::<Hierarchy>() || self.has_deferred_removal::<Mobility>() {
            self.final_state_validated.set(true);
            return Ok(());
        }
        let parent = self.staged_hierarchy_parent.unwrap_or_else(|| {
            world
                .get::<Hierarchy>(self.entity)
                .and_then(|value| value.parent)
        });
        let mobility = self
            .staged_mobility
            .unwrap_or_else(|| world.mobility(self.entity).unwrap_or_default());
        world.validate_bundle_mobility_state(self.entity, parent, mobility)?;
        self.final_state_validated.set(true);
        Ok(())
    }

    fn validate_commit_invariants(&self) -> SceneResult<()> {
        for component_index in 0..self.component_count {
            if self.pending_values[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "staged component value is missing",
                });
            }
            if self.components[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "staged component preflight is missing",
                });
            }
        }
        for component_index in 0..self.default_value_count {
            if self.default_values[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "staged node record component value is missing",
                });
            }
        }
        for component_index in 0..self.unregistered_component_count {
            if self.unregistered_component_types[component_index].is_none() {
                return Err(SceneError::BundleTransactionInvariant {
                    reason: "reserved component type is missing",
                });
            }
        }
        if self.target.is_none() {
            return Err(SceneError::BundleTransactionInvariant {
                reason: "pending entity target is missing",
            });
        }
        if matches!(self.target, Some(BundleTarget::Spawn(_))) && self.spawn_next_id.is_none() {
            return Err(SceneError::BundleTransactionInvariant {
                reason: "spawn allocator successor is missing",
            });
        }
        Ok(())
    }

    fn rebind_component_preflights(&mut self, world: &World) -> SceneResult<()> {
        for value in self.pending_values[..self.component_count]
            .iter_mut()
            .flatten()
        {
            let component_id = value.component_id();
            value.rebind_preflight(world, component_id)?;
        }
        for value in self.default_values[..self.default_value_count]
            .iter_mut()
            .flatten()
        {
            let component_id = value.component_id();
            value.rebind_preflight(world, component_id)?;
        }
        Ok(())
    }

    fn has_deferred_removal<T>(&self) -> bool
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        self.deferred_removals[..self.deferred_removal_count]
            .iter()
            .flatten()
            .any(|removal| removal.type_id() == type_id)
    }
}

impl BundleStaging for BundleInsertionTransaction<'_> {
    fn stage<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        Self::stage(self, component)
    }

    fn validate_final_state(&self) -> SceneResult<()> {
        Self::validate_final_state(self)
    }
}

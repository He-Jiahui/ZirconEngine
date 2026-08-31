use std::sync::Arc;

use crate::core::framework::render::{
    RenderComponentChangeArtifact, RenderComponentChangeKind, RenderComponentChangeMask,
    RenderComponentChangeStats, RenderComponentFullReprojectionReason, RenderComponentMeshLodLevel,
    RenderComponentMeshPayload, RenderComponentMeshPrimitiveBinding, RenderComponentProjectionMode,
    RenderComponentSnapshot, RenderComponentSourceWorldId, RenderComponentValue,
};
use crate::scene::components::{
    ActiveInHierarchy, MeshRenderer, Mobility, RenderLayerMask, WorldMatrix,
};
use crate::scene::ecs::{
    ChangeTick, ChangeTickWindow, Component, RemovedComponentEvents, RemovedComponentReader,
};
use crate::scene::{EntityId, World};

use super::super::render_dirty_journal::{RenderDirtyEntityJournal, RenderDirtyWorldId};
#[derive(Clone, Debug, Default)]
struct RemovalReaders {
    mesh_renderer: RemovedComponentReader<MeshRenderer>,
    world_matrix: RemovedComponentReader<WorldMatrix>,
    active_in_hierarchy: RemovedComponentReader<ActiveInHierarchy>,
    render_layer_mask: RemovedComponentReader<RenderLayerMask>,
    mobility: RemovedComponentReader<Mobility>,
}

impl RemovalReaders {
    fn clear(&mut self, events: &RemovedComponentEvents) {
        self.mesh_renderer.clear(events);
        self.world_matrix.clear(events);
        self.active_in_hierarchy.clear(events);
        self.render_layer_mask.clear(events);
        self.mobility.clear(events);
    }

    fn read(&mut self, events: &RemovedComponentEvents) -> RemovalWindows {
        RemovalWindows {
            mesh_renderer: read_removals(&mut self.mesh_renderer, events),
            world_matrix: read_removals(&mut self.world_matrix, events),
            active_in_hierarchy: read_removals(&mut self.active_in_hierarchy, events),
            render_layer_mask: read_removals(&mut self.render_layer_mask, events),
            mobility: read_removals(&mut self.mobility, events),
        }
    }
}

#[derive(Default)]
struct RemovalWindow {
    entities: Vec<EntityId>,
    events_read: usize,
    dropped: u64,
}

impl RemovalWindow {
    fn contains(&self, entity: EntityId) -> bool {
        self.entities.binary_search(&entity).is_ok()
    }
}

#[derive(Default)]
struct RemovalWindows {
    mesh_renderer: RemovalWindow,
    world_matrix: RemovalWindow,
    active_in_hierarchy: RemovalWindow,
    render_layer_mask: RemovalWindow,
    mobility: RemovalWindow,
}

impl RemovalWindows {
    fn events_read(&self) -> usize {
        self.mesh_renderer
            .events_read
            .saturating_add(self.world_matrix.events_read)
            .saturating_add(self.active_in_hierarchy.events_read)
            .saturating_add(self.render_layer_mask.events_read)
            .saturating_add(self.mobility.events_read)
    }

    fn dropped(&self) -> u64 {
        self.mesh_renderer
            .dropped
            .saturating_add(self.world_matrix.dropped)
            .saturating_add(self.active_in_hierarchy.dropped)
            .saturating_add(self.render_layer_mask.dropped)
            .saturating_add(self.mobility.dropped)
    }
}

#[derive(Clone, Debug, Default)]
pub(in crate::scene::world) struct RenderComponentChangeProjector {
    bound_world: Option<RenderDirtyWorldId>,
    journal_generation: u64,
    source_change_tick: ChangeTick,
    removal_readers: RemovalReaders,
    published: Option<Arc<RenderComponentChangeArtifact>>,
}

impl PartialEq for RenderComponentChangeProjector {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for RenderComponentChangeProjector {}

impl RenderComponentChangeProjector {
    pub(in crate::scene::world) fn publish(
        &mut self,
        world: &World,
        journal: &RenderDirtyEntityJournal,
    ) {
        if self.bound_world == Some(journal.world())
            && self.journal_generation == journal.generation()
            && self.published.is_some()
        {
            return;
        }

        let source_world_generation = world.world_generation();
        let source_change_tick = world.read_change_tick();
        let mut mode =
            self.required_full_projection(journal, source_world_generation, source_change_tick);
        let world_rebound = self
            .bound_world
            .is_some_and(|world| world != journal.world());
        let mut removal_readers = if world_rebound {
            RemovalReaders::default()
        } else {
            self.removal_readers.clone()
        };
        let mut stats = RenderComponentChangeStats {
            candidate_entities: journal.entities().len(),
            ..RenderComponentChangeStats::default()
        };
        let removal_windows = if mode.is_some() {
            RemovalWindows::default()
        } else {
            let windows = removal_readers.read(world.removed_component_events());
            stats.removal_events_read = windows.events_read();
            stats.removal_events_dropped = windows.dropped();
            if stats.removal_events_dropped != 0 {
                mode = Some(RenderComponentFullReprojectionReason::RemovalHistoryLoss);
            }
            windows
        };

        let (mode, mut upserts, removals) = match mode {
            Some(reason) => {
                removal_readers.clear(world.removed_component_events());
                let upserts = project_full_world(world, &mut stats);
                (
                    RenderComponentProjectionMode::Full(reason),
                    upserts,
                    Vec::new(),
                )
            }
            None => {
                let window = ChangeTickWindow::new(self.source_change_tick, source_change_tick);
                let (upserts, removals) =
                    project_incremental(world, journal, window, &removal_windows, &mut stats);
                (
                    RenderComponentProjectionMode::Incremental,
                    upserts,
                    removals,
                )
            }
        };
        upserts.sort_unstable_by_key(RenderComponentSnapshot::entity);
        stats.upserts = upserts.len();
        stats.removals = removals.len();

        self.bound_world = Some(journal.world());
        self.journal_generation = journal.generation();
        self.source_change_tick = source_change_tick;
        self.removal_readers = removal_readers;
        self.published = Some(Arc::new(RenderComponentChangeArtifact::new(
            RenderComponentSourceWorldId::new(journal.world().raw()),
            journal.generation(),
            source_world_generation,
            source_change_tick.get(),
            mode,
            upserts,
            removals,
            stats,
        )));
    }

    pub(in crate::scene::world) fn published(&self) -> Option<Arc<RenderComponentChangeArtifact>> {
        self.published.as_ref().map(Arc::clone)
    }

    fn required_full_projection(
        &self,
        journal: &RenderDirtyEntityJournal,
        source_world_generation: u64,
        source_change_tick: ChangeTick,
    ) -> Option<RenderComponentFullReprojectionReason> {
        let Some(bound_world) = self.bound_world else {
            return Some(RenderComponentFullReprojectionReason::InitialProjection);
        };
        if bound_world != journal.world() {
            return Some(RenderComponentFullReprojectionReason::WorldRebound);
        }
        if journal.source_world_generation() != source_world_generation
            || journal.source_change_tick() != source_change_tick
        {
            return Some(RenderComponentFullReprojectionReason::SourceDrift);
        }
        if journal.all_entities() {
            return Some(RenderComponentFullReprojectionReason::JournalRequested);
        }
        if journal.generation() != self.journal_generation.saturating_add(1) {
            return Some(RenderComponentFullReprojectionReason::JournalDiscontinuity);
        }
        None
    }
}

fn project_full_world(
    world: &World,
    stats: &mut RenderComponentChangeStats,
) -> Vec<RenderComponentSnapshot> {
    let entities = world.stable_entity_ids().collect::<Vec<_>>();
    stats.full_scan_entities = entities.len();
    let mut upserts = Vec::new();
    for entity in entities {
        let Some(mesh_renderer) = world.get::<MeshRenderer>(entity) else {
            continue;
        };
        upserts.push(snapshot(
            world,
            entity,
            mesh_renderer,
            RenderComponentChangeKind::Added,
            RenderComponentChangeMask::ALL,
            stats,
        ));
    }
    upserts
}

fn project_incremental(
    world: &World,
    journal: &RenderDirtyEntityJournal,
    window: ChangeTickWindow,
    removals: &RemovalWindows,
    stats: &mut RenderComponentChangeStats,
) -> (Vec<RenderComponentSnapshot>, Vec<EntityId>) {
    let mut upserts = Vec::new();
    for entity in journal.entities().iter().copied() {
        let Some(mesh_renderer) = world.get::<MeshRenderer>(entity) else {
            continue;
        };

        let mesh_change = probe_component::<MeshRenderer>(world, entity, window, stats);
        let world_matrix_change = probe_component::<WorldMatrix>(world, entity, window, stats);
        let active_change = probe_component::<ActiveInHierarchy>(world, entity, window, stats);
        let layer_change = probe_component::<RenderLayerMask>(world, entity, window, stats);
        let mobility_change = probe_component::<Mobility>(world, entity, window, stats);
        let mesh_readded = removals.mesh_renderer.contains(entity);
        let kind = if mesh_change.added || mesh_readded {
            RenderComponentChangeKind::Added
        } else {
            RenderComponentChangeKind::Updated
        };
        let mut mask = RenderComponentChangeMask::default();
        if mesh_change.changed {
            mask |= RenderComponentChangeMask::MESH_RENDERER;
        }
        if world_matrix_change.changed || removals.world_matrix.contains(entity) {
            mask |= RenderComponentChangeMask::WORLD_TRANSFORM;
        }
        if active_change.changed || removals.active_in_hierarchy.contains(entity) {
            mask |= RenderComponentChangeMask::ACTIVE_IN_HIERARCHY;
        }
        if layer_change.changed || removals.render_layer_mask.contains(entity) {
            mask |= RenderComponentChangeMask::RENDER_LAYER;
        }
        if mobility_change.changed || removals.mobility.contains(entity) {
            mask |= RenderComponentChangeMask::MOBILITY;
        }
        if kind == RenderComponentChangeKind::Added {
            mask = RenderComponentChangeMask::ALL;
        }
        if mask.is_empty() {
            continue;
        }
        upserts.push(snapshot(world, entity, mesh_renderer, kind, mask, stats));
    }

    let mut primitive_removals = removals
        .mesh_renderer
        .entities
        .iter()
        .copied()
        .filter(|entity| world.get::<MeshRenderer>(*entity).is_none())
        .collect::<Vec<_>>();
    primitive_removals.sort_unstable();
    primitive_removals.dedup();
    (upserts, primitive_removals)
}

#[derive(Clone, Copy, Default)]
struct ComponentChange {
    added: bool,
    changed: bool,
}

fn probe_component<T>(
    world: &World,
    entity: EntityId,
    window: ChangeTickWindow,
    stats: &mut RenderComponentChangeStats,
) -> ComponentChange
where
    T: Component,
{
    stats.component_tick_probes = stats.component_tick_probes.saturating_add(1);
    let Some(ticks) = world.component_change_ticks::<T>(entity) else {
        return ComponentChange::default();
    };
    ComponentChange {
        added: ticks.is_added(window),
        changed: ticks.is_changed(window),
    }
}

fn snapshot(
    world: &World,
    entity: EntityId,
    mesh_renderer: &MeshRenderer,
    kind: RenderComponentChangeKind,
    mask: RenderComponentChangeMask,
    stats: &mut RenderComponentChangeStats,
) -> RenderComponentSnapshot {
    let mesh_renderer = if mask.contains(RenderComponentChangeMask::MESH_RENDERER) {
        stats.mesh_renderer_payload_clones = stats.mesh_renderer_payload_clones.saturating_add(1);
        RenderComponentValue::Present(project_mesh_renderer(mesh_renderer))
    } else {
        RenderComponentValue::Unchanged
    };
    RenderComponentSnapshot::new(
        entity,
        kind,
        mask,
        mesh_renderer,
        copy_component_patch::<WorldMatrix, _>(
            world,
            entity,
            mask.contains(RenderComponentChangeMask::WORLD_TRANSFORM),
            |matrix| matrix.0,
        ),
        copy_component_patch::<ActiveInHierarchy, _>(
            world,
            entity,
            mask.contains(RenderComponentChangeMask::ACTIVE_IN_HIERARCHY),
            |active| active.0,
        ),
        copy_component_patch::<RenderLayerMask, _>(
            world,
            entity,
            mask.contains(RenderComponentChangeMask::RENDER_LAYER),
            |layer| layer.0,
        ),
        copy_component_patch::<Mobility, _>(
            world,
            entity,
            mask.contains(RenderComponentChangeMask::MOBILITY),
            |mobility| mobility,
        ),
    )
}

fn copy_component_patch<T, U>(
    world: &World,
    entity: EntityId,
    included: bool,
    project: impl FnOnce(T) -> U,
) -> RenderComponentValue<U>
where
    T: Component + Copy,
{
    if !included {
        return RenderComponentValue::Unchanged;
    }
    world
        .get::<T>(entity)
        .copied()
        .map(project)
        .map(RenderComponentValue::Present)
        .unwrap_or(RenderComponentValue::Removed)
}

fn project_mesh_renderer(mesh: &MeshRenderer) -> RenderComponentMeshPayload {
    RenderComponentMeshPayload::new(
        mesh.model,
        mesh.mesh,
        mesh.material,
        mesh.render_queue,
        mesh.material_queue,
        mesh.order_in_layer,
        mesh.depth_bias,
        mesh.morph_weights.clone(),
        project_primitive_bindings(&mesh.primitives),
        mesh.lods
            .iter()
            .map(|lod| {
                RenderComponentMeshLodLevel::new(
                    lod.min_distance,
                    lod.model,
                    lod.mesh,
                    lod.material,
                    project_primitive_bindings(&lod.primitives),
                )
            })
            .collect(),
        mesh.material_property_overrides.clone(),
        mesh.tint,
        mesh.material_alpha_mode,
    )
}

fn project_primitive_bindings(
    bindings: &[crate::scene::components::MeshRendererPrimitiveBinding],
) -> Vec<RenderComponentMeshPrimitiveBinding> {
    bindings
        .iter()
        .map(|binding| RenderComponentMeshPrimitiveBinding::new(binding.mesh, binding.material))
        .collect()
}

fn read_removals<T>(
    reader: &mut RemovedComponentReader<T>,
    events: &RemovedComponentEvents,
) -> RemovalWindow
where
    T: 'static,
{
    let dropped_before = reader.dropped_count();
    let mut entities = reader.read(events).collect::<Vec<_>>();
    let events_read = entities.len();
    entities.sort_unstable();
    entities.dedup();
    RemovalWindow {
        entities,
        events_read,
        dropped: reader.dropped_count().saturating_sub(dropped_before),
    }
}

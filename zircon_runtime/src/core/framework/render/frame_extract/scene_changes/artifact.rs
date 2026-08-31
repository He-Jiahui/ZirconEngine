use std::sync::Arc;

use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::Mat4;

use super::{RenderComponentChangeMask, RenderComponentMeshPayload};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderComponentSourceWorldId(u64);

impl RenderComponentSourceWorldId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderComponentChangeKind {
    Added,
    Updated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderComponentFullReprojectionReason {
    InitialProjection,
    WorldRebound,
    SourceDrift,
    JournalRequested,
    JournalDiscontinuity,
    RemovalHistoryLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderComponentProjectionMode {
    Incremental,
    Full(RenderComponentFullReprojectionReason),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderComponentValue<T> {
    Unchanged,
    Present(T),
    Removed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderComponentSnapshot {
    entity: EntityId,
    kind: RenderComponentChangeKind,
    mask: RenderComponentChangeMask,
    mesh_renderer: RenderComponentValue<RenderComponentMeshPayload>,
    world_matrix: RenderComponentValue<Mat4>,
    active_in_hierarchy: RenderComponentValue<bool>,
    render_layer_mask: RenderComponentValue<u32>,
    mobility: RenderComponentValue<Mobility>,
}

impl RenderComponentSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        entity: EntityId,
        kind: RenderComponentChangeKind,
        mask: RenderComponentChangeMask,
        mesh_renderer: RenderComponentValue<RenderComponentMeshPayload>,
        world_matrix: RenderComponentValue<Mat4>,
        active_in_hierarchy: RenderComponentValue<bool>,
        render_layer_mask: RenderComponentValue<u32>,
        mobility: RenderComponentValue<Mobility>,
    ) -> Self {
        Self {
            entity,
            kind,
            mask,
            mesh_renderer,
            world_matrix,
            active_in_hierarchy,
            render_layer_mask,
            mobility,
        }
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn kind(&self) -> RenderComponentChangeKind {
        self.kind
    }

    pub const fn mask(&self) -> RenderComponentChangeMask {
        self.mask
    }

    pub const fn mesh_renderer(&self) -> &RenderComponentValue<RenderComponentMeshPayload> {
        &self.mesh_renderer
    }

    pub const fn world_matrix(&self) -> &RenderComponentValue<Mat4> {
        &self.world_matrix
    }

    pub const fn active_in_hierarchy(&self) -> &RenderComponentValue<bool> {
        &self.active_in_hierarchy
    }

    pub const fn render_layer_mask(&self) -> &RenderComponentValue<u32> {
        &self.render_layer_mask
    }

    pub const fn mobility(&self) -> &RenderComponentValue<Mobility> {
        &self.mobility
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderComponentChangeStats {
    pub(crate) candidate_entities: usize,
    pub(crate) full_scan_entities: usize,
    pub(crate) component_tick_probes: usize,
    pub(crate) removal_events_read: usize,
    pub(crate) removal_events_dropped: u64,
    pub(crate) mesh_renderer_payload_clones: usize,
    pub(crate) upserts: usize,
    pub(crate) removals: usize,
}

impl RenderComponentChangeStats {
    pub const fn candidate_entities(self) -> usize {
        self.candidate_entities
    }

    pub const fn full_scan_entities(self) -> usize {
        self.full_scan_entities
    }

    pub const fn component_tick_probes(self) -> usize {
        self.component_tick_probes
    }

    pub const fn removal_events_read(self) -> usize {
        self.removal_events_read
    }

    pub const fn removal_events_dropped(self) -> u64 {
        self.removal_events_dropped
    }

    pub const fn mesh_renderer_payload_clones(self) -> usize {
        self.mesh_renderer_payload_clones
    }

    pub const fn upserts(self) -> usize {
        self.upserts
    }

    pub const fn removals(self) -> usize {
        self.removals
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderComponentChangeArtifact {
    world: RenderComponentSourceWorldId,
    journal_generation: u64,
    source_world_generation: u64,
    source_change_tick: u64,
    mode: RenderComponentProjectionMode,
    upserts: Arc<[RenderComponentSnapshot]>,
    removals: Arc<[EntityId]>,
    stats: RenderComponentChangeStats,
}

impl RenderComponentChangeArtifact {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        world: RenderComponentSourceWorldId,
        journal_generation: u64,
        source_world_generation: u64,
        source_change_tick: u64,
        mode: RenderComponentProjectionMode,
        upserts: Vec<RenderComponentSnapshot>,
        removals: Vec<EntityId>,
        stats: RenderComponentChangeStats,
    ) -> Self {
        Self {
            world,
            journal_generation,
            source_world_generation,
            source_change_tick,
            mode,
            upserts: upserts.into(),
            removals: removals.into(),
            stats,
        }
    }

    pub const fn world(&self) -> RenderComponentSourceWorldId {
        self.world
    }

    pub const fn journal_generation(&self) -> u64 {
        self.journal_generation
    }

    pub const fn source_world_generation(&self) -> u64 {
        self.source_world_generation
    }

    pub const fn source_change_tick(&self) -> u64 {
        self.source_change_tick
    }

    pub const fn mode(&self) -> RenderComponentProjectionMode {
        self.mode
    }

    pub fn upserts(&self) -> &[RenderComponentSnapshot] {
        &self.upserts
    }

    pub fn removals(&self) -> &[EntityId] {
        &self.removals
    }

    pub const fn stats(&self) -> RenderComponentChangeStats {
        self.stats
    }
}

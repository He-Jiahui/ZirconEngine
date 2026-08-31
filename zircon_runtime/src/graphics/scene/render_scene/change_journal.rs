use std::ops::{BitOr, BitOrAssign};
use std::sync::Arc;

use crate::core::framework::render::RenderWorldSnapshotHandle;

use super::primitive::RenderScenePrimitive;
use super::resource_dependencies::{
    RenderSceneResourceReferenceDelta, RenderSceneResourceReferenceDeltaStats,
    build_resource_reference_deltas,
};
use super::scene::{RenderSceneGeneration, RenderScenePrimitiveHandle, RenderSceneStorageStats};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderSceneDirtyDomainCounts {
    transform_count: usize,
    geometry_count: usize,
    material_count: usize,
    deformation_count: usize,
    render_state_count: usize,
    visibility_count: usize,
    bounds_count: usize,
}

impl RenderSceneDirtyDomainCounts {
    pub(super) fn record(&mut self, dirty: RenderScenePrimitiveDirtyFlags) {
        self.transform_count += dirty.contains(RenderScenePrimitiveDirtyFlags::TRANSFORM) as usize;
        self.geometry_count += dirty.contains(RenderScenePrimitiveDirtyFlags::GEOMETRY) as usize;
        self.material_count += dirty.contains(RenderScenePrimitiveDirtyFlags::MATERIAL) as usize;
        self.deformation_count +=
            dirty.contains(RenderScenePrimitiveDirtyFlags::DEFORMATION) as usize;
        self.render_state_count +=
            dirty.contains(RenderScenePrimitiveDirtyFlags::RENDER_STATE) as usize;
        self.visibility_count +=
            dirty.contains(RenderScenePrimitiveDirtyFlags::VISIBILITY) as usize;
        self.bounds_count += dirty.contains(RenderScenePrimitiveDirtyFlags::BOUNDS) as usize;
    }

    pub(crate) const fn transform_count(self) -> usize {
        self.transform_count
    }

    pub(crate) const fn geometry_count(self) -> usize {
        self.geometry_count
    }

    pub(crate) const fn material_count(self) -> usize {
        self.material_count
    }

    pub(crate) const fn deformation_count(self) -> usize {
        self.deformation_count
    }

    pub(crate) const fn render_state_count(self) -> usize {
        self.render_state_count
    }

    pub(crate) const fn visibility_count(self) -> usize {
        self.visibility_count
    }

    pub(crate) const fn bounds_count(self) -> usize {
        self.bounds_count
    }

    pub(crate) const fn total_count(self) -> usize {
        self.transform_count
            + self.geometry_count
            + self.material_count
            + self.deformation_count
            + self.render_state_count
            + self.visibility_count
            + self.bounds_count
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderSceneApplyStats {
    input_upsert_count: usize,
    input_removal_count: usize,
    stable_key_lookup_count: usize,
    primitive_comparison_count: usize,
    dirty_domain_counts: RenderSceneDirtyDomainCounts,
    reused_handle_slot_count: usize,
    appended_handle_slot_count: usize,
    dense_relocation_count: usize,
    storage_stats: RenderSceneStorageStats,
}

impl RenderSceneApplyStats {
    pub(super) const fn new(
        input_upsert_count: usize,
        input_removal_count: usize,
        stable_key_lookup_count: usize,
        primitive_comparison_count: usize,
        dirty_domain_counts: RenderSceneDirtyDomainCounts,
        reused_handle_slot_count: usize,
        appended_handle_slot_count: usize,
        dense_relocation_count: usize,
    ) -> Self {
        Self {
            input_upsert_count,
            input_removal_count,
            stable_key_lookup_count,
            primitive_comparison_count,
            dirty_domain_counts,
            reused_handle_slot_count,
            appended_handle_slot_count,
            dense_relocation_count,
            storage_stats: RenderSceneStorageStats::new(0, 0, 0),
        }
    }

    pub(super) const fn with_storage_stats(
        mut self,
        storage_stats: RenderSceneStorageStats,
    ) -> Self {
        self.storage_stats = storage_stats;
        self
    }

    pub(crate) const fn input_upsert_count(self) -> usize {
        self.input_upsert_count
    }

    pub(crate) const fn input_removal_count(self) -> usize {
        self.input_removal_count
    }

    pub(crate) const fn stable_key_lookup_count(self) -> usize {
        self.stable_key_lookup_count
    }

    pub(crate) const fn primitive_comparison_count(self) -> usize {
        self.primitive_comparison_count
    }

    pub(crate) const fn dirty_domain_counts(self) -> RenderSceneDirtyDomainCounts {
        self.dirty_domain_counts
    }

    pub(crate) const fn reused_handle_slot_count(self) -> usize {
        self.reused_handle_slot_count
    }

    pub(crate) const fn appended_handle_slot_count(self) -> usize {
        self.appended_handle_slot_count
    }

    pub(crate) const fn dense_relocation_count(self) -> usize {
        self.dense_relocation_count
    }

    pub(crate) const fn storage_stats(self) -> RenderSceneStorageStats {
        self.storage_stats
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct RenderScenePrimitiveDirtyFlags(u32);

impl RenderScenePrimitiveDirtyFlags {
    pub(crate) const NONE: Self = Self(0);
    pub(crate) const TRANSFORM: Self = Self(1 << 0);
    pub(crate) const GEOMETRY: Self = Self(1 << 1);
    pub(crate) const MATERIAL: Self = Self(1 << 2);
    pub(crate) const DEFORMATION: Self = Self(1 << 3);
    pub(crate) const RENDER_STATE: Self = Self(1 << 4);
    pub(crate) const VISIBILITY: Self = Self(1 << 5);
    pub(crate) const BOUNDS: Self = Self(1 << 6);
    /// Staging qualifier for local-space bounds stored in GPU primitive rows.
    /// `BOUNDS` remains the CPU world-envelope invalidation domain.
    pub(crate) const LOCAL_BOUNDS: Self = Self(1 << 7);
    pub(crate) const ALL: Self = Self((1 << 8) - 1);

    pub(crate) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for RenderScenePrimitiveDirtyFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RenderScenePrimitiveDirtyFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderSceneAddedPrimitive {
    handle: RenderScenePrimitiveHandle,
    dense_index: u32,
    primitive: Arc<RenderScenePrimitive>,
}

impl RenderSceneAddedPrimitive {
    pub(super) fn new(
        handle: RenderScenePrimitiveHandle,
        dense_index: u32,
        primitive: Arc<RenderScenePrimitive>,
    ) -> Self {
        Self {
            handle,
            dense_index,
            primitive,
        }
    }

    pub(crate) const fn handle(&self) -> RenderScenePrimitiveHandle {
        self.handle
    }

    pub(crate) const fn dense_index(&self) -> u32 {
        self.dense_index
    }

    pub(crate) fn primitive(&self) -> &Arc<RenderScenePrimitive> {
        &self.primitive
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderSceneUpdatedPrimitive {
    handle: RenderScenePrimitiveHandle,
    dense_index: u32,
    dirty: RenderScenePrimitiveDirtyFlags,
    previous_primitive: Arc<RenderScenePrimitive>,
    primitive: Arc<RenderScenePrimitive>,
}

impl RenderSceneUpdatedPrimitive {
    pub(super) fn new(
        handle: RenderScenePrimitiveHandle,
        dense_index: u32,
        dirty: RenderScenePrimitiveDirtyFlags,
        previous_primitive: Arc<RenderScenePrimitive>,
        primitive: Arc<RenderScenePrimitive>,
    ) -> Self {
        Self {
            handle,
            dense_index,
            dirty,
            previous_primitive,
            primitive,
        }
    }

    pub(crate) const fn handle(&self) -> RenderScenePrimitiveHandle {
        self.handle
    }

    pub(crate) const fn dense_index(&self) -> u32 {
        self.dense_index
    }

    pub(crate) const fn dirty(&self) -> RenderScenePrimitiveDirtyFlags {
        self.dirty
    }

    pub(crate) fn previous_primitive(&self) -> &Arc<RenderScenePrimitive> {
        &self.previous_primitive
    }

    pub(crate) fn primitive(&self) -> &Arc<RenderScenePrimitive> {
        &self.primitive
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderScenePrimitiveRelocation {
    handle: RenderScenePrimitiveHandle,
    from_dense_index: u32,
    to_dense_index: u32,
}

impl RenderScenePrimitiveRelocation {
    pub(super) const fn new(
        handle: RenderScenePrimitiveHandle,
        from_dense_index: u32,
        to_dense_index: u32,
    ) -> Self {
        Self {
            handle,
            from_dense_index,
            to_dense_index,
        }
    }

    pub(crate) const fn handle(self) -> RenderScenePrimitiveHandle {
        self.handle
    }

    pub(crate) const fn from_dense_index(self) -> u32 {
        self.from_dense_index
    }

    pub(crate) const fn to_dense_index(self) -> u32 {
        self.to_dense_index
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderSceneRemovedPrimitive {
    handle: RenderScenePrimitiveHandle,
    dense_index: u32,
    primitive: Arc<RenderScenePrimitive>,
    relocation: Option<RenderScenePrimitiveRelocation>,
}

impl RenderSceneRemovedPrimitive {
    pub(super) fn new(
        handle: RenderScenePrimitiveHandle,
        dense_index: u32,
        primitive: Arc<RenderScenePrimitive>,
        relocation: Option<RenderScenePrimitiveRelocation>,
    ) -> Self {
        Self {
            handle,
            dense_index,
            primitive,
            relocation,
        }
    }

    pub(crate) const fn handle(&self) -> RenderScenePrimitiveHandle {
        self.handle
    }

    pub(crate) const fn dense_index(&self) -> u32 {
        self.dense_index
    }

    pub(crate) fn primitive(&self) -> &Arc<RenderScenePrimitive> {
        &self.primitive
    }

    pub(crate) const fn relocation(&self) -> Option<RenderScenePrimitiveRelocation> {
        self.relocation
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderSceneChangeJournal {
    world: RenderWorldSnapshotHandle,
    from_generation: RenderSceneGeneration,
    to_generation: RenderSceneGeneration,
    removals: Arc<[RenderSceneRemovedPrimitive]>,
    updates: Arc<[RenderSceneUpdatedPrimitive]>,
    additions: Arc<[RenderSceneAddedPrimitive]>,
    resource_reference_deltas: Arc<[RenderSceneResourceReferenceDelta]>,
    resource_reference_stats: RenderSceneResourceReferenceDeltaStats,
    stats: RenderSceneApplyStats,
}

impl RenderSceneChangeJournal {
    pub(super) fn new(
        world: RenderWorldSnapshotHandle,
        from_generation: RenderSceneGeneration,
        to_generation: RenderSceneGeneration,
        removals: Vec<RenderSceneRemovedPrimitive>,
        updates: Vec<RenderSceneUpdatedPrimitive>,
        additions: Vec<RenderSceneAddedPrimitive>,
        stats: RenderSceneApplyStats,
    ) -> Self {
        let resource_reference_build =
            build_resource_reference_deltas(&removals, &updates, &additions);
        Self {
            world,
            from_generation,
            to_generation,
            removals: removals.into(),
            updates: updates.into(),
            additions: additions.into(),
            resource_reference_deltas: resource_reference_build.deltas.into(),
            resource_reference_stats: resource_reference_build.stats,
            stats,
        }
    }

    pub(super) fn empty(
        world: RenderWorldSnapshotHandle,
        generation: RenderSceneGeneration,
        stats: RenderSceneApplyStats,
    ) -> Self {
        Self::new(
            world,
            generation,
            generation,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            stats,
        )
    }

    pub(crate) const fn world(&self) -> RenderWorldSnapshotHandle {
        self.world
    }

    pub(crate) const fn from_generation(&self) -> RenderSceneGeneration {
        self.from_generation
    }

    pub(crate) const fn to_generation(&self) -> RenderSceneGeneration {
        self.to_generation
    }

    pub(crate) fn removals(&self) -> &[RenderSceneRemovedPrimitive] {
        &self.removals
    }

    pub(crate) fn updates(&self) -> &[RenderSceneUpdatedPrimitive] {
        &self.updates
    }

    pub(crate) fn additions(&self) -> &[RenderSceneAddedPrimitive] {
        &self.additions
    }

    pub(crate) fn resource_reference_deltas(&self) -> &[RenderSceneResourceReferenceDelta] {
        &self.resource_reference_deltas
    }

    pub(crate) const fn resource_reference_stats(&self) -> RenderSceneResourceReferenceDeltaStats {
        self.resource_reference_stats
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.removals.is_empty() && self.updates.is_empty() && self.additions.is_empty()
    }

    pub(crate) const fn stats(&self) -> RenderSceneApplyStats {
        self.stats
    }
}

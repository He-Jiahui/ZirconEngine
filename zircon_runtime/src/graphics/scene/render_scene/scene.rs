use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::core::framework::render::RenderWorldSnapshotHandle;
use crate::core::framework::scene::EntityId;

use super::change_journal::{
    RenderSceneAddedPrimitive, RenderSceneApplyStats, RenderSceneChangeJournal,
    RenderSceneDirtyDomainCounts, RenderScenePrimitiveDirtyFlags, RenderScenePrimitiveRelocation,
    RenderSceneRemovedPrimitive, RenderSceneUpdatedPrimitive,
};
use super::primitive::RenderScenePrimitive;

mod delta_transaction;

pub(crate) use delta_transaction::RenderSceneDeltaTransactionError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RenderSceneGeneration(u64);

impl RenderSceneGeneration {
    pub(crate) const INITIAL: Self = Self(0);

    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }

    const fn next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RenderScenePrimitiveHandle {
    slot: u32,
    slot_generation: u32,
}

impl RenderScenePrimitiveHandle {
    pub(crate) const fn slot(self) -> u32 {
        self.slot
    }

    pub(crate) const fn slot_generation(self) -> u32 {
        self.slot_generation
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderSceneStorageStats {
    live_primitive_count: usize,
    handle_slot_high_water: usize,
    reusable_handle_hole_count: usize,
}

impl RenderSceneStorageStats {
    pub(super) const fn new(
        live_primitive_count: usize,
        handle_slot_high_water: usize,
        reusable_handle_hole_count: usize,
    ) -> Self {
        Self {
            live_primitive_count,
            handle_slot_high_water,
            reusable_handle_hole_count,
        }
    }

    pub(crate) const fn live_primitive_count(self) -> usize {
        self.live_primitive_count
    }

    pub(crate) const fn handle_slot_high_water(self) -> usize {
        self.handle_slot_high_water
    }

    pub(crate) const fn reusable_handle_hole_count(self) -> usize {
        self.reusable_handle_hole_count
    }

    pub(crate) const fn generation_exhausted_handle_slot_count(self) -> usize {
        self.fragmented_handle_slot_count()
            .saturating_sub(self.reusable_handle_hole_count)
    }

    pub(crate) const fn fragmented_handle_slot_count(self) -> usize {
        self.handle_slot_high_water
            .saturating_sub(self.live_primitive_count)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RenderSceneDelta {
    upserts: Vec<RenderScenePrimitive>,
    removals: Vec<u64>,
}

impl RenderSceneDelta {
    pub(crate) fn new(upserts: Vec<RenderScenePrimitive>, removals: Vec<u64>) -> Self {
        Self { upserts, removals }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneApplyError {
    DuplicateUpsert {
        stable_instance_key: u64,
    },
    DuplicateRemoval {
        stable_instance_key: u64,
    },
    ConflictingMutation {
        stable_instance_key: u64,
    },
    StableKeyOwnerChanged {
        stable_instance_key: u64,
        previous_node_id: EntityId,
        incoming_node_id: EntityId,
    },
    InvalidLiveHandle {
        stable_instance_key: u64,
    },
    GenerationExhausted,
    PrimitiveHandleCapacityExhausted,
}

impl fmt::Display for RenderSceneApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateUpsert {
                stable_instance_key,
            } => write!(
                formatter,
                "render-scene delta contains duplicate upsert key {stable_instance_key}"
            ),
            Self::DuplicateRemoval {
                stable_instance_key,
            } => write!(
                formatter,
                "render-scene delta contains duplicate removal key {stable_instance_key}"
            ),
            Self::ConflictingMutation {
                stable_instance_key,
            } => write!(
                formatter,
                "render-scene delta both removes and upserts key {stable_instance_key}"
            ),
            Self::StableKeyOwnerChanged {
                stable_instance_key,
                previous_node_id,
                incoming_node_id,
            } => write!(
                formatter,
                "render-scene key {stable_instance_key} changed owner from entity {previous_node_id} to {incoming_node_id}"
            ),
            Self::InvalidLiveHandle {
                stable_instance_key,
            } => write!(
                formatter,
                "render-scene key map references an invalid live handle for key {stable_instance_key}"
            ),
            Self::GenerationExhausted => {
                formatter.write_str("render-scene generation space is exhausted")
            }
            Self::PrimitiveHandleCapacityExhausted => {
                formatter.write_str("render-scene primitive handle capacity is exhausted")
            }
        }
    }
}

impl Error for RenderSceneApplyError {}

#[derive(Clone, Copy, Debug)]
struct RenderSceneHandleSlot {
    slot_generation: u32,
    dense_index: Option<u32>,
}

pub(crate) struct RenderScene {
    world: RenderWorldSnapshotHandle,
    generation: RenderSceneGeneration,
    primitives: Vec<Arc<RenderScenePrimitive>>,
    dense_handles: Vec<RenderScenePrimitiveHandle>,
    handle_slots: Vec<RenderSceneHandleSlot>,
    free_handle_slots: BinaryHeap<Reverse<u32>>,
    stable_key_to_handle: HashMap<u64, RenderScenePrimitiveHandle>,
}

impl RenderScene {
    pub(crate) fn new(world: RenderWorldSnapshotHandle) -> Self {
        Self {
            world,
            generation: RenderSceneGeneration::INITIAL,
            primitives: Vec::new(),
            dense_handles: Vec::new(),
            handle_slots: Vec::new(),
            free_handle_slots: BinaryHeap::new(),
            stable_key_to_handle: HashMap::new(),
        }
    }

    pub(crate) fn read(&self) -> RenderSceneReadView<'_> {
        RenderSceneReadView {
            world: self.world,
            generation: self.generation,
            storage_stats: self.storage_stats(),
            primitives: &self.primitives,
            dense_handles: &self.dense_handles,
            handle_slots: &self.handle_slots,
            stable_key_to_handle: &self.stable_key_to_handle,
        }
    }

    fn storage_stats(&self) -> RenderSceneStorageStats {
        RenderSceneStorageStats::new(
            self.primitives.len(),
            self.handle_slots.len(),
            self.free_handle_slots.len(),
        )
    }

    pub(crate) fn apply_delta(
        &mut self,
        delta: RenderSceneDelta,
    ) -> Result<RenderSceneChangeJournal, RenderSceneApplyError> {
        delta_transaction::apply(self, delta)
    }

    pub(crate) fn apply_delta_with_staging<StageOutput, StageError>(
        &mut self,
        delta: RenderSceneDelta,
        stage: impl FnOnce(
            &[super::resource_dependencies::RenderSceneResourceReferenceDelta],
        ) -> Result<StageOutput, StageError>,
    ) -> Result<(RenderSceneChangeJournal, StageOutput), RenderSceneDeltaTransactionError<StageError>>
    {
        delta_transaction::apply_with_staging(self, delta, stage)
    }

    pub(super) fn resource_release_deltas(
        &self,
    ) -> Vec<super::resource_dependencies::RenderSceneResourceReferenceDelta> {
        super::resource_dependencies::build_resource_reference_deltas(
            self.primitives.iter().map(Arc::as_ref),
            std::iter::empty(),
            std::iter::empty(),
        )
        .deltas
    }

    fn primitive_for_handle(
        &self,
        handle: RenderScenePrimitiveHandle,
    ) -> Option<&RenderScenePrimitive> {
        let slot = self.handle_slots.get(handle.slot as usize)?;
        if slot.slot_generation != handle.slot_generation {
            return None;
        }
        let dense_index = slot.dense_index? as usize;
        if self.dense_handles.get(dense_index).copied() != Some(handle) {
            return None;
        }
        self.primitives.get(dense_index).map(Arc::as_ref)
    }

    fn plan_addition_handles(
        &self,
        removals: &[(u64, RenderScenePrimitiveHandle)],
        addition_count: usize,
    ) -> Result<Vec<RenderScenePrimitiveHandle>, RenderSceneApplyError> {
        let mut available_slots = self.free_handle_slots.clone();
        let removed_slots = removals
            .iter()
            .filter_map(|(_, handle)| (handle.slot_generation < u32::MAX).then_some(handle.slot))
            .collect::<HashSet<_>>();
        for slot in &removed_slots {
            available_slots.push(Reverse(*slot));
        }

        let appended_count = addition_count.saturating_sub(available_slots.len());
        let final_slot_count = self
            .handle_slots
            .len()
            .checked_add(appended_count)
            .ok_or(RenderSceneApplyError::PrimitiveHandleCapacityExhausted)?;
        if final_slot_count as u64 > u64::from(u32::MAX) + 1 {
            return Err(RenderSceneApplyError::PrimitiveHandleCapacityExhausted);
        }

        let mut handles = Vec::with_capacity(addition_count);
        let mut appended_slot = self.handle_slots.len();
        for _ in 0..addition_count {
            if let Some(Reverse(slot)) = available_slots.pop() {
                let current_generation = self
                    .handle_slots
                    .get(slot as usize)
                    .ok_or(RenderSceneApplyError::PrimitiveHandleCapacityExhausted)?
                    .slot_generation;
                let slot_generation = if removed_slots.contains(&slot) {
                    current_generation
                        .checked_add(1)
                        .ok_or(RenderSceneApplyError::PrimitiveHandleCapacityExhausted)?
                } else {
                    current_generation
                };
                handles.push(RenderScenePrimitiveHandle {
                    slot,
                    slot_generation,
                });
            } else {
                handles.push(RenderScenePrimitiveHandle {
                    slot: appended_slot as u32,
                    slot_generation: 1,
                });
                appended_slot += 1;
            }
        }
        Ok(handles)
    }

    fn remove_primitive(
        &mut self,
        handle: RenderScenePrimitiveHandle,
    ) -> Option<RenderSceneRemovedPrimitive> {
        let slot_index = handle.slot as usize;
        let dense_index = self.handle_slots.get(slot_index)?.dense_index?;
        if self.dense_handles.get(dense_index as usize).copied() != Some(handle) {
            return None;
        }
        let last_dense_index = u32::try_from(self.primitives.len().checked_sub(1)?).ok()?;
        let primitive = self.primitives.swap_remove(dense_index as usize);
        self.dense_handles.swap_remove(dense_index as usize);
        self.stable_key_to_handle
            .remove(&primitive.stable_instance_key());

        let relocation = if dense_index != last_dense_index {
            let moved_handle = self.dense_handles[dense_index as usize];
            self.handle_slots
                .get_mut(moved_handle.slot as usize)?
                .dense_index = Some(dense_index);
            Some(RenderScenePrimitiveRelocation::new(
                moved_handle,
                last_dense_index,
                dense_index,
            ))
        } else {
            None
        };

        let slot = self.handle_slots.get_mut(slot_index)?;
        slot.dense_index = None;
        if let Some(next_slot_generation) = slot.slot_generation.checked_add(1) {
            slot.slot_generation = next_slot_generation;
            self.free_handle_slots.push(Reverse(handle.slot));
        }

        Some(RenderSceneRemovedPrimitive::new(
            handle,
            dense_index,
            primitive,
            relocation,
        ))
    }

    fn install_update(
        &mut self,
        handle: RenderScenePrimitiveHandle,
        dirty: RenderScenePrimitiveDirtyFlags,
        primitive: RenderScenePrimitive,
    ) -> Option<RenderSceneUpdatedPrimitive> {
        let dense_index = self.handle_slots.get(handle.slot as usize)?.dense_index?;
        let destination = self.primitives.get_mut(dense_index as usize)?;
        let previous_primitive = Arc::clone(destination);
        let primitive = Arc::new(primitive);
        *destination = Arc::clone(&primitive);
        Some(RenderSceneUpdatedPrimitive::new(
            handle,
            dense_index,
            dirty,
            previous_primitive,
            primitive,
        ))
    }

    fn install_addition(
        &mut self,
        handle: RenderScenePrimitiveHandle,
        primitive: RenderScenePrimitive,
    ) -> RenderSceneAddedPrimitive {
        let dense_index = self.primitives.len() as u32;
        if handle.slot as usize == self.handle_slots.len() {
            self.handle_slots.push(RenderSceneHandleSlot {
                slot_generation: handle.slot_generation,
                dense_index: Some(dense_index),
            });
        } else {
            let _reused_slot = self.free_handle_slots.pop();
            self.handle_slots[handle.slot as usize].dense_index = Some(dense_index);
        }

        let stable_instance_key = primitive.stable_instance_key();
        let primitive = Arc::new(primitive);
        self.primitives.push(Arc::clone(&primitive));
        self.dense_handles.push(handle);
        self.stable_key_to_handle
            .insert(stable_instance_key, handle);
        RenderSceneAddedPrimitive::new(handle, dense_index, primitive)
    }
}

pub(crate) struct RenderSceneReadView<'scene> {
    world: RenderWorldSnapshotHandle,
    generation: RenderSceneGeneration,
    storage_stats: RenderSceneStorageStats,
    primitives: &'scene [Arc<RenderScenePrimitive>],
    dense_handles: &'scene [RenderScenePrimitiveHandle],
    handle_slots: &'scene [RenderSceneHandleSlot],
    stable_key_to_handle: &'scene HashMap<u64, RenderScenePrimitiveHandle>,
}

impl<'scene> RenderSceneReadView<'scene> {
    pub(crate) const fn world(&self) -> RenderWorldSnapshotHandle {
        self.world
    }

    pub(crate) const fn generation(&self) -> RenderSceneGeneration {
        self.generation
    }

    pub(crate) const fn storage_stats(&self) -> RenderSceneStorageStats {
        self.storage_stats
    }

    pub(crate) const fn len(&self) -> usize {
        self.primitives.len()
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub(crate) fn get(&self, handle: RenderScenePrimitiveHandle) -> Option<&RenderScenePrimitive> {
        let dense_index = self.dense_index(handle)? as usize;
        self.primitives.get(dense_index).map(Arc::as_ref)
    }

    pub(crate) fn dense_index(&self, handle: RenderScenePrimitiveHandle) -> Option<u32> {
        let slot = self.handle_slots.get(handle.slot as usize)?;
        if slot.slot_generation != handle.slot_generation {
            return None;
        }
        let dense_index = slot.dense_index?;
        (self.dense_handles.get(dense_index as usize).copied() == Some(handle))
            .then_some(dense_index)
    }

    pub(crate) fn handle_for_stable_key(
        &self,
        stable_instance_key: u64,
    ) -> Option<RenderScenePrimitiveHandle> {
        self.stable_key_to_handle.get(&stable_instance_key).copied()
    }

    pub(crate) fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (RenderScenePrimitiveHandle, &RenderScenePrimitive)> + '_
    {
        self.dense_handles
            .iter()
            .copied()
            .zip(self.primitives.iter().map(Arc::as_ref))
    }
}

fn validate_delta_keys(delta: &RenderSceneDelta) -> Result<(), RenderSceneApplyError> {
    if let Some(stable_instance_key) = adjacent_duplicate(
        delta
            .upserts
            .iter()
            .map(RenderScenePrimitive::stable_instance_key),
    ) {
        return Err(RenderSceneApplyError::DuplicateUpsert {
            stable_instance_key,
        });
    }
    if let Some(stable_instance_key) = adjacent_duplicate(delta.removals.iter().copied()) {
        return Err(RenderSceneApplyError::DuplicateRemoval {
            stable_instance_key,
        });
    }
    if let Some(stable_instance_key) = delta
        .upserts
        .iter()
        .map(RenderScenePrimitive::stable_instance_key)
        .find(|stable_instance_key| delta.removals.binary_search(stable_instance_key).is_ok())
    {
        return Err(RenderSceneApplyError::ConflictingMutation {
            stable_instance_key,
        });
    }
    Ok(())
}

fn adjacent_duplicate(values: impl IntoIterator<Item = u64>) -> Option<u64> {
    let mut previous = None;
    for value in values {
        if previous == Some(value) {
            return Some(value);
        }
        previous = Some(value);
    }
    None
}

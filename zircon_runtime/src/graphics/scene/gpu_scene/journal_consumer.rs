use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::core::framework::render::RenderWorldSnapshotHandle;

use super::super::render_scene::{
    RenderSceneChangeJournal, RenderSceneGeneration, RenderSceneJournalCommit,
    RenderSceneJournalCursor, RenderSceneJournalCursorError, RenderSceneJournalPreflight,
    RenderScenePrimitiveHandle,
};

mod reprojection;
mod work;

pub(crate) use reprojection::{
    GpuSceneJournalReprojectionError, GpuSceneJournalReprojectionPlan,
    GpuSceneJournalReprojectionPreflightError,
};
use work::GpuSceneJournalWorkSet;
pub(crate) use work::{
    GpuSceneJournalResidentWrite, GpuSceneJournalResidentWriteKind, GpuSceneJournalRetirement,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GpuSceneJournalSlot {
    slot_generation: u32,
    stable_instance_key: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuSceneJournalSlotMutation {
    slot: u32,
    slot_generation: u32,
    stable_instance_key: Option<u64>,
}

impl GpuSceneJournalSlotMutation {
    pub(crate) const fn slot(self) -> u32 {
        self.slot
    }

    pub(crate) const fn slot_generation(self) -> u32 {
        self.slot_generation
    }

    pub(crate) const fn stable_instance_key(self) -> Option<u64> {
        self.stable_instance_key
    }

    const fn into_slot(self) -> GpuSceneJournalSlot {
        GpuSceneJournalSlot {
            slot_generation: self.slot_generation,
            stable_instance_key: self.stable_instance_key,
        }
    }
}

#[derive(Debug)]
pub(crate) struct GpuSceneJournalApplyPlan<'journal> {
    cursor_preflight: RenderSceneJournalPreflight,
    journal: &'journal RenderSceneChangeJournal,
    slot_mutations: Vec<GpuSceneJournalSlotMutation>,
    projected_slot_high_water: usize,
    projected_resident_count: usize,
    direct_slot_validation_count: usize,
    work_set: GpuSceneJournalWorkSet<'journal>,
}

impl<'journal> GpuSceneJournalApplyPlan<'journal> {
    pub(crate) const fn requires_apply(&self) -> bool {
        self.cursor_preflight.requires_apply()
    }

    pub(crate) const fn journal(&self) -> &'journal RenderSceneChangeJournal {
        self.journal
    }

    pub(crate) fn slot_mutations(&self) -> &[GpuSceneJournalSlotMutation] {
        &self.slot_mutations
    }

    pub(crate) const fn projected_slot_high_water(&self) -> usize {
        self.projected_slot_high_water
    }

    pub(crate) const fn projected_resident_count(&self) -> usize {
        self.projected_resident_count
    }

    pub(crate) const fn direct_slot_validation_count(&self) -> usize {
        self.direct_slot_validation_count
    }

    pub(crate) const fn stable_key_lookup_count(&self) -> usize {
        0
    }

    pub(crate) fn resident_writes(&self) -> &[GpuSceneJournalResidentWrite<'journal>] {
        self.work_set.resident_writes()
    }

    pub(crate) fn retirements(&self) -> &[GpuSceneJournalRetirement<'journal>] {
        self.work_set.retirements()
    }

    pub(crate) const fn full_resident_write_count(&self) -> usize {
        self.work_set.full_resident_write_count()
    }

    pub(crate) const fn dirty_resident_write_count(&self) -> usize {
        self.work_set.dirty_resident_write_count()
    }

    pub(crate) const fn instance_transform_write_count(&self) -> usize {
        self.work_set.instance_transform_write_count()
    }

    pub(crate) const fn local_bounds_write_count(&self) -> usize {
        self.work_set.local_bounds_write_count()
    }

    pub(crate) fn retirement_count(&self) -> usize {
        self.work_set.retirements().len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuSceneJournalConsumerError {
    Cursor(RenderSceneJournalCursorError),
    SlotOutOfRange {
        handle: RenderScenePrimitiveHandle,
        slot_high_water: usize,
    },
    SlotGenerationMismatch {
        handle: RenderScenePrimitiveHandle,
        resident_slot_generation: u32,
    },
    VacantSlot {
        handle: RenderScenePrimitiveHandle,
    },
    OccupiedSlot {
        handle: RenderScenePrimitiveHandle,
        resident_stable_key: u64,
    },
    StableKeyMismatch {
        handle: RenderScenePrimitiveHandle,
        resident_stable_key: u64,
        journal_stable_key: u64,
    },
    NonContiguousSlotAppend {
        handle: RenderScenePrimitiveHandle,
        expected_slot: u32,
    },
    AppendedSlotGenerationMismatch {
        handle: RenderScenePrimitiveHandle,
    },
    InvalidPlan,
}

impl From<RenderSceneJournalCursorError> for GpuSceneJournalConsumerError {
    fn from(error: RenderSceneJournalCursorError) -> Self {
        Self::Cursor(error)
    }
}

impl fmt::Display for GpuSceneJournalConsumerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cursor(error) => fmt::Display::fmt(error, formatter),
            Self::SlotOutOfRange {
                handle,
                slot_high_water,
            } => write!(
                formatter,
                "GPUScene journal handle {}:{} exceeds slot high-water {}",
                handle.slot(),
                handle.slot_generation(),
                slot_high_water
            ),
            Self::SlotGenerationMismatch {
                handle,
                resident_slot_generation,
            } => write!(
                formatter,
                "GPUScene journal handle {}:{} does not match resident generation {}",
                handle.slot(),
                handle.slot_generation(),
                resident_slot_generation
            ),
            Self::VacantSlot { handle } => write!(
                formatter,
                "GPUScene journal handle {}:{} references a vacant slot",
                handle.slot(),
                handle.slot_generation()
            ),
            Self::OccupiedSlot {
                handle,
                resident_stable_key,
            } => write!(
                formatter,
                "GPUScene journal addition {}:{} aliases resident key {}",
                handle.slot(),
                handle.slot_generation(),
                resident_stable_key
            ),
            Self::StableKeyMismatch {
                handle,
                resident_stable_key,
                journal_stable_key,
            } => write!(
                formatter,
                "GPUScene journal handle {}:{} maps resident key {} to journal key {}",
                handle.slot(),
                handle.slot_generation(),
                resident_stable_key,
                journal_stable_key
            ),
            Self::NonContiguousSlotAppend {
                handle,
                expected_slot,
            } => write!(
                formatter,
                "GPUScene journal appends slot {} while next persistent slot is {}",
                handle.slot(),
                expected_slot
            ),
            Self::AppendedSlotGenerationMismatch { handle } => write!(
                formatter,
                "GPUScene journal appends slot {} with non-initial generation {}",
                handle.slot(),
                handle.slot_generation()
            ),
            Self::InvalidPlan => formatter.write_str("GPUScene journal apply plan is malformed"),
        }
    }
}

impl Error for GpuSceneJournalConsumerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cursor(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GpuSceneJournalTransactionError<StageError> {
    Preflight(GpuSceneJournalConsumerError),
    Staging(StageError),
    Commit(GpuSceneJournalConsumerError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuSceneJournalTransactionCommit<StageOutput> {
    Applied(StageOutput),
    Replayed,
}

impl<StageError> fmt::Display for GpuSceneJournalTransactionError<StageError>
where
    StageError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preflight(error) => {
                write!(formatter, "GPUScene journal preflight failed: {error}")
            }
            Self::Staging(error) => write!(formatter, "GPUScene journal staging failed: {error}"),
            Self::Commit(error) => write!(formatter, "GPUScene journal commit failed: {error}"),
        }
    }
}

impl<StageError> Error for GpuSceneJournalTransactionError<StageError>
where
    StageError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Preflight(error) | Self::Commit(error) => Some(error),
            Self::Staging(error) => Some(error),
        }
    }
}

pub(crate) struct GpuSceneJournalConsumer {
    cursor: RenderSceneJournalCursor,
    slots: Vec<GpuSceneJournalSlot>,
    resident_count: usize,
}

impl GpuSceneJournalConsumer {
    pub(crate) fn new(world: RenderWorldSnapshotHandle) -> Self {
        Self {
            cursor: RenderSceneJournalCursor::at(world, RenderSceneGeneration::INITIAL),
            slots: Vec::new(),
            resident_count: 0,
        }
    }

    pub(crate) const fn applied_generation(&self) -> RenderSceneGeneration {
        self.cursor.applied_generation()
    }

    pub(crate) fn slot_high_water(&self) -> usize {
        self.slots.len()
    }

    pub(crate) const fn resident_count(&self) -> usize {
        self.resident_count
    }

    pub(crate) fn resident_stable_key(&self, handle: RenderScenePrimitiveHandle) -> Option<u64> {
        let slot = self.slots.get(handle.slot() as usize)?;
        (slot.slot_generation == handle.slot_generation())
            .then_some(slot.stable_instance_key)
            .flatten()
    }

    /// Stages every journal-owned GPU mutation before publishing the matching
    /// residency and cursor generation. The staging callback must leave its
    /// external owner unchanged when it returns an error.
    pub(crate) fn apply_with_staging<'journal, StageOutput, StageError>(
        &mut self,
        journal: &'journal RenderSceneChangeJournal,
        stage: impl FnOnce(&GpuSceneJournalApplyPlan<'journal>) -> Result<StageOutput, StageError>,
    ) -> Result<
        GpuSceneJournalTransactionCommit<StageOutput>,
        GpuSceneJournalTransactionError<StageError>,
    > {
        let plan = self
            .preflight(journal)
            .map_err(GpuSceneJournalTransactionError::Preflight)?;
        let staged = plan
            .requires_apply()
            .then(|| stage(&plan).map_err(GpuSceneJournalTransactionError::Staging))
            .transpose()?;
        let commit = self
            .commit_preflighted(plan)
            .map_err(GpuSceneJournalTransactionError::Commit)?;
        match (commit, staged) {
            (RenderSceneJournalCommit::Applied, Some(output)) => {
                Ok(GpuSceneJournalTransactionCommit::Applied(output))
            }
            (RenderSceneJournalCommit::Replayed, None) => {
                Ok(GpuSceneJournalTransactionCommit::Replayed)
            }
            _ => Err(GpuSceneJournalTransactionError::Commit(
                GpuSceneJournalConsumerError::InvalidPlan,
            )),
        }
    }

    fn preflight<'journal>(
        &self,
        journal: &'journal RenderSceneChangeJournal,
    ) -> Result<GpuSceneJournalApplyPlan<'journal>, GpuSceneJournalConsumerError> {
        let cursor_preflight = self.cursor.preflight(journal)?;
        if !cursor_preflight.requires_apply() {
            return Ok(GpuSceneJournalApplyPlan {
                cursor_preflight,
                journal,
                slot_mutations: Vec::new(),
                projected_slot_high_water: self.slots.len(),
                projected_resident_count: self.resident_count,
                direct_slot_validation_count: 0,
                work_set: GpuSceneJournalWorkSet::empty(),
            });
        }

        let mut projected = BTreeMap::<u32, GpuSceneJournalSlot>::new();
        let mut projected_slot_high_water = self.slots.len();
        let mut projected_resident_count = self.resident_count;
        let mut direct_slot_validation_count = 0usize;

        for removal in journal.removals() {
            let handle = removal.handle();
            let stable_instance_key = removal.primitive().stable_instance_key();
            let slot = self.require_live_slot(&projected, handle, stable_instance_key)?;
            direct_slot_validation_count = direct_slot_validation_count
                .checked_add(1)
                .ok_or(GpuSceneJournalConsumerError::InvalidPlan)?;
            projected_resident_count = projected_resident_count
                .checked_sub(1)
                .ok_or(GpuSceneJournalConsumerError::InvalidPlan)?;
            projected.insert(
                handle.slot(),
                GpuSceneJournalSlot {
                    slot_generation: slot.slot_generation.saturating_add(1),
                    stable_instance_key: None,
                },
            );
        }

        for update in journal.updates() {
            self.require_live_slot(
                &projected,
                update.handle(),
                update.primitive().stable_instance_key(),
            )?;
            direct_slot_validation_count = direct_slot_validation_count
                .checked_add(1)
                .ok_or(GpuSceneJournalConsumerError::InvalidPlan)?;
        }

        for addition in journal.additions() {
            let handle = addition.handle();
            let slot_index = handle.slot() as usize;
            if slot_index > projected_slot_high_water {
                return Err(GpuSceneJournalConsumerError::NonContiguousSlotAppend {
                    handle,
                    expected_slot: projected_slot_high_water as u32,
                });
            }
            if slot_index == projected_slot_high_water {
                if handle.slot_generation() != 1 {
                    return Err(
                        GpuSceneJournalConsumerError::AppendedSlotGenerationMismatch { handle },
                    );
                }
                projected_slot_high_water = projected_slot_high_water.saturating_add(1);
            } else {
                let slot = self.require_slot(&projected, handle)?;
                direct_slot_validation_count = direct_slot_validation_count
                    .checked_add(1)
                    .ok_or(GpuSceneJournalConsumerError::InvalidPlan)?;
                if slot.slot_generation != handle.slot_generation() {
                    return Err(GpuSceneJournalConsumerError::SlotGenerationMismatch {
                        handle,
                        resident_slot_generation: slot.slot_generation,
                    });
                }
                if let Some(resident_stable_key) = slot.stable_instance_key {
                    return Err(GpuSceneJournalConsumerError::OccupiedSlot {
                        handle,
                        resident_stable_key,
                    });
                }
            }
            projected_resident_count = projected_resident_count
                .checked_add(1)
                .ok_or(GpuSceneJournalConsumerError::InvalidPlan)?;
            projected.insert(
                handle.slot(),
                GpuSceneJournalSlot {
                    slot_generation: handle.slot_generation(),
                    stable_instance_key: Some(addition.primitive().stable_instance_key()),
                },
            );
        }

        let slot_mutations = projected
            .into_iter()
            .map(|(slot, state)| GpuSceneJournalSlotMutation {
                slot,
                slot_generation: state.slot_generation,
                stable_instance_key: state.stable_instance_key,
            })
            .collect();
        Ok(GpuSceneJournalApplyPlan {
            cursor_preflight,
            journal,
            slot_mutations,
            projected_slot_high_water,
            projected_resident_count,
            direct_slot_validation_count,
            work_set: GpuSceneJournalWorkSet::compile(journal),
        })
    }

    fn commit_preflighted(
        &mut self,
        plan: GpuSceneJournalApplyPlan<'_>,
    ) -> Result<RenderSceneJournalCommit, GpuSceneJournalConsumerError> {
        if plan.projected_slot_high_water < self.slots.len()
            || plan.projected_resident_count > plan.projected_slot_high_water
            || plan
                .slot_mutations
                .iter()
                .any(|mutation| mutation.slot as usize >= plan.projected_slot_high_water)
        {
            return Err(GpuSceneJournalConsumerError::InvalidPlan);
        }
        let commit = self.cursor.commit(plan.cursor_preflight)?;
        if commit == RenderSceneJournalCommit::Replayed {
            if !plan.slot_mutations.is_empty()
                || plan.projected_slot_high_water != self.slots.len()
                || plan.projected_resident_count != self.resident_count
                || plan.direct_slot_validation_count != 0
                || !plan.work_set.is_empty()
            {
                return Err(GpuSceneJournalConsumerError::InvalidPlan);
            }
            return Ok(commit);
        }

        self.slots.resize(
            plan.projected_slot_high_water,
            GpuSceneJournalSlot::default(),
        );
        for mutation in plan.slot_mutations {
            self.slots[mutation.slot as usize] = mutation.into_slot();
        }
        self.resident_count = plan.projected_resident_count;
        Ok(commit)
    }

    fn require_live_slot(
        &self,
        projected: &BTreeMap<u32, GpuSceneJournalSlot>,
        handle: RenderScenePrimitiveHandle,
        journal_stable_key: u64,
    ) -> Result<GpuSceneJournalSlot, GpuSceneJournalConsumerError> {
        let slot = self.require_slot(projected, handle)?;
        if slot.slot_generation != handle.slot_generation() {
            return Err(GpuSceneJournalConsumerError::SlotGenerationMismatch {
                handle,
                resident_slot_generation: slot.slot_generation,
            });
        }
        let Some(resident_stable_key) = slot.stable_instance_key else {
            return Err(GpuSceneJournalConsumerError::VacantSlot { handle });
        };
        if resident_stable_key != journal_stable_key {
            return Err(GpuSceneJournalConsumerError::StableKeyMismatch {
                handle,
                resident_stable_key,
                journal_stable_key,
            });
        }
        Ok(slot)
    }

    fn require_slot(
        &self,
        projected: &BTreeMap<u32, GpuSceneJournalSlot>,
        handle: RenderScenePrimitiveHandle,
    ) -> Result<GpuSceneJournalSlot, GpuSceneJournalConsumerError> {
        projected
            .get(&handle.slot())
            .copied()
            .or_else(|| self.slots.get(handle.slot() as usize).copied())
            .ok_or(GpuSceneJournalConsumerError::SlotOutOfRange {
                handle,
                slot_high_water: self.slots.len(),
            })
    }
}

#[cfg(test)]
mod tests;

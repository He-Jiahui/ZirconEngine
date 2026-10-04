use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use crate::core::editing::engine::{EditCommandError, HistoryContextId};

use super::{EditorTransactionEngine, EngineState};

const HISTORY_DIRTY_JOURNAL_CAPACITY: usize = 4_096;

#[derive(Clone)]
pub struct HistoryDirtyCursor {
    lineage: Arc<()>,
    generation: u64,
    pending_history: Option<HistoryContextId>,
}

impl HistoryDirtyCursor {
    fn new(lineage: Arc<()>, generation: u64, pending_history: Option<HistoryContextId>) -> Self {
        Self {
            lineage,
            generation,
            pending_history,
        }
    }

    fn belongs_to(&self, lineage: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.lineage, lineage)
    }
}

impl fmt::Debug for HistoryDirtyCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HistoryDirtyCursor")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryDirtyBatchKind {
    Unchanged,
    Delta,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryDirtyState {
    history: HistoryContextId,
    history_generation: u64,
    dirty: bool,
}

impl HistoryDirtyState {
    pub const fn history(self) -> HistoryContextId {
        self.history
    }

    pub const fn history_generation(self) -> u64 {
        self.history_generation
    }

    pub const fn is_dirty(self) -> bool {
        self.dirty
    }
}

#[derive(Clone, Debug)]
pub struct HistoryDirtyBatch {
    cursor: HistoryDirtyCursor,
    kind: HistoryDirtyBatchKind,
    states: Vec<HistoryDirtyState>,
}

impl HistoryDirtyBatch {
    pub const fn cursor(&self) -> &HistoryDirtyCursor {
        &self.cursor
    }

    pub const fn kind(&self) -> HistoryDirtyBatchKind {
        self.kind
    }

    pub fn states(&self) -> &[HistoryDirtyState] {
        &self.states
    }
}

#[derive(Clone, Copy)]
pub(super) struct HistoryDirtyChangeReservation {
    generation: u64,
}

#[derive(Clone, Copy)]
pub(super) struct HistoryMutationReservation {
    history_generation: u64,
    dirty: Option<HistoryDirtyChangeReservation>,
}

pub(super) struct HistoryDirtyJournal {
    generation: u64,
    changes: VecDeque<(u64, HistoryContextId)>,
    #[cfg(test)]
    journal_visits: usize,
}

impl Default for HistoryDirtyJournal {
    fn default() -> Self {
        Self {
            generation: 0,
            changes: VecDeque::with_capacity(HISTORY_DIRTY_JOURNAL_CAPACITY),
            #[cfg(test)]
            journal_visits: 0,
        }
    }
}

impl HistoryDirtyJournal {
    fn reserve_dirty_change(&self) -> Result<HistoryDirtyChangeReservation, EditCommandError> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(EditCommandError::HistoryDirtyGenerationExhausted)?;
        Ok(HistoryDirtyChangeReservation { generation })
    }

    fn preflight_changes(&self, count: u64) -> Result<(), EditCommandError> {
        self.generation
            .checked_add(count)
            .ok_or(EditCommandError::HistoryDirtyGenerationExhausted)
            .map(|_| ())
    }

    fn record_dirty_change(
        &mut self,
        history: HistoryContextId,
        reservation: HistoryDirtyChangeReservation,
    ) {
        debug_assert_eq!(reservation.generation, self.generation + 1);
        self.generation = reservation.generation;
        if self.changes.len() == HISTORY_DIRTY_JOURNAL_CAPACITY {
            self.changes.pop_front();
        }
        self.changes.push_back((self.generation, history));
    }

    fn can_replay_from(&self, generation: u64) -> bool {
        if generation >= self.generation {
            return generation == self.generation;
        }
        self.changes
            .front()
            .is_some_and(|(oldest, _)| generation >= oldest.saturating_sub(1))
    }

    fn change_start_after(&self, generation: u64) -> usize {
        self.changes.front().map_or(0, |(oldest, _)| {
            generation
                .saturating_add(1)
                .saturating_sub(*oldest)
                .try_into()
                .unwrap_or(self.changes.len())
        })
    }

    fn changed_histories_after(&mut self, generation: u64) -> Vec<HistoryContextId> {
        let start = self.change_start_after(generation).min(self.changes.len());
        #[cfg(test)]
        let (changes, journal_visits) = (&self.changes, &mut self.journal_visits);
        #[cfg(not(test))]
        let changes = &self.changes;
        let remaining = changes.len().saturating_sub(start);
        let mut changed = HashSet::with_capacity(remaining);
        for (_, history) in changes.range(start..) {
            #[cfg(test)]
            {
                *journal_visits += 1;
            }
            changed.insert(*history);
        }
        let mut changed = changed.into_iter().collect::<Vec<_>>();
        changed.sort_unstable();
        changed
    }
}

impl EditorTransactionEngine {
    pub fn dirty_states_since(
        &self,
        cursor: Option<&HistoryDirtyCursor>,
    ) -> Result<HistoryDirtyBatch, EditCommandError> {
        if cursor.is_some_and(|cursor| !cursor.belongs_to(&self.save_token_lineage)) {
            return Err(EditCommandError::HistoryDirtyCursorEngineMismatch);
        }
        self.start_observation("query dirty state batch")?;
        let mut state = self.lock_state();
        let current_generation = state.history_dirty.generation;
        let pending_history = Self::pending_dirty_history(&state);
        let (mut kind, mut changed) = match cursor {
            Some(cursor) if state.history_dirty.can_replay_from(cursor.generation) => (
                HistoryDirtyBatchKind::Delta,
                state
                    .history_dirty
                    .changed_histories_after(cursor.generation),
            ),
            _ => (
                HistoryDirtyBatchKind::Reset,
                state
                    .history_generations
                    .keys()
                    .copied()
                    .collect::<Vec<_>>(),
            ),
        };
        // Pending changes have no committed generation yet. Compare the previous
        // observation as well, so cancelling the first interaction emits clean.
        if kind == HistoryDirtyBatchKind::Reset {
            changed.extend(pending_history);
            if let Some(cursor) = cursor {
                changed.extend(cursor.pending_history);
            }
        } else if let Some(cursor) = cursor {
            if cursor.pending_history != pending_history {
                changed.extend(cursor.pending_history);
                changed.extend(pending_history);
            }
        }
        changed.retain(|history| !history.is_volatile());
        changed.sort_unstable();
        changed.dedup();
        if kind == HistoryDirtyBatchKind::Delta && changed.is_empty() {
            kind = HistoryDirtyBatchKind::Unchanged;
        }
        let states = changed
            .into_iter()
            .map(|history| HistoryDirtyState {
                history,
                history_generation: Self::history_generation(&state, history),
                dirty: Self::observed_history_status(&state, history).dirty,
            })
            .collect();
        let batch = HistoryDirtyBatch {
            cursor: HistoryDirtyCursor::new(
                Arc::clone(&self.save_token_lineage),
                current_generation,
                pending_history,
            ),
            kind,
            states,
        };
        self.clear_operation_locked(&mut state);
        Ok(batch)
    }

    pub(super) fn reserve_dirty_change(
        state: &EngineState,
    ) -> Result<HistoryDirtyChangeReservation, EditCommandError> {
        state.history_dirty.reserve_dirty_change()
    }

    /// Budgets the known group commit and history clear under one held admission.
    /// No counter is advanced; the reservation prevents competing mutation until both finish.
    pub(super) fn preflight_history_clear(
        state: &EngineState,
        target: HistoryContextId,
        pending_commit: Option<HistoryContextId>,
    ) -> Result<(), EditCommandError> {
        let dirty_count = u64::from(!target.is_volatile())
            + u64::from(pending_commit.is_some_and(|history| !history.is_volatile()));
        state.history_dirty.preflight_changes(dirty_count)?;
        let target_count = 1 + u64::from(pending_commit == Some(target));
        Self::history_generation(state, target)
            .checked_add(target_count)
            .ok_or(EditCommandError::HistoryGenerationExhausted { history: target })?;
        if let Some(history) = pending_commit.filter(|history| *history != target) {
            Self::history_generation(state, history)
                .checked_add(1)
                .ok_or(EditCommandError::HistoryGenerationExhausted { history })?;
        }
        Ok(())
    }

    pub(super) fn reserve_history_mutation(
        state: &EngineState,
        history: HistoryContextId,
    ) -> Result<HistoryMutationReservation, EditCommandError> {
        let dirty = if !history.is_volatile() {
            Some(Self::reserve_dirty_change(state)?)
        } else {
            None
        };
        Ok(HistoryMutationReservation {
            history_generation: Self::next_history_generation(state, history)?,
            dirty,
        })
    }

    pub(super) fn record_dirty_change(
        state: &mut EngineState,
        history: HistoryContextId,
        reservation: HistoryDirtyChangeReservation,
    ) {
        state
            .history_dirty
            .record_dirty_change(history, reservation);
    }

    pub(super) fn record_history_mutation(
        state: &mut EngineState,
        history: HistoryContextId,
        reservation: HistoryMutationReservation,
    ) {
        state
            .history_generations
            .insert(history, reservation.history_generation);
        if let Some(dirty) = reservation.dirty {
            Self::record_dirty_change(state, history, dirty);
        }
    }

    #[cfg(test)]
    pub(crate) fn take_dirty_journal_visits_for_test(&self) -> usize {
        let mut state = self.lock_state();
        std::mem::take(&mut state.history_dirty.journal_visits)
    }

    #[cfg(test)]
    pub(crate) fn dirty_generation_for_test(&self) -> u64 {
        self.lock_state().history_dirty.generation
    }

    #[cfg(test)]
    pub(crate) fn set_dirty_generation_for_test(&self, generation: u64) {
        self.lock_state().history_dirty.generation = generation;
    }
}

#[cfg(test)]
#[path = "tests/dirty_batch_optimization_tests.rs"]
mod optimization_tests;

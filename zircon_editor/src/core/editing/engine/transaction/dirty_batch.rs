use std::collections::{BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use crate::core::editing::engine::{EditCommandError, HistoryContextId, HistoryStore};

use super::{EditorTransactionEngine, EngineState};

const HISTORY_DIRTY_JOURNAL_CAPACITY: usize = 4_096;

#[derive(Clone)]
pub struct HistoryDirtyCursor {
    lineage: Arc<()>,
    generation: u64,
}

impl HistoryDirtyCursor {
    fn new(lineage: Arc<()>, generation: u64) -> Self {
        Self {
            lineage,
            generation,
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
    dirty: HistoryDirtyChangeReservation,
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

    fn changed_histories_after(&mut self, generation: u64) -> BTreeSet<HistoryContextId> {
        let start = self.change_start_after(generation).min(self.changes.len());
        #[cfg(test)]
        let (changes, journal_visits) = (&self.changes, &mut self.journal_visits);
        #[cfg(not(test))]
        let changes = &self.changes;
        changes
            .range(start..)
            .map(|(_, history)| {
                #[cfg(test)]
                {
                    *journal_visits += 1;
                }
                *history
            })
            .collect()
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
        self.flush_operation_group()?;
        self.start_operation("query dirty state batch")?;
        let mut state = self.lock_state();
        let current_generation = state.history_dirty.generation;
        if cursor.is_some_and(|cursor| cursor.generation == current_generation) {
            let batch = HistoryDirtyBatch {
                cursor: HistoryDirtyCursor::new(
                    Arc::clone(&self.save_token_lineage),
                    current_generation,
                ),
                kind: HistoryDirtyBatchKind::Unchanged,
                states: Vec::new(),
            };
            self.clear_operation_locked(&mut state);
            return Ok(batch);
        }
        let (kind, changed) = match cursor {
            None => (
                HistoryDirtyBatchKind::Reset,
                state
                    .history_generations
                    .keys()
                    .copied()
                    .collect::<BTreeSet<_>>(),
            ),
            Some(cursor) if state.history_dirty.can_replay_from(cursor.generation) => {
                let changed = state
                    .history_dirty
                    .changed_histories_after(cursor.generation);
                let kind = if changed.is_empty() {
                    HistoryDirtyBatchKind::Unchanged
                } else {
                    HistoryDirtyBatchKind::Delta
                };
                (kind, changed)
            }
            Some(_) => (
                HistoryDirtyBatchKind::Reset,
                state
                    .history_generations
                    .keys()
                    .copied()
                    .collect::<BTreeSet<_>>(),
            ),
        };
        let states = changed
            .into_iter()
            .map(|history| HistoryDirtyState {
                history,
                history_generation: Self::history_generation(&state, history),
                dirty: state
                    .histories
                    .get(&history)
                    .is_some_and(HistoryStore::is_dirty),
            })
            .collect();
        let batch = HistoryDirtyBatch {
            cursor: HistoryDirtyCursor::new(
                Arc::clone(&self.save_token_lineage),
                current_generation,
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

    pub(super) fn reserve_history_mutation(
        state: &EngineState,
        history: HistoryContextId,
    ) -> Result<HistoryMutationReservation, EditCommandError> {
        Ok(HistoryMutationReservation {
            history_generation: Self::next_history_generation(state, history)?,
            dirty: Self::reserve_dirty_change(state)?,
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
        Self::record_dirty_change(state, history, reservation.dirty);
    }

    #[cfg(test)]
    pub(crate) fn take_dirty_journal_visits_for_test(&self) -> usize {
        let mut state = self.lock_state();
        std::mem::take(&mut state.history_dirty.journal_visits)
    }

    #[cfg(test)]
    pub(crate) fn set_dirty_generation_for_test(&self, generation: u64) {
        self.lock_state().history_dirty.generation = generation;
    }
}

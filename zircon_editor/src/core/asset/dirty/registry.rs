use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editing::engine::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, HistoryDirtyBatch,
    HistoryDirtyBatchKind, HistoryDirtyCursor, HistorySaveToken,
};
use crate::core::editor_message::DocumentId;

use super::{DirtyExternalEffectId, DirtyRegistryError};

const MAX_SNAPSHOT_ATTEMPTS: usize = 8;
const DIRTY_CHANGE_JOURNAL_CAPACITY: usize = 4_096;

pub(super) trait DirtyTransactionStateSource: Send + Sync {
    fn is_dirty(&self, document: DocumentId) -> Result<bool, EditCommandError>;

    fn dirty_states_since(
        &self,
        cursor: Option<&HistoryDirtyCursor>,
    ) -> Result<HistoryDirtyBatch, EditCommandError>;

    fn capture_save_token(
        &self,
        document: DocumentId,
    ) -> Result<HistorySaveToken, EditCommandError>;

    fn mark_saved_if_unchanged(
        &self,
        document: DocumentId,
        token: HistorySaveToken,
    ) -> Result<(), EditCommandError>;
}

impl DirtyTransactionStateSource for EditorTransactionEngine {
    fn is_dirty(&self, document: DocumentId) -> Result<bool, EditCommandError> {
        self.is_dirty(HistoryContextId::Document(document))
    }

    fn dirty_states_since(
        &self,
        cursor: Option<&HistoryDirtyCursor>,
    ) -> Result<HistoryDirtyBatch, EditCommandError> {
        self.dirty_states_since(cursor)
    }

    fn capture_save_token(
        &self,
        document: DocumentId,
    ) -> Result<HistorySaveToken, EditCommandError> {
        EditorTransactionEngine::capture_save_token(self, HistoryContextId::Document(document))
    }

    fn mark_saved_if_unchanged(
        &self,
        document: DocumentId,
        token: HistorySaveToken,
    ) -> Result<(), EditCommandError> {
        EditorTransactionEngine::mark_saved_if_unchanged(
            self,
            HistoryContextId::Document(document),
            token,
        )
        .map(drop)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirtyExternalEffectRevision(u64);

impl DirtyExternalEffectRevision {
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirtyDocumentSnapshot {
    document: DocumentId,
    generation: u64,
    transaction_dirty: bool,
    external_effects: Vec<DirtyExternalEffectId>,
    external_revisions: Vec<DirtyExternalEffectRevision>,
}

impl DirtyDocumentSnapshot {
    pub fn document(&self) -> DocumentId {
        self.document
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn transaction_dirty(&self) -> bool {
        self.transaction_dirty
    }

    pub fn external_effects(&self) -> &[DirtyExternalEffectId] {
        &self.external_effects
    }

    pub fn external_revision(
        &self,
        effect: &DirtyExternalEffectId,
    ) -> Option<DirtyExternalEffectRevision> {
        self.external_effects
            .binary_search(effect)
            .ok()
            .and_then(|index| self.external_revisions.get(index))
            .copied()
    }

    pub fn is_dirty(&self) -> bool {
        self.transaction_dirty || !self.external_effects.is_empty()
    }
}

#[derive(Clone)]
pub struct DirtyRegistryCursor {
    lineage: Arc<()>,
    registry_generation: u64,
    transaction: HistoryDirtyCursor,
}

impl fmt::Debug for DirtyRegistryCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirtyRegistryCursor")
            .field("registry_generation", &self.registry_generation)
            .field("transaction", &self.transaction)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct DirtyRegistryDelta {
    cursor: DirtyRegistryCursor,
    reset: bool,
    snapshots: Vec<DirtyDocumentSnapshot>,
    removed_documents: Vec<DocumentId>,
}

impl DirtyRegistryDelta {
    pub const fn cursor(&self) -> &DirtyRegistryCursor {
        &self.cursor
    }

    pub const fn is_reset(&self) -> bool {
        self.reset
    }

    pub fn snapshots(&self) -> &[DirtyDocumentSnapshot] {
        &self.snapshots
    }

    pub fn removed_documents(&self) -> &[DocumentId] {
        &self.removed_documents
    }
}

#[derive(Default)]
struct DirtyRegistryState {
    documents: BTreeSet<DocumentId>,
    document_generations: BTreeMap<DocumentId, u64>,
    external_effects:
        BTreeMap<DocumentId, BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>>,
    changes: VecDeque<(u64, DocumentId)>,
    #[cfg(test)]
    journal_visits: usize,
    next_external_revision: u64,
    next_document_generation: u64,
    registry_generation: u64,
}

impl DirtyRegistryState {
    fn can_replay_from(&self, generation: u64) -> bool {
        if generation >= self.registry_generation {
            return generation == self.registry_generation;
        }
        self.changes
            .front()
            .is_some_and(|(oldest, _)| generation >= oldest.saturating_sub(1))
    }

    fn record_change(&mut self, generation: u64, document: DocumentId) {
        self.next_document_generation = generation;
        self.registry_generation = generation;
        self.document_generations.insert(document, generation);
        if self.changes.len() == DIRTY_CHANGE_JOURNAL_CAPACITY {
            self.changes.pop_front();
        }
        self.changes.push_back((generation, document));
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

    fn changed_documents_after(&mut self, generation: u64) -> BTreeSet<DocumentId> {
        let start = self.change_start_after(generation).min(self.changes.len());
        #[cfg(test)]
        let (changes, journal_visits) = (&self.changes, &mut self.journal_visits);
        #[cfg(not(test))]
        let changes = &self.changes;
        changes
            .range(start..)
            .map(|(_, document)| {
                #[cfg(test)]
                {
                    *journal_visits += 1;
                }
                *document
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct DirtyRegistry {
    transactions: Arc<dyn DirtyTransactionStateSource>,
    state: Arc<Mutex<DirtyRegistryState>>,
    cursor_lineage: Arc<()>,
}

impl DirtyRegistry {
    pub fn new(transactions: Arc<EditorTransactionEngine>) -> Self {
        Self::from_transaction_source(transactions)
    }

    #[cfg(test)]
    pub(super) fn with_transaction_source(
        transactions: Arc<dyn DirtyTransactionStateSource>,
    ) -> Self {
        Self::from_transaction_source(transactions)
    }

    fn from_transaction_source(transactions: Arc<dyn DirtyTransactionStateSource>) -> Self {
        Self {
            transactions,
            state: Arc::new(Mutex::new(DirtyRegistryState::default())),
            cursor_lineage: Arc::new(()),
        }
    }

    pub fn register_document(&self, document: DocumentId) -> Result<bool, DirtyRegistryError> {
        let mut state = self.lock_state();
        if state.documents.contains(&document) {
            return Ok(false);
        }
        let generation = Self::next_document_generation(&state)?;
        state.documents.insert(document);
        state.record_change(generation, document);
        Ok(true)
    }

    pub fn unregister_document(&self, document: DocumentId) -> Result<bool, DirtyRegistryError> {
        let mut state = self.lock_state();
        if !state.documents.contains(&document) {
            return Ok(false);
        }
        let generation = Self::next_document_generation(&state)?;
        state.documents.remove(&document);
        state.external_effects.remove(&document);
        state.record_change(generation, document);
        state.document_generations.remove(&document);
        Ok(true)
    }

    pub fn mark_external_effect(
        &self,
        document: DocumentId,
        effect: DirtyExternalEffectId,
    ) -> Result<DirtyExternalEffectRevision, DirtyRegistryError> {
        let mut state = self.lock_state();
        Self::require_document(&state, document)?;
        let revision = state
            .next_external_revision
            .checked_add(1)
            .map(DirtyExternalEffectRevision)
            .ok_or(DirtyRegistryError::ExternalEffectRevisionExhausted)?;
        let generation = Self::next_document_generation(&state)?;
        state.next_external_revision = revision.value();
        state
            .external_effects
            .entry(document)
            .or_default()
            .insert(effect, revision);
        state.record_change(generation, document);
        Ok(revision)
    }

    pub fn clear_external_effect(
        &self,
        document: DocumentId,
        effect: &DirtyExternalEffectId,
        expected_revision: DirtyExternalEffectRevision,
    ) -> Result<bool, DirtyRegistryError> {
        let mut state = self.lock_state();
        Self::require_document(&state, document)?;
        let matches_revision = state
            .external_effects
            .get(&document)
            .and_then(|effects| effects.get(effect))
            .is_some_and(|revision| *revision == expected_revision);
        if !matches_revision {
            return Ok(false);
        }
        let generation = Self::next_document_generation(&state)?;
        let removed = state
            .external_effects
            .get_mut(&document)
            .is_some_and(|effects| effects.remove(effect).is_some());
        if state
            .external_effects
            .get(&document)
            .is_some_and(BTreeMap::is_empty)
        {
            state.external_effects.remove(&document);
        }
        state.record_change(generation, document);
        Ok(removed)
    }

    pub fn snapshot(
        &self,
        document: DocumentId,
    ) -> Result<DirtyDocumentSnapshot, DirtyRegistryError> {
        for _ in 0..MAX_SNAPSHOT_ATTEMPTS {
            let (generation, effects) = {
                let state = self.lock_state();
                Self::require_document(&state, document)?;
                (
                    state.document_generations[&document],
                    state
                        .external_effects
                        .get(&document)
                        .cloned()
                        .unwrap_or_default(),
                )
            };
            let snapshot = self.snapshot_with_effects(document, generation, effects)?;
            let state = self.lock_state();
            Self::require_document(&state, document)?;
            if state.document_generations.get(&document) == Some(&generation) {
                return Ok(snapshot);
            }
        }
        Err(DirtyRegistryError::SnapshotUnstable {
            document,
            attempts: MAX_SNAPSHOT_ATTEMPTS,
        })
    }

    pub fn changes_since(
        &self,
        cursor: Option<&DirtyRegistryCursor>,
    ) -> Result<DirtyRegistryDelta, DirtyRegistryError> {
        if cursor.is_some_and(|cursor| !Arc::ptr_eq(&cursor.lineage, &self.cursor_lineage)) {
            return Err(DirtyRegistryError::CursorRegistryMismatch);
        }
        for _ in 0..MAX_SNAPSHOT_ATTEMPTS {
            let (registry_generation, external_reset, external_changed, external_present) = {
                let mut state = self.lock_state();
                let external_reset =
                    cursor.is_none_or(|cursor| !state.can_replay_from(cursor.registry_generation));
                let external_changed = if external_reset {
                    state.documents.iter().copied().collect()
                } else if cursor.expect("cursor checked above").registry_generation
                    == state.registry_generation
                {
                    BTreeSet::new()
                } else {
                    state.changed_documents_after(
                        cursor.expect("cursor checked above").registry_generation,
                    )
                };
                let external_present = external_changed
                    .iter()
                    .filter(|document| state.documents.contains(document))
                    .copied()
                    .collect::<BTreeSet<_>>();
                (
                    state.registry_generation,
                    external_reset,
                    external_changed,
                    external_present,
                )
            };

            let (transaction_batch, external_transaction_states) = if external_reset {
                (self.transactions.dirty_states_since(None)?, BTreeMap::new())
            } else {
                let fresh = external_present
                    .iter()
                    .copied()
                    .map(|document| {
                        self.transactions
                            .is_dirty(document)
                            .map(|dirty| (HistoryContextId::Document(document), dirty))
                    })
                    .collect::<Result<BTreeMap<_, _>, _>>()?;
                (
                    self.transactions.dirty_states_since(Some(
                        &cursor.expect("cursor checked above").transaction,
                    ))?,
                    fresh,
                )
            };

            let transaction_reset = transaction_batch.kind() == HistoryDirtyBatchKind::Reset;
            let external_change_documents = external_changed.clone();
            let mut changed_documents = if external_reset || transaction_reset {
                let state = self.lock_state();
                if state.registry_generation != registry_generation {
                    continue;
                }
                state.documents.clone()
            } else {
                external_changed
            };
            let mut transaction_states = external_transaction_states;
            for transaction_state in transaction_batch.states() {
                if let HistoryContextId::Document(document) = transaction_state.history() {
                    changed_documents.insert(document);
                    transaction_states.insert(
                        HistoryContextId::Document(document),
                        transaction_state.is_dirty(),
                    );
                }
            }

            let mut state = self.lock_state();
            if state.registry_generation != registry_generation {
                continue;
            }
            let mut snapshots = Vec::with_capacity(changed_documents.len());
            let mut removed_documents = Vec::new();
            for document in changed_documents {
                if !state.documents.contains(&document) {
                    if external_change_documents.contains(&document) {
                        removed_documents.push(document);
                    }
                    continue;
                }
                let history = HistoryContextId::Document(document);
                let transaction_dirty = transaction_states.get(&history).copied().unwrap_or(false);
                let external_revisions = state
                    .external_effects
                    .get(&document)
                    .cloned()
                    .unwrap_or_default();
                let document_generation = state.document_generations[&document];
                snapshots.push(Self::snapshot_from_parts(
                    document,
                    document_generation,
                    transaction_dirty,
                    external_revisions,
                ));
            }
            let delta = DirtyRegistryDelta {
                cursor: DirtyRegistryCursor {
                    lineage: Arc::clone(&self.cursor_lineage),
                    registry_generation,
                    transaction: transaction_batch.cursor().clone(),
                },
                reset: external_reset || transaction_reset,
                snapshots,
                removed_documents,
            };
            drop(state);
            return Ok(delta);
        }
        Err(DirtyRegistryError::DeltaUnstable {
            attempts: MAX_SNAPSHOT_ATTEMPTS,
        })
    }

    fn snapshot_with_effects(
        &self,
        document: DocumentId,
        generation: u64,
        external_revisions: BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>,
    ) -> Result<DirtyDocumentSnapshot, DirtyRegistryError> {
        let transaction_dirty = self.transactions.is_dirty(document)?;
        Ok(Self::snapshot_from_parts(
            document,
            generation,
            transaction_dirty,
            external_revisions,
        ))
    }

    fn snapshot_from_parts(
        document: DocumentId,
        generation: u64,
        transaction_dirty: bool,
        external_revisions: BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>,
    ) -> DirtyDocumentSnapshot {
        let (external_effects, external_revisions) = external_revisions.into_iter().unzip();
        DirtyDocumentSnapshot {
            document,
            generation,
            transaction_dirty,
            external_effects,
            external_revisions,
        }
    }

    pub fn is_generation_current(
        &self,
        document: DocumentId,
        generation: u64,
    ) -> Result<bool, DirtyRegistryError> {
        let state = self.lock_state();
        Self::require_document(&state, document)?;
        Ok(state.document_generations.get(&document) == Some(&generation))
    }

    pub fn capture_save_token(
        &self,
        document: DocumentId,
    ) -> Result<HistorySaveToken, DirtyRegistryError> {
        let state = self.lock_state();
        Self::require_document(&state, document)?;
        drop(state);
        Ok(self.transactions.capture_save_token(document)?)
    }

    pub fn mark_saved_if_unchanged(
        &self,
        document: DocumentId,
        token: HistorySaveToken,
    ) -> Result<(), DirtyRegistryError> {
        let state = self.lock_state();
        Self::require_document(&state, document)?;
        drop(state);
        Ok(self.transactions.mark_saved_if_unchanged(document, token)?)
    }

    pub fn clear_saved_external_effects(
        &self,
        snapshot: &DirtyDocumentSnapshot,
    ) -> Result<bool, DirtyRegistryError> {
        if !self.is_generation_current(snapshot.document(), snapshot.generation())? {
            return Ok(false);
        }
        let mut unchanged = true;
        for effect in snapshot.external_effects() {
            let Some(revision) = snapshot.external_revision(effect) else {
                unchanged = false;
                continue;
            };
            if !self.clear_external_effect(snapshot.document(), effect, revision)? {
                unchanged = false;
            }
        }
        let state = self.lock_state();
        Self::require_document(&state, snapshot.document())?;
        let has_residual_effects = state
            .external_effects
            .get(&snapshot.document())
            .is_some_and(|effects| !effects.is_empty());
        Ok(unchanged && !has_residual_effects)
    }

    fn require_document(
        state: &DirtyRegistryState,
        document: DocumentId,
    ) -> Result<(), DirtyRegistryError> {
        if state.documents.contains(&document) {
            Ok(())
        } else {
            Err(DirtyRegistryError::DocumentNotRegistered { document })
        }
    }

    fn next_document_generation(state: &DirtyRegistryState) -> Result<u64, DirtyRegistryError> {
        state
            .next_document_generation
            .checked_add(1)
            .ok_or(DirtyRegistryError::DocumentGenerationExhausted)
    }

    fn lock_state(&self) -> MutexGuard<'_, DirtyRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn take_journal_visits_for_test(&self) -> usize {
        let mut state = self.lock_state();
        std::mem::take(&mut state.journal_visits)
    }
}

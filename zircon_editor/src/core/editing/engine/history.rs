use std::collections::{BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::editor_message::DocumentId;
use crate::core::play::{PlayInstanceId, WorldDomain};

use super::{
    CommandBox, CommandEffect, CommandExecutionError, EditCommandError, EditContext,
    EditWorldRoute, SelectionSnapshot, TransactionJournal, TransactionJournalError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HistoryContextId {
    Global,
    Document(DocumentId),
    PlaySession(PlayInstanceId),
}

impl HistoryContextId {
    pub const fn is_volatile(self) -> bool {
        matches!(self, Self::PlaySession(_))
    }

    pub const fn world_domain(self) -> WorldDomain {
        match self {
            Self::Global | Self::Document(_) => WorldDomain::Edit,
            Self::PlaySession(instance) => WorldDomain::Play(instance),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransactionId(u64);

impl TransactionId {
    pub(crate) const fn from_sequence(sequence: u64) -> Self {
        Self(sequence)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct HistorySaveToken {
    lineage: Arc<()>,
    history: HistoryContextId,
    transaction: Option<TransactionId>,
    generation: u64,
}

impl HistorySaveToken {
    pub(crate) fn new(
        lineage: Arc<()>,
        history: HistoryContextId,
        transaction: Option<TransactionId>,
        generation: u64,
    ) -> Self {
        Self {
            lineage,
            history,
            transaction,
            generation,
        }
    }

    pub(crate) fn belongs_to(&self, lineage: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.lineage, lineage)
    }

    pub(crate) const fn history(&self) -> HistoryContextId {
        self.history
    }

    pub(crate) const fn transaction(&self) -> Option<TransactionId> {
        self.transaction
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }
}

impl fmt::Debug for HistorySaveToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HistorySaveToken")
            .field("history", &self.history)
            .field("transaction", &self.transaction)
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistorySaveMarkOutcome {
    Marked,
    AlreadyMarked,
}

pub struct TransactionRecord {
    pub id: TransactionId,
    pub label: String,
    pub timestamp_frame: u64,
    pub(crate) route: EditWorldRoute,
    pub commands: Vec<CommandBox>,
    pub participants: BTreeSet<DocumentId>,
    pub selection_before: SelectionSnapshot,
    pub selection_after: SelectionSnapshot,
    pub significant: bool,
}

impl TransactionRecord {
    pub(crate) fn route(&self) -> &EditWorldRoute {
        &self.route
    }

    pub(crate) fn journal(
        &self,
        history: HistoryContextId,
    ) -> Result<TransactionJournal, TransactionJournalError> {
        TransactionJournal::from_record(history, self)
    }

    pub(crate) fn detail(&self) -> HistoryRecordDetail {
        HistoryRecordDetail {
            id: self.id,
            label: self.label.clone(),
            timestamp_frame: self.timestamp_frame,
            command_count: self.commands.len(),
            participants: self.participants.clone(),
            selection_before: self.selection_before.clone(),
            selection_after: self.selection_after.clone(),
            significant: self.significant,
        }
    }

    fn undo(&mut self, context: &mut dyn EditContext) -> Result<(), EditCommandError> {
        let original_selection = context.selection_snapshot();
        for index in (0..self.commands.len()).rev() {
            let result = self.commands[index].revert(context);
            if let Err(command_error) = result {
                return self.recover_failed_undo(context, index, original_selection, command_error);
            }
        }
        if let Err(selection_error) = context.restore_selection(&self.selection_before) {
            return self.recover_all_applied(context, original_selection, selection_error);
        }
        Ok(())
    }

    fn redo(&mut self, context: &mut dyn EditContext) -> Result<(), EditCommandError> {
        let original_selection = context.selection_snapshot();
        for index in 0..self.commands.len() {
            let result = self.commands[index].apply(context);
            if let Err(command_error) = result {
                return self.recover_failed_redo(context, index, original_selection, command_error);
            }
        }
        if let Err(selection_error) = context.restore_selection(&self.selection_after) {
            return self.recover_all_reverted(context, original_selection, selection_error);
        }
        Ok(())
    }

    fn recover_failed_undo(
        &mut self,
        context: &mut dyn EditContext,
        failed_index: usize,
        original_selection: SelectionSnapshot,
        command_error: CommandExecutionError,
    ) -> Result<(), EditCommandError> {
        let rollback_start = match command_error.effect {
            CommandEffect::Unchanged => failed_index + 1,
            CommandEffect::Applied => failed_index,
        };
        let command_error = command_error.source;
        for rollback_index in rollback_start..self.commands.len() {
            if let Err(rollback_error) = self.commands[rollback_index].apply(context) {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.source),
                });
            }
        }
        match context.restore_selection(&original_selection) {
            Ok(()) => Err(command_error),
            Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                command_error: Box::new(command_error),
                rollback_error: Box::new(rollback_error),
            }),
        }
    }

    fn recover_failed_redo(
        &mut self,
        context: &mut dyn EditContext,
        failed_index: usize,
        original_selection: SelectionSnapshot,
        command_error: CommandExecutionError,
    ) -> Result<(), EditCommandError> {
        let rollback_end = match command_error.effect {
            CommandEffect::Unchanged => failed_index,
            CommandEffect::Applied => failed_index + 1,
        };
        let command_error = command_error.source;
        for rollback_index in (0..rollback_end).rev() {
            if let Err(rollback_error) = self.commands[rollback_index].revert(context) {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.source),
                });
            }
        }
        match context.restore_selection(&original_selection) {
            Ok(()) => Err(command_error),
            Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                command_error: Box::new(command_error),
                rollback_error: Box::new(rollback_error),
            }),
        }
    }

    fn recover_all_applied(
        &mut self,
        context: &mut dyn EditContext,
        original_selection: SelectionSnapshot,
        command_error: EditCommandError,
    ) -> Result<(), EditCommandError> {
        for command in &mut self.commands {
            if let Err(rollback_error) = command.apply(context) {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.source),
                });
            }
        }
        match context.restore_selection(&original_selection) {
            Ok(()) => Err(command_error),
            Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                command_error: Box::new(command_error),
                rollback_error: Box::new(rollback_error),
            }),
        }
    }

    fn recover_all_reverted(
        &mut self,
        context: &mut dyn EditContext,
        original_selection: SelectionSnapshot,
        command_error: EditCommandError,
    ) -> Result<(), EditCommandError> {
        for command in self.commands.iter_mut().rev() {
            if let Err(rollback_error) = command.revert(context) {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.source),
                });
            }
        }
        match context.restore_selection(&original_selection) {
            Ok(()) => Err(command_error),
            Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                command_error: Box::new(command_error),
                rollback_error: Box::new(rollback_error),
            }),
        }
    }

    pub(crate) fn finalize(&mut self, context: &mut dyn EditContext) {
        for command in &mut self.commands {
            command.finalize(context);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryRecordDetail {
    pub id: TransactionId,
    pub label: String,
    pub timestamp_frame: u64,
    pub command_count: usize,
    pub participants: BTreeSet<DocumentId>,
    pub selection_before: SelectionSnapshot,
    pub selection_after: SelectionSnapshot,
    pub significant: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryStatus {
    pub len: usize,
    pub top: Option<TransactionId>,
    pub saved_top: Option<TransactionId>,
    pub saved_top_reachable: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    pub generation: u64,
}

impl HistoryStatus {
    pub(crate) fn empty(generation: u64) -> Self {
        Self {
            len: 0,
            top: None,
            saved_top: None,
            saved_top_reachable: true,
            can_undo: false,
            can_redo: false,
            dirty: false,
            generation,
        }
    }

    pub(crate) fn for_context(mut self, history: HistoryContextId) -> Self {
        if history.is_volatile() {
            self.saved_top = None;
            self.saved_top_reachable = true;
            self.dirty = false;
        }
        self
    }
}

#[derive(Clone)]
pub struct HistoryPageCursor {
    lineage: Arc<()>,
    history: HistoryContextId,
    generation: u64,
    offset: usize,
}

impl HistoryPageCursor {
    pub(crate) fn new(
        lineage: Arc<()>,
        history: HistoryContextId,
        generation: u64,
        offset: usize,
    ) -> Self {
        Self {
            lineage,
            history,
            generation,
            offset,
        }
    }

    pub(crate) fn belongs_to(&self, lineage: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.lineage, lineage)
    }

    pub const fn history(&self) -> HistoryContextId {
        self.history
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) const fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Debug for HistoryPageCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HistoryPageCursor")
            .field("history", &self.history)
            .field("generation", &self.generation)
            .field("offset", &self.offset)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct HistoryDetailPage {
    status: HistoryStatus,
    records: Vec<HistoryRecordDetail>,
    next_cursor: Option<HistoryPageCursor>,
}

impl HistoryDetailPage {
    pub(crate) fn new(
        status: HistoryStatus,
        records: Vec<HistoryRecordDetail>,
        next_cursor: Option<HistoryPageCursor>,
    ) -> Self {
        Self {
            status,
            records,
            next_cursor,
        }
    }

    pub const fn status(&self) -> HistoryStatus {
        self.status
    }

    pub fn records(&self) -> &[HistoryRecordDetail] {
        &self.records
    }

    pub fn into_records(self) -> Vec<HistoryRecordDetail> {
        self.records
    }

    pub fn next_cursor(&self) -> Option<&HistoryPageCursor> {
        self.next_cursor.as_ref()
    }
}

pub struct HistoryStore {
    entries: VecDeque<TransactionRecord>,
    top: Option<usize>,
    saved_top: Option<usize>,
    saved_top_reachable: bool,
    capacity: usize,
}

impl HistoryStore {
    pub fn new(capacity: usize) -> Result<Self, EditCommandError> {
        if capacity == 0 {
            return Err(EditCommandError::InvalidHistoryCapacity);
        }
        Ok(Self::from_validated_capacity(capacity))
    }

    pub(crate) fn from_validated_capacity(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            top: None,
            saved_top: None,
            saved_top_reachable: true,
            capacity,
        }
    }

    pub(crate) fn push(&mut self, record: TransactionRecord) -> Vec<TransactionRecord> {
        let retained = self.top.map_or(0, |top| top + 1);
        if self.saved_top_reachable && self.saved_top.is_some_and(|saved| saved >= retained) {
            self.saved_top_reachable = false;
        }
        let mut removed = self.entries.drain(retained..).collect::<Vec<_>>();
        self.entries.push_back(record);
        self.top = Some(self.entries.len() - 1);

        while self.entries.len() > self.capacity {
            if let Some(record) = self.entries.pop_front() {
                removed.push(record);
            }
            self.top = self.top.and_then(|top| top.checked_sub(1));
            if self.saved_top_reachable {
                match self.saved_top {
                    Some(saved) => self.saved_top = saved.checked_sub(1),
                    None => self.saved_top_reachable = false,
                }
            }
        }
        removed
    }

    pub(crate) fn undo(
        &mut self,
        context: &mut dyn EditContext,
    ) -> Result<Option<(TransactionId, String)>, EditCommandError> {
        let Some(top) = self.top else {
            return Ok(None);
        };
        let Some(record) = self.entries.get_mut(top) else {
            return Err(EditCommandError::InvariantViolation {
                invariant: "history top points at an existing record",
            });
        };
        record.undo(context)?;
        let event_metadata = (record.id, record.label.clone());
        self.top = top.checked_sub(1);
        Ok(Some(event_metadata))
    }

    pub(crate) fn redo(
        &mut self,
        context: &mut dyn EditContext,
    ) -> Result<Option<(TransactionId, String)>, EditCommandError> {
        let next = self.top.map_or(0, |top| top + 1);
        let Some(record) = self.entries.get_mut(next) else {
            return Ok(None);
        };
        record.redo(context)?;
        let event_metadata = (record.id, record.label.clone());
        self.top = Some(next);
        Ok(Some(event_metadata))
    }

    pub(crate) fn journal(
        &self,
        history: HistoryContextId,
        transaction: TransactionId,
    ) -> Result<TransactionJournal, TransactionJournalError> {
        let Some(record) = transaction_record_by_id(&self.entries, transaction) else {
            return Err(TransactionJournalError::TransactionNotFound {
                history,
                transaction,
            });
        };
        record.journal(history)
    }

    pub(crate) fn mark_saved_current(&mut self) {
        self.saved_top = self.top;
        self.saved_top_reachable = true;
    }

    pub(crate) fn current_transaction(&self) -> Option<TransactionId> {
        self.top
            .and_then(|top| self.entries.get(top))
            .map(|record| record.id)
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.top.is_some()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.top
            .map_or(!self.entries.is_empty(), |top| top + 1 < self.entries.len())
    }

    pub(crate) fn replay_route(&self, undo: bool) -> Option<&EditWorldRoute> {
        let index = if undo {
            self.top?
        } else {
            self.top.map_or(0, |top| top + 1)
        };
        self.entries.get(index).map(TransactionRecord::route)
    }

    pub(crate) fn world_route(&self) -> Result<Option<&EditWorldRoute>, EditCommandError> {
        let Some(route) = self.entries.front().map(TransactionRecord::route) else {
            return Ok(None);
        };
        if self.entries.iter().any(|record| record.route() != route) {
            return Err(EditCommandError::InvariantViolation {
                invariant: "one history store must not span multiple edit world routes",
            });
        }
        Ok(Some(route))
    }

    pub fn is_dirty(&self) -> bool {
        !self.saved_top_reachable || self.top != self.saved_top
    }

    pub(crate) fn status(&self, generation: u64) -> HistoryStatus {
        HistoryStatus {
            len: self.entries.len(),
            top: self.top.and_then(|top| self.record_identity_at(top)),
            saved_top: self
                .saved_top
                .filter(|_| self.saved_top_reachable)
                .and_then(|saved_top| self.record_identity_at(saved_top)),
            saved_top_reachable: self.saved_top_reachable,
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
            dirty: self.is_dirty(),
            generation,
        }
    }

    pub(crate) fn detail_window(
        &self,
        offset: usize,
        page_size: usize,
    ) -> (Vec<HistoryRecordDetail>, bool) {
        let end = offset.saturating_add(page_size).min(self.entries.len());
        let records = self
            .entries
            .range(offset.min(self.entries.len())..end)
            .map(TransactionRecord::detail)
            .collect();
        (records, end < self.entries.len())
    }

    fn record_identity_at(&self, index: usize) -> Option<TransactionId> {
        self.entries.get(index).map(|record| record.id)
    }

    pub(crate) fn clear(&mut self) -> Vec<TransactionRecord> {
        self.top = None;
        self.saved_top = None;
        self.saved_top_reachable = true;
        self.entries.drain(..).collect()
    }
}

fn transaction_record_by_id(
    entries: &VecDeque<TransactionRecord>,
    transaction: TransactionId,
) -> Option<&TransactionRecord> {
    let (front, back) = entries.as_slices();
    transaction_record_by_id_in_slice(front, transaction)
        .or_else(|| transaction_record_by_id_in_slice(back, transaction))
}

fn transaction_record_by_id_in_slice(
    entries: &[TransactionRecord],
    transaction: TransactionId,
) -> Option<&TransactionRecord> {
    entries
        .binary_search_by_key(&transaction, |record| record.id)
        .ok()
        .map(|index| &entries[index])
}

#[cfg(test)]
mod performance_source_guards {
    use std::collections::VecDeque;
    use std::time::{Duration, Instant};

    use super::{
        transaction_record_by_id, EditWorldRoute, SelectionSnapshot, TransactionId,
        TransactionRecord, WorldDomain,
    };

    fn transaction_record(sequence: u64) -> TransactionRecord {
        TransactionRecord {
            id: TransactionId::from_sequence(sequence),
            label: String::new(),
            timestamp_frame: sequence,
            route: EditWorldRoute::logical(WorldDomain::Edit),
            commands: Vec::new(),
            participants: Default::default(),
            selection_before: SelectionSnapshot::default(),
            selection_after: SelectionSnapshot::default(),
            significant: true,
        }
    }

    #[test]
    fn undo_and_redo_return_compact_event_metadata_without_copying_detail_records() {
        let source = include_str!("history.rs");
        let undo_body = source
            .split("pub(crate) fn undo")
            .nth(1)
            .and_then(|body| body.split("pub(crate) fn redo").next())
            .expect("undo body should remain available");
        let redo_body = source
            .split("pub(crate) fn redo")
            .nth(1)
            .and_then(|body| body.split("pub(crate) fn mark_saved_current").next())
            .expect("redo body should remain available");
        let full_detail = ["record", ".detail()"].concat();

        assert!(!undo_body.contains(&full_detail));
        assert!(!redo_body.contains(&full_detail));
    }

    #[test]
    fn optimization_wave_20260824e_editor03_history_lookup_handles_wrapped_storage_and_id_gaps() {
        let mut entries = VecDeque::with_capacity(8);
        for sequence in [10, 20, 30, 40, 50, 60, 70, 80] {
            entries.push_back(transaction_record(sequence));
        }
        for _ in 0..4 {
            entries.pop_front();
        }
        for sequence in [90, 100, 110, 120] {
            entries.push_back(transaction_record(sequence));
        }
        let (front, back) = entries.as_slices();

        assert!(!front.is_empty());
        assert!(!back.is_empty());
        assert_eq!(
            transaction_record_by_id(&entries, TransactionId::from_sequence(50))
                .map(|record| record.id.raw()),
            Some(50)
        );
        assert_eq!(
            transaction_record_by_id(&entries, TransactionId::from_sequence(120))
                .map(|record| record.id.raw()),
            Some(120)
        );
        assert!(transaction_record_by_id(&entries, TransactionId::from_sequence(55)).is_none());
    }

    #[test]
    fn optimization_wave_20260824e_editor03_history_journal_lookup_is_logarithmic() {
        let source = include_str!("history.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("history implementation");
        let journal = production
            .split("pub(crate) fn journal")
            .nth(1)
            .and_then(|source| source.split("pub(crate) fn mark_saved_current").next())
            .expect("history journal implementation");

        assert!(journal.contains("transaction_record_by_id"));
        assert!(production.contains("binary_search_by_key"));
        assert!(!journal.contains("self.entries.iter().find"));
    }

    #[test]
    #[ignore = "managed release evidence"]
    fn optimization_wave_20260824e_editor03_history_journal_lookup_evidence() {
        const ENTRY_COUNT: usize = 100_000;
        const LOOKUP_COUNT: usize = 100_000;
        const TARGET: Duration = Duration::from_secs(1);

        let mut entries = VecDeque::with_capacity(ENTRY_COUNT);
        for sequence in 0..ENTRY_COUNT as u64 {
            entries.push_back(transaction_record(sequence));
        }
        for _ in 0..ENTRY_COUNT / 2 {
            entries.pop_front();
        }
        for sequence in ENTRY_COUNT as u64..ENTRY_COUNT as u64 + ENTRY_COUNT as u64 / 2 {
            entries.push_back(transaction_record(sequence));
        }
        let target = TransactionId::from_sequence(ENTRY_COUNT as u64 + ENTRY_COUNT as u64 / 2 - 1);
        let (front, back) = entries.as_slices();
        let binary_comparisons_per_lookup = binary_search_comparison_bound(front.len())
            + binary_search_comparison_bound(back.len());

        let started = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..LOOKUP_COUNT {
            let record = transaction_record_by_id(&entries, std::hint::black_box(target))
                .expect("tail transaction");
            checksum = checksum.wrapping_add(std::hint::black_box(record.id.raw()));
        }
        let elapsed = started.elapsed();
        let comparisons_before = ENTRY_COUNT * LOOKUP_COUNT;
        let comparisons_after_bound = binary_comparisons_per_lookup * LOOKUP_COUNT;
        let comparison_reduction_percent =
            (1.0 - comparisons_after_bound as f64 / comparisons_before as f64) * 100.0;

        assert_ne!(checksum, 0);
        assert!(elapsed <= TARGET, "elapsed={elapsed:?} target={TARGET:?}");
        println!(
            "EDITOR03_HISTORY_LOOKUP_BENCH_V1 entries={} lookups={} comparisons_before={} comparisons_after_bound={} comparison_reduction_percent={:.4} elapsed_ns={} target_ns={}",
            ENTRY_COUNT,
            LOOKUP_COUNT,
            comparisons_before,
            comparisons_after_bound,
            comparison_reduction_percent,
            elapsed.as_nanos(),
            TARGET.as_nanos()
        );
    }

    fn binary_search_comparison_bound(len: usize) -> usize {
        if len == 0 {
            0
        } else {
            (usize::BITS - len.leading_zeros()) as usize
        }
    }
}

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
    // Declared last: the payload is destroyed before its accounting capability.
    pub(crate) retention: Option<super::retention::RootRetentionV1>,
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

    fn retained_bound_v1(
        &self,
        model: super::retention::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, EditCommandError> {
        use super::retention::{RetainedBoundBuilderV1, RetainedBytesV1};
        let mut m = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        m.layout(std::mem::size_of::<Self>(), std::mem::align_of::<Self>())?;
        m.owned(&self.label)?;
        m.array::<CommandBox>(self.commands.capacity())?;
        for command in &self.commands {
            m.owned(command.as_ref())?;
        }
        m.btree::<DocumentId, ()>(self.participants.len())?;
        m.owned(&self.selection_before)?;
        m.owned(&self.selection_after)?;
        Ok(m.bytes().get())
    }
    fn prepare_command_v1(
        &mut self,
        index: usize,
        context: &mut dyn EditContext,
        phase: super::RetentionPhaseV1,
    ) -> Result<(), CommandExecutionError> {
        self.prepare_quoted_command_v1(index, context, phase, None)
    }
    fn prepare_quoted_command_v1(
        &mut self,
        index: usize,
        context: &mut dyn EditContext,
        phase: super::RetentionPhaseV1,
        quoted: Option<&super::CommandRetentionPlanV1>,
    ) -> Result<(), CommandExecutionError> {
        let Some(owner) = self.retention.as_ref() else {
            return Ok(());
        }; // unadmitted standalone records
        let command = &mut self.commands[index];
        let plan = match quoted {
            Some(quote) => {
                let materialization_bytes = quote
                    .retained_bytes
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(quote.peak_bytes))
                    .ok_or_else(|| {
                        CommandExecutionError::unchanged(
                            super::retention::RetentionContractError::Overflow.into(),
                        )
                    })?;
                let plan = command
                    .retention_plan_with_limits_v1(
                        context,
                        phase,
                        owner.model(),
                        super::CommandRetentionLimitsV1 {
                            retained_bytes: quote.retained_bytes,
                            materialization_bytes,
                            selection_bytes: quote.selection_bytes,
                        },
                    )
                    .map_err(CommandExecutionError::unchanged)?;
                // Refresh the current revision after earlier commands, without
                // widening any command's record-wide preflight allowance.
                if plan.retained_bytes > quote.retained_bytes
                    || plan.peak_bytes > quote.peak_bytes
                    || plan.selection_bytes > quote.selection_bytes
                    || quote.phase != phase
                {
                    return Err(CommandExecutionError::unchanged(
                        super::retention::RetentionContractError::Provider {
                            reason: "replay preparation widened its admitted command allowance",
                        }
                        .into(),
                    ));
                }
                plan
            }
            None => command
                .retention_plan_v1(context, phase, owner.model(), owner.budget().record_bytes)
                .map_err(CommandExecutionError::unchanged)?,
        };
        // The record-wide preflight covers these preparations before any model mutation.
        if plan.peak_bytes > owner.reserved_peak_v1() {
            return Err(CommandExecutionError::unchanged(
                super::retention::RetentionContractError::Provider {
                    reason: "replay materialization exceeded its preflight proof",
                }
                .into(),
            ));
        }
        let mut permit = super::RetentionMaterializationV1::new(plan, owner.model());
        command.prepare_retention_v1(context, phase, &mut permit)?;
        permit.finish_v1().map_err(CommandExecutionError::unchanged)
    }
    fn undo(&mut self, context: &mut dyn EditContext) -> Result<(), EditCommandError> {
        self.undo_quoted_v1(context, &[])
    }
    fn undo_quoted_v1(
        &mut self,
        context: &mut dyn EditContext,
        quotes: &[super::CommandRetentionPlanV1],
    ) -> Result<(), EditCommandError> {
        let original_selection = context.selection_snapshot();
        for index in (0..self.commands.len()).rev() {
            let result = self
                .prepare_quoted_command_v1(
                    index,
                    context,
                    super::RetentionPhaseV1::Undo,
                    quotes.get(index),
                )
                .and_then(|()| self.commands[index].revert(context));
            if let Err(command_error) = result {
                if command_error.effect == CommandEffect::Interrupted
                    || command_error.source.is_interrupted()
                {
                    return Err(command_error.into_source());
                }
                return self.recover_failed_undo(context, index, original_selection, command_error);
            }
        }
        if let Err(selection_error) = context.restore_selection(&self.selection_before) {
            if selection_error.is_interrupted() {
                return Err(selection_error);
            }
            return self.recover_all_applied(context, original_selection, selection_error);
        }
        Ok(())
    }

    fn redo(&mut self, context: &mut dyn EditContext) -> Result<(), EditCommandError> {
        self.redo_quoted_v1(context, &[])
    }
    fn redo_quoted_v1(
        &mut self,
        context: &mut dyn EditContext,
        quotes: &[super::CommandRetentionPlanV1],
    ) -> Result<(), EditCommandError> {
        let original_selection = context.selection_snapshot();
        for index in 0..self.commands.len() {
            let result = self
                .prepare_quoted_command_v1(
                    index,
                    context,
                    super::RetentionPhaseV1::Redo,
                    quotes.get(index),
                )
                .and_then(|()| self.commands[index].apply(context));
            if let Err(command_error) = result {
                if command_error.effect == CommandEffect::Interrupted
                    || command_error.source.is_interrupted()
                {
                    return Err(command_error.into_source());
                }
                return self.recover_failed_redo(context, index, original_selection, command_error);
            }
        }
        if let Err(selection_error) = context.restore_selection(&self.selection_after) {
            if selection_error.is_interrupted() {
                return Err(selection_error);
            }
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
            CommandEffect::Interrupted => return Err(command_error.into_source()),
        };
        let command_error = command_error.into_source();
        for rollback_index in rollback_start..self.commands.len() {
            if let Err(rollback_error) = self
                .prepare_command_v1(rollback_index, context, super::RetentionPhaseV1::Apply)
                .and_then(|()| self.commands[rollback_index].apply(context))
            {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.into_source()),
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
            CommandEffect::Interrupted => return Err(command_error.into_source()),
        };
        let command_error = command_error.into_source();
        for rollback_index in (0..rollback_end).rev() {
            if let Err(rollback_error) = self
                .prepare_command_v1(rollback_index, context, super::RetentionPhaseV1::Rollback)
                .and_then(|()| self.commands[rollback_index].revert(context))
            {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.into_source()),
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
        for index in 0..self.commands.len() {
            if let Err(rollback_error) = self
                .prepare_command_v1(index, context, super::RetentionPhaseV1::Apply)
                .and_then(|()| self.commands[index].apply(context))
            {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.into_source()),
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
        for index in (0..self.commands.len()).rev() {
            if let Err(rollback_error) = self
                .prepare_command_v1(index, context, super::RetentionPhaseV1::Rollback)
                .and_then(|()| self.commands[index].revert(context))
            {
                return Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error.into_source()),
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

    pub(crate) fn with_retention_capacity_v1(capacity: usize) -> Result<Self, EditCommandError> {
        let mut store = Self::new(capacity)?;
        let count = capacity
            .checked_add(1)
            .ok_or(super::retention::RetentionContractError::Overflow)?;
        store
            .entries
            .try_reserve_exact(count)
            .map_err(|_| super::retention::RetentionContractError::Allocation)?;
        if store.entries.capacity() != count {
            return Err(super::retention::RetentionContractError::Provider {
                reason: "history VecDeque capacity differs from admitted model",
            }
            .into());
        }
        Ok(store)
    }
    pub(crate) fn backing_capacity_v1(&self) -> usize {
        self.entries.capacity()
    }
    pub(crate) fn retention_owner_v1(
        &self,
        id: TransactionId,
    ) -> Option<&super::retention::RootRetentionV1> {
        transaction_record_by_id(&self.entries, id).and_then(|record| record.retention.as_ref())
    }
    pub(crate) fn reserve_retention_v1(
        &self,
        owner: &super::retention::RootRetentionV1,
        retained: u64,
        peak: u64,
    ) -> Result<(), EditCommandError> {
        use super::retention::RetentionContractError;
        let kept = self.top.map_or(0, |top| top + 1);
        let redo = self.entries.len() - kept;
        if self
            .entries
            .range(kept..)
            .any(|r| r.retention.as_ref().is_some_and(|o| o.protected()))
        {
            return Err(RetentionContractError::Provider {
                reason: "redo branch is pinned",
            }
            .into());
        }
        let mut prefix = kept.saturating_add(1).saturating_sub(self.capacity);
        let mut last_error = None;
        loop {
            if self
                .entries
                .range(..prefix)
                .any(|r| r.retention.as_ref().is_some_and(|o| o.protected()))
            {
                return Err(last_error
                    .unwrap_or_else(|| RetentionContractError::Provider {
                        reason: "pinned record or undo barrier prevents retirement",
                    })
                    .into());
            }
            let mut bytes = owner.charge_for_payload_v1(retained)?;
            for record in self.entries.range(prefix..kept) {
                bytes = bytes
                    .checked_add(
                        record
                            .retention
                            .as_ref()
                            .ok_or(RetentionContractError::Provider {
                                reason: "history record has no retention owner",
                            })?
                            .retained(),
                    )
                    .ok_or(RetentionContractError::Overflow)?;
            }
            if bytes <= owner.budget().history_bytes {
                let old = self
                    .entries
                    .range(..prefix)
                    .chain(self.entries.range(kept..))
                    .filter_map(|r| r.retention.as_ref().map(|o| o.id()));
                match owner.reserve(retained, peak, old, prefix, redo) {
                    Ok(()) => return Ok(()),
                    Err(error @ RetentionContractError::Budget { .. }) => last_error = Some(error),
                    Err(error) => return Err(error.into()),
                }
            } else {
                last_error = Some(RetentionContractError::Budget {
                    budget: "one history",
                    requested: bytes,
                    limit: owner.budget().history_bytes,
                });
            }
            if prefix == kept {
                return Err(last_error.expect("budget refusal has a reason").into());
            }
            prefix += 1;
        }
    }

    pub(crate) fn restore_reservation_v1(
        &self,
        owner: &super::retention::RootRetentionV1,
        checkpoint: super::retention::RetentionReservationCheckpointV1,
    ) {
        let (prefix, redo) = checkpoint.retirement_plan();
        let tail = self.entries.len() - redo;
        owner.restore_reservation_v1(
            checkpoint,
            self.entries
                .range(..prefix)
                .chain(self.entries.range(tail..))
                .filter_map(|record| record.retention.as_ref().map(|o| o.id())),
        );
    }
    pub(crate) fn push_reserved_v1(&mut self, record: TransactionRecord) -> Vec<TransactionRecord> {
        let owner = record
            .retention
            .as_ref()
            .expect("engine records have a retention capability");
        let (prefix, redo) = owner.retirement_plan();
        let kept = self.top.map_or(0, |top| top + 1);
        assert_eq!(
            self.entries.len() - kept,
            redo,
            "reserved redo branch remained stable"
        );
        assert!(prefix <= kept);
        // The scratch allocation and every removed owner are covered by the preparation peak.
        let mut removed = Vec::with_capacity(self.capacity + 1);
        if self.saved_top_reachable && self.saved_top.is_some_and(|saved| saved >= kept) {
            self.saved_top_reachable = false;
        }
        removed.extend(self.entries.drain(kept..));
        for _ in 0..prefix {
            removed.push(self.entries.pop_front().expect("reserved prefix"));
            if self.saved_top_reachable {
                match self.saved_top {
                    Some(saved) => self.saved_top = saved.checked_sub(1),
                    None => self.saved_top_reachable = false,
                }
            }
        }
        self.entries.push_back(record);
        self.top = Some(self.entries.len() - 1);
        self.entries
            .back()
            .and_then(|r| r.retention.as_ref())
            .expect("owner")
            .ready();
        removed
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
    pub(crate) fn undo_quoted_v1(
        &mut self,
        context: &mut dyn EditContext,
        quotes: &[super::CommandRetentionPlanV1],
    ) -> Result<Option<(TransactionId, String)>, EditCommandError> {
        let Some(top) = self.top else {
            return Ok(None);
        };
        let Some(record) = self.entries.get_mut(top) else {
            return Err(EditCommandError::InvariantViolation {
                invariant: "history top points at an existing record",
            });
        };
        record.undo_quoted_v1(context, quotes)?;
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
    pub(crate) fn redo_quoted_v1(
        &mut self,
        context: &mut dyn EditContext,
        quotes: &[super::CommandRetentionPlanV1],
    ) -> Result<Option<(TransactionId, String)>, EditCommandError> {
        let next = self.top.map_or(0, |top| top + 1);
        let Some(record) = self.entries.get_mut(next) else {
            return Ok(None);
        };
        record.redo_quoted_v1(context, quotes)?;
        let event_metadata = (record.id, record.label.clone());
        self.top = Some(next);
        Ok(Some(event_metadata))
    }

    pub(crate) fn prepare_replay_v1(
        &self,
        context: &dyn EditContext,
        undo: bool,
    ) -> Result<Vec<super::CommandRetentionPlanV1>, EditCommandError> {
        use super::retention::{RetainedBoundBuilderV1, RetainedBytesV1, RetentionContractError};
        let index = if undo {
            self.top
        } else {
            Some(self.top.map_or(0, |top| top + 1))
        };
        let Some(record) = index.and_then(|index| self.entries.get(index)) else {
            return Ok(Vec::new());
        };
        let Some(owner) = record.retention.as_ref() else {
            return Ok(Vec::new());
        };
        let model = owner.model();
        let budget = owner.budget();
        let selection = context.selection_retention_bound_v1(model, budget.record_bytes)?;
        let actual = record.retained_bound_v1(model, budget.record_bytes)?;
        let old = owner.payload_bytes();
        let lease = owner.charge_for_payload_v1(0)?;
        let mut other_history = 0u64;
        for other in &self.entries {
            if other.id != record.id {
                other_history = other_history
                    .checked_add(other.retention.as_ref().map_or(0, |o| o.retained()))
                    .ok_or(RetentionContractError::Overflow)?;
            }
        }
        let history_remaining = budget
            .history_bytes
            .checked_sub(other_history)
            .and_then(|n| n.checked_sub(lease))
            .ok_or(RetentionContractError::Budget {
                budget: "one replaying history",
                requested: other_history,
                limit: budget.history_bytes,
            })?;
        let record_remaining = budget
            .record_bytes
            .checked_sub(lease)
            .ok_or(RetentionContractError::Overflow)?
            .min(history_remaining);
        let mut metadata =
            RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(budget.staging_bytes));
        metadata.layout(
            std::mem::size_of::<Vec<super::CommandRetentionPlanV1>>(),
            std::mem::align_of::<Vec<super::CommandRetentionPlanV1>>(),
        )?;
        metadata.array::<super::CommandRetentionPlanV1>(record.commands.len())?;
        metadata.add(selection)?;
        let fixed = metadata.bytes().get();
        let headroom = owner.materialization_headroom_v1()?;
        let checkpoint = owner.checkpoint_v1();
        // The temporary quote table is itself reserved before allocation.
        let baseline = actual.max(old);
        let initial_peak = baseline
            .checked_mul(2)
            .and_then(|n| n.checked_add(fixed))
            .ok_or(RetentionContractError::Overflow)?;
        owner.reserve(baseline, initial_peak, std::iter::empty(), 0, 0)?;
        let result = (|| {
            let mut quotes = Vec::new();
            quotes
                .try_reserve_exact(record.commands.len())
                .map_err(|_| RetentionContractError::Overflow)?;
            if quotes.capacity() != record.commands.len() {
                return Err(RetentionContractError::Provider {
                    reason: "replay quote backing differs from the supported Vec model",
                }
                .into());
            }
            let phase = if undo {
                super::RetentionPhaseV1::Undo
            } else {
                super::RetentionPhaseV1::Redo
            };
            let mut bound = actual;
            let mut peak = 0u64;
            for command in &record.commands {
                let floor = bound
                    .checked_add(selection)
                    .ok_or(RetentionContractError::Overflow)?
                    .max(old);
                let fixed_peak = floor
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(fixed))
                    .and_then(|n| n.checked_add(peak))
                    .and_then(|n| n.checked_add(lease))
                    .ok_or(RetentionContractError::Overflow)?;
                let retained_bytes = record_remaining
                    .checked_sub(bound)
                    .and_then(|n| n.checked_sub(selection))
                    .ok_or(RetentionContractError::Budget {
                        budget: "remaining replay command record",
                        requested: bound,
                        limit: record_remaining,
                    })?;
                let materialization_bytes =
                    headroom
                        .checked_sub(fixed_peak)
                        .ok_or(RetentionContractError::Budget {
                            budget: "remaining replay materialization",
                            requested: fixed_peak,
                            limit: headroom,
                        })?;
                let quote = command.retention_plan_with_limits_v1(
                    context,
                    phase,
                    model,
                    super::CommandRetentionLimitsV1 {
                        retained_bytes,
                        materialization_bytes,
                        selection_bytes: selection,
                    },
                )?;
                bound = bound
                    .checked_add(quote.retained_bytes)
                    .and_then(|n| n.checked_add(quote.selection_bytes))
                    .ok_or(RetentionContractError::Overflow)?;
                peak = peak
                    .checked_add(quote.peak_bytes)
                    .ok_or(RetentionContractError::Overflow)?;
                if bound > record_remaining {
                    return Err(RetentionContractError::Budget {
                        budget: "one replaying history record",
                        requested: bound,
                        limit: record_remaining,
                    }
                    .into());
                }
                quotes.push(quote);
            }
            let bound = bound.max(old);
            let peak = bound
                .checked_mul(2)
                .and_then(|n| n.checked_add(fixed))
                .and_then(|n| n.checked_add(peak))
                .ok_or(RetentionContractError::Overflow)?;
            owner.reserve(bound, peak, std::iter::empty(), 0, 0)?;
            Ok(quotes)
        })();
        if result.is_err() {
            // The closure has dropped its temporary table before releasing its
            // charge. No cursor, saved point, model or history row changed.
            owner.restore_reservation_v1(checkpoint, std::iter::empty());
        }
        result
    }
    pub(crate) fn finish_replay_v1(
        &self,
        undo: bool,
        previous_top: Option<TransactionId>,
        faulted: bool,
    ) -> Result<(), EditCommandError> {
        if faulted {
            return Ok(());
        }
        let record = if undo {
            previous_top.and_then(|id| transaction_record_by_id(&self.entries, id))
        } else {
            let next = previous_top
                .and_then(|id| self.entries.iter().position(|r| r.id == id))
                .map_or(0, |i| i + 1);
            self.entries.get(next)
        };
        if let Some(record) = record {
            if let Some(owner) = record.retention.as_ref() {
                let retained =
                    record.retained_bound_v1(owner.model(), owner.budget().record_bytes)?;
                owner.finish_replay(retained.max(owner.payload_bytes()))?;
            }
        }
        Ok(())
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
        self.top.is_some_and(|top| {
            !self.entries[top]
                .retention
                .as_ref()
                .is_some_and(|owner| owner.is_barrier())
        })
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
        for record in &self.entries {
            if let Some(owner) = &record.retention {
                owner.retire();
            }
        }
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
#[path = "tests/history_performance_source_guards.rs"]
mod performance_source_guards;

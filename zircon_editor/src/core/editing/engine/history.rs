use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::core::editor_message::DocumentId;

use super::{
    CommandBox, CommandEffect, CommandExecutionError, EditCommandError, EditContext,
    SelectionSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HistoryContextId {
    Global,
    Document(DocumentId),
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

pub struct TransactionRecord {
    pub id: TransactionId,
    pub label: String,
    pub timestamp_frame: u64,
    pub commands: Vec<CommandBox>,
    pub participants: BTreeSet<DocumentId>,
    pub selection_before: SelectionSnapshot,
    pub selection_after: SelectionSnapshot,
    pub significant: bool,
}

impl TransactionRecord {
    pub(crate) fn snapshot(&self) -> TransactionRecordSnapshot {
        TransactionRecordSnapshot {
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
pub struct TransactionRecordSnapshot {
    pub id: TransactionId,
    pub label: String,
    pub timestamp_frame: u64,
    pub command_count: usize,
    pub participants: BTreeSet<DocumentId>,
    pub selection_before: SelectionSnapshot,
    pub selection_after: SelectionSnapshot,
    pub significant: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistorySnapshot {
    pub len: usize,
    pub top: Option<usize>,
    pub saved_top: Option<usize>,
    pub saved_top_reachable: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub records: Vec<TransactionRecordSnapshot>,
}

impl HistorySnapshot {
    pub(crate) fn empty() -> Self {
        Self {
            len: 0,
            top: None,
            saved_top: None,
            saved_top_reachable: true,
            can_undo: false,
            can_redo: false,
            records: Vec::new(),
        }
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
    ) -> Result<Option<TransactionRecordSnapshot>, EditCommandError> {
        let Some(top) = self.top else {
            return Ok(None);
        };
        let Some(record) = self.entries.get_mut(top) else {
            return Err(EditCommandError::InvariantViolation {
                invariant: "history top points at an existing record",
            });
        };
        record.undo(context)?;
        let snapshot = record.snapshot();
        self.top = top.checked_sub(1);
        Ok(Some(snapshot))
    }

    pub(crate) fn redo(
        &mut self,
        context: &mut dyn EditContext,
    ) -> Result<Option<TransactionRecordSnapshot>, EditCommandError> {
        let next = self.top.map_or(0, |top| top + 1);
        let Some(record) = self.entries.get_mut(next) else {
            return Ok(None);
        };
        record.redo(context)?;
        let snapshot = record.snapshot();
        self.top = Some(next);
        Ok(Some(snapshot))
    }

    pub fn mark_saved(&mut self) {
        self.saved_top = self.top;
        self.saved_top_reachable = true;
    }

    pub fn is_dirty(&self) -> bool {
        !self.saved_top_reachable || self.top != self.saved_top
    }

    pub fn snapshot(&self) -> HistorySnapshot {
        HistorySnapshot {
            len: self.entries.len(),
            top: self.top,
            saved_top: self.saved_top,
            saved_top_reachable: self.saved_top_reachable,
            can_undo: self.top.is_some(),
            can_redo: self
                .top
                .map_or(!self.entries.is_empty(), |top| top + 1 < self.entries.len()),
            records: self
                .entries
                .iter()
                .map(TransactionRecord::snapshot)
                .collect(),
        }
    }
}

use std::collections::HashMap;

use crate::error::HubError;
use crate::process::editor_child_receipt::{EditorChildDisposition, EditorChildTerminalReceipt};
use crate::projects::now_unix_ms;
use crate::state::{
    HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId, ProcessMessageId,
    TaskOperationKind, TaskStatus,
};

use super::HubRuntimeSession;

#[derive(Default)]
pub(super) struct EditorProcessLedger {
    attempts: HashMap<u64, AttemptState>,
    next_delivery_sequence: u64,
}

#[derive(Default)]
struct AttemptState {
    launch: Option<EditorLaunchAttempt>,
    pre_ready_target: Option<String>,
    pending_terminal: Vec<EditorChildTerminalReceipt>,
    delivered: bool,
    pending_persist: Option<(u64, HubActionRecord)>,
}

pub(super) struct EditorLaunchAttempt {
    pub(super) attempt_id: u64,
    pub(super) process_id: u32,
    pub(super) target: String,
}

impl EditorLaunchAttempt {
    pub(super) fn new(attempt_id: u64, process_id: u32, target: impl Into<String>) -> Self {
        Self {
            attempt_id,
            process_id,
            target: target.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EditorTerminalEvent {
    pub(super) attempt_id: u64,
    pub(super) process_id: u32,
    pub(super) target: String,
    pub(super) receipt: EditorChildTerminalReceipt,
}

impl EditorProcessLedger {
    pub(super) fn admit_editor_attempt(&mut self, attempt_id: u64, target: String) {
        self.attempts
            .entry(attempt_id)
            .or_default()
            .pre_ready_target = Some(target);
    }

    fn has_pending_persist(&self) -> bool {
        self.attempts
            .values()
            .any(|state| state.pending_persist.is_some())
    }

    fn is_pending_persist(&self, attempt_id: u64) -> bool {
        self.attempts
            .get(&attempt_id)
            .is_some_and(|state| state.pending_persist.is_some())
    }

    fn mark_pending_persist(&mut self, attempt_id: u64, record: HubActionRecord) {
        if let Some(state) = self.attempts.get_mut(&attempt_id) {
            if state.pending_persist.is_none() {
                self.next_delivery_sequence = self.next_delivery_sequence.saturating_add(1);
                state.pending_persist = Some((self.next_delivery_sequence, record));
            }
        }
    }

    fn pending_records(&self) -> Vec<(u64, HubActionRecord)> {
        let mut pending = self
            .attempts
            .values()
            .filter_map(|state| state.pending_persist.clone())
            .collect::<Vec<_>>();
        pending.sort_by_key(|(sequence, _)| *sequence);
        pending
    }

    fn restore_pending_records(
        &self,
        history: &mut Vec<HubActionRecord>,
        pending: &[(u64, HubActionRecord)],
    ) {
        for (sequence, record) in pending {
            if history.contains(record) {
                continue;
            }
            let index = history
                .iter()
                .position(|existing| {
                    existing.finished_unix_ms < record.finished_unix_ms
                        || (existing.finished_unix_ms == record.finished_unix_ms
                            && pending.iter().any(|(existing_sequence, pending_record)| {
                                existing == pending_record && existing_sequence < sequence
                            }))
                })
                .unwrap_or(history.len());
            history.insert(index, record.clone());
        }
        while history.len() > crate::state::ACTION_HISTORY_LIMIT {
            let index = history
                .iter()
                .rposition(|existing| !pending.iter().any(|(_, record)| existing == record))
                .expect("pending terminals fit action history capacity");
            history.remove(index);
        }
    }

    fn clear_pending_persist(&mut self) {
        for state in self.attempts.values_mut() {
            state.pending_persist = None;
        }
    }

    pub(super) fn observe_launch(
        &mut self,
        launch: EditorLaunchAttempt,
    ) -> Option<EditorTerminalEvent> {
        let state = self.attempts.entry(launch.attempt_id).or_default();
        if state.launch.is_some() || state.delivered {
            return None;
        }
        state.pre_ready_target = None;
        state.launch = Some(launch);
        let process_id = state.launch.as_ref()?.process_id;
        let matching_index = state
            .pending_terminal
            .iter()
            .position(|receipt| receipt.process_id == process_id);
        let receipt = matching_index.map(|index| state.pending_terminal.swap_remove(index));
        state.pending_terminal.clear();
        let receipt = receipt?;
        Self::deliver(state, receipt)
    }

    pub(super) fn observe_terminal(
        &mut self,
        receipt: EditorChildTerminalReceipt,
    ) -> Option<EditorTerminalEvent> {
        let state = self.attempts.entry(receipt.attempt_id).or_default();
        if state.delivered {
            return None;
        }
        if state.launch.is_none()
            && matches!(
                receipt.disposition,
                EditorChildDisposition::StoppedBeforeReady
                    | EditorChildDisposition::FailedBeforeReady
            )
        {
            if let Some(target) = state.pre_ready_target.take() {
                state.launch = Some(EditorLaunchAttempt::new(
                    receipt.attempt_id,
                    receipt.process_id,
                    target,
                ));
            }
        }
        if state.launch.is_none() {
            if !state
                .pending_terminal
                .iter()
                .any(|pending| pending.process_id == receipt.process_id)
            {
                state.pending_terminal.push(receipt);
            }
            return None;
        }
        Self::deliver(state, receipt)
    }

    fn deliver(
        state: &mut AttemptState,
        receipt: EditorChildTerminalReceipt,
    ) -> Option<EditorTerminalEvent> {
        let launch = state.launch.as_ref()?;
        if launch.process_id != receipt.process_id {
            return None;
        }
        state.delivered = true;
        Some(EditorTerminalEvent {
            attempt_id: launch.attempt_id,
            process_id: launch.process_id,
            target: launch.target.clone(),
            receipt,
        })
    }
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn retry_pending_editor_terminal_persistence(
        &mut self,
    ) -> Result<bool, HubError> {
        if !self.editor_process_ledger.has_pending_persist() {
            return Ok(false);
        }
        let pending = self.editor_process_ledger.pending_records();
        if pending.len() > crate::state::ACTION_HISTORY_LIMIT {
            return Err(HubError::message(
                "Editor terminal history exceeds the Hub action-history capacity",
            ));
        }
        self.editor_process_ledger
            .restore_pending_records(&mut self.config.action_history, &pending);
        self.persist()?;
        self.editor_process_ledger.clear_pending_persist();
        Ok(true)
    }

    pub(in crate::tauri_app) fn record_editor_supervision_incomplete(
        &mut self,
        launch_task_id: Option<u64>,
        child_reaper_incomplete: bool,
        unprojected_terminals: &[EditorChildTerminalReceipt],
    ) -> Result<(), HubError> {
        let mut details = Vec::new();
        if let Some(task_id) = launch_task_id {
            details.push(HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorLaunchShutdownIncomplete),
                [task_id.to_string()],
            ));
        }
        if child_reaper_incomplete {
            details.push(HubMessage::new(HubMessageId::Process(
                ProcessMessageId::EditorChildShutdownIncomplete,
            )));
        }
        let recovery = HubMessage::new(HubMessageId::Process(
            ProcessMessageId::VerifyEditorAndProjectPath,
        ));
        if let Some(task_id) = launch_task_id {
            if self.task_status.task_id == task_id {
                self.task_status =
                    TaskStatus::error("Open Editor failed", details[0].clone(), recovery.clone())
                        .with_operation(TaskOperationKind::Process, "Editor")
                        .with_task_id(task_id);
            }
        }
        for detail in details {
            self.config.action_history.insert(
                0,
                HubActionRecord {
                    finished_unix_ms: now_unix_ms(),
                    action: HubActionKind::OpenEditor,
                    status: HubActionStatus::Failed,
                    target: "Editor".to_string(),
                    detail,
                    log_excerpt: HubMessage::empty(),
                    recovery: Some(recovery.clone()),
                    process_id: None,
                    command_line: Vec::new(),
                    output_dir: None,
                },
            );
        }
        for receipt in unprojected_terminals {
            self.config.action_history.insert(
                0,
                HubActionRecord {
                    finished_unix_ms: now_unix_ms(),
                    action: HubActionKind::OpenEditor,
                    status: HubActionStatus::Failed,
                    target: "Editor".to_string(),
                    detail: HubMessage::with_params(
                        HubMessageId::Process(ProcessMessageId::EditorTerminalProjectionIncomplete),
                        [
                            receipt.attempt_id.to_string(),
                            receipt.process_id.to_string(),
                        ],
                    ),
                    log_excerpt: HubMessage::empty(),
                    recovery: Some(recovery.clone()),
                    process_id: Some(receipt.process_id),
                    command_line: Vec::new(),
                    output_dir: None,
                },
            );
        }
        self.config
            .action_history
            .truncate(crate::state::ACTION_HISTORY_LIMIT);
        self.persist()
    }

    pub(super) fn observe_completed_editor_launch(
        &mut self,
        attempt_id: u64,
        process_id: u32,
        target: String,
    ) -> Result<(), HubError> {
        let terminal = self
            .editor_process_ledger
            .observe_launch(EditorLaunchAttempt::new(attempt_id, process_id, target));
        if let Some(terminal) = terminal {
            self.record_editor_terminal(terminal)?;
        }
        Ok(())
    }

    pub(in crate::tauri_app) fn observe_editor_child_terminal(
        &mut self,
        receipt: EditorChildTerminalReceipt,
    ) -> Result<bool, HubError> {
        if self
            .editor_process_ledger
            .is_pending_persist(receipt.attempt_id)
        {
            return self.retry_pending_editor_terminal_persistence();
        }
        let Some(terminal) = self.editor_process_ledger.observe_terminal(receipt) else {
            return Ok(false);
        };
        self.record_editor_terminal(terminal)?;
        Ok(true)
    }

    fn record_editor_terminal(&mut self, terminal: EditorTerminalEvent) -> Result<(), HubError> {
        let (status, detail) = terminal_status_detail(&terminal.receipt);
        let recovery = (status == HubActionStatus::Failed).then(|| {
            HubMessage::new(HubMessageId::Process(
                ProcessMessageId::VerifyEditorAndProjectPath,
            ))
        });
        if self.task_status.task_id == terminal.attempt_id {
            match status {
                HubActionStatus::Success => {
                    self.task_status.detail = detail.clone();
                }
                HubActionStatus::Failed => {
                    self.task_status = TaskStatus::error(
                        "Open Editor failed",
                        detail.clone(),
                        recovery.clone().expect("failed terminal has recovery"),
                    )
                    .with_operation(TaskOperationKind::Process, terminal.target.clone())
                    .with_task_id(terminal.attempt_id);
                }
                HubActionStatus::Cancelled => {
                    self.task_status =
                        TaskStatus::cancelled("Task cancelled", detail.clone(), None)
                            .with_operation(TaskOperationKind::Process, terminal.target.clone())
                            .with_task_id(terminal.attempt_id);
                }
            }
        }
        if let Some(error) = terminal.receipt.cleanup_error.as_deref() {
            eprintln!(
                "zircon_hub: Editor process {} cleanup incomplete: {error}",
                terminal.process_id
            );
        }
        let record = HubActionRecord {
            finished_unix_ms: now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status,
            target: terminal.target,
            detail,
            log_excerpt: HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorTerminalAttemptId),
                [terminal.attempt_id.to_string()],
            ),
            recovery,
            process_id: Some(terminal.process_id),
            command_line: Vec::new(),
            output_dir: None,
        };
        let result = self.record_action_and_persist(record.clone());
        if result.is_err() {
            self.editor_process_ledger
                .mark_pending_persist(terminal.attempt_id, record);
        }
        result
    }
}

fn terminal_status_detail(receipt: &EditorChildTerminalReceipt) -> (HubActionStatus, HubMessage) {
    let process_id = receipt.process_id.to_string();
    if receipt.cleanup_error.is_some() {
        return (
            HubActionStatus::Failed,
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessCleanupIncomplete),
                [process_id],
            ),
        );
    }
    match receipt.disposition {
        EditorChildDisposition::Exited {
            code: Some(code), ..
        } => (
            if code == 0 {
                HubActionStatus::Success
            } else {
                HubActionStatus::Failed
            },
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessExitCode),
                [process_id, code.to_string()],
            ),
        ),
        EditorChildDisposition::Exited {
            code: None,
            signal: Some(signal),
        } => (
            HubActionStatus::Failed,
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessExitSignal),
                [process_id, signal.to_string()],
            ),
        ),
        EditorChildDisposition::Exited {
            code: None,
            signal: None,
        } => (
            HubActionStatus::Failed,
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessExitUnknown),
                [process_id],
            ),
        ),
        EditorChildDisposition::StoppedByHub => (
            HubActionStatus::Cancelled,
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessStopped),
                [process_id],
            ),
        ),
        EditorChildDisposition::StoppedBeforeReady => (
            HubActionStatus::Cancelled,
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessStoppedBeforeReady),
                [process_id],
            ),
        ),
        EditorChildDisposition::FailedBeforeReady => (
            HubActionStatus::Failed,
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorProcessFailedBeforeReady),
                [process_id],
            ),
        ),
    }
}

#[cfg(test)]
#[path = "editor_process_ledger/tests/cases.rs"]
mod tests;

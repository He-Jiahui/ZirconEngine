use zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionLifecycleV1;

use crate::core::recovery::{
    ProjectSessionEffect, ProjectSessionEffectLedgerError, ProjectSessionEffectLedgerStore,
};

use super::super::editor_manager::EditorManager;
use super::{ProjectCloseError, ProjectCloseOperation, ProjectCloseReceipt};

impl EditorManager {
    /// Persists the non-ready close phase and returns the only capability accepted by teardown.
    pub(super) fn begin_project_close_operation(
        &self,
    ) -> Result<Option<ProjectCloseOperation>, ProjectCloseError> {
        let mut heartbeat = self
            .project_session_heartbeat
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(guard) = guard_slot.as_mut() else {
            heartbeat.clear();
            return Ok(None);
        };
        let operation = ProjectCloseOperation::new(
            guard.project_root().to_path_buf(),
            guard.record().operation_id(),
        );
        if let Err(error) = guard.begin_close() {
            let recovery_detail = guard
                .mark_recovery_required()
                .err()
                .map(|recovery_error| {
                    format!("; additionally failed to persist RecoveryRequired: {recovery_error}")
                })
                .unwrap_or_default();
            heartbeat.clear();
            return Err(ProjectCloseError::new(
                operation,
                ProjectSessionEffect::DirtyDocuments,
                None,
                format!("cannot persist Closing: {error}{recovery_detail}"),
            ));
        }
        heartbeat.clear();
        drop(guard_slot);
        drop(heartbeat);

        let mut ledger = ProjectSessionEffectLedgerStore::load(
            operation.project_root(),
            operation.operation_id(),
        )
        .map_err(|error| {
            self.require_project_close_recovery(
                &operation,
                ProjectSessionEffect::DirtyDocuments,
                format!("cannot load the Ready session effect ledger: {error}"),
            )
        })?;
        ledger.begin_closing().map_err(|error| {
            self.require_project_close_recovery(
                &operation,
                ProjectSessionEffect::DirtyDocuments,
                format!("cannot persist the Closing effect phase: {error}"),
            )
        })?;
        ledger
            .prepare(ProjectSessionEffect::DirtyDocuments)
            .and_then(|()| ledger.commit(ProjectSessionEffect::DirtyDocuments))
            .map_err(|error| {
                self.require_project_close_recovery(
                    &operation,
                    ProjectSessionEffect::DirtyDocuments,
                    format!("cannot commit the clean-document close decision: {error}"),
                )
            })?;
        Ok(Some(operation))
    }

    pub(crate) fn prepare_project_close_effect(
        &self,
        operation: &ProjectCloseOperation,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectCloseError> {
        let mut ledger = self.load_project_close_ledger(operation, effect)?;
        ledger.prepare(effect).map_err(|error| {
            self.require_project_close_recovery(
                operation,
                effect,
                format!("cannot prepare close effect: {error}"),
            )
        })
    }

    pub(crate) fn commit_project_close_effect(
        &self,
        operation: &ProjectCloseOperation,
        effect: ProjectSessionEffect,
    ) -> Result<ProjectCloseReceipt, ProjectCloseError> {
        let mut ledger = self.load_project_close_ledger(operation, effect)?;
        ledger.commit(effect).map_err(|error| {
            self.require_project_close_recovery(
                operation,
                effect,
                format!("cannot commit close effect: {error}"),
            )
        })?;
        Ok(ProjectCloseReceipt::from_ledger(
            operation.clone(),
            ledger.ledger(),
        ))
    }

    pub(crate) fn require_project_close_recovery(
        &self,
        operation: &ProjectCloseOperation,
        effect: ProjectSessionEffect,
        message: impl Into<String>,
    ) -> ProjectCloseError {
        let message = message.into();
        let mut ledger_detail = String::new();
        let receipt = match ProjectSessionEffectLedgerStore::load(
            operation.project_root(),
            operation.operation_id(),
        ) {
            Ok(mut ledger) => {
                if ledger.ledger().disposition(effect).is_none() {
                    let _ = ledger.prepare(effect);
                }
                if let Err(error) = ledger.mark_recovery_required(effect) {
                    ledger_detail = format!(
                        "; additionally failed to persist exact effect recovery state: {error}"
                    );
                }
                Some(ProjectCloseReceipt::from_ledger(
                    operation.clone(),
                    ledger.ledger(),
                ))
            }
            Err(error) => {
                ledger_detail =
                    format!("; additionally failed to load the session effect ledger: {error}");
                None
            }
        };
        let guard_detail = self
            .mark_project_close_guard_recovery(operation)
            .err()
            .map(|error| format!("; additionally failed to retain the session guard: {error}"))
            .unwrap_or_default();
        ProjectCloseError::new(
            operation.clone(),
            effect,
            receipt,
            format!("{message}{ledger_detail}{guard_detail}"),
        )
    }

    pub(super) fn finish_project_close_ledger(
        &self,
        operation: &ProjectCloseOperation,
    ) -> Result<ProjectCloseReceipt, ProjectCloseError> {
        let mut ledger =
            self.load_project_close_ledger(operation, ProjectSessionEffect::Session)?;
        ledger
            .prepare(ProjectSessionEffect::Session)
            .and_then(|()| ledger.commit(ProjectSessionEffect::Session))
            .and_then(|()| ledger.finish_closed())
            .map_err(|error| {
                self.require_project_close_recovery(
                    operation,
                    ProjectSessionEffect::Session,
                    format!("cannot commit the terminal close receipt: {error}"),
                )
            })?;
        Ok(ProjectCloseReceipt::from_ledger(
            operation.clone(),
            ledger.ledger(),
        ))
    }

    pub(super) fn release_project_close_guard(
        &self,
        operation: &ProjectCloseOperation,
    ) -> Result<(), ProjectCloseError> {
        let mut heartbeat = self
            .project_session_heartbeat
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(guard) = guard_slot.as_mut() else {
            heartbeat.clear();
            return Err(ProjectCloseError::new(
                operation.clone(),
                ProjectSessionEffect::Session,
                None,
                "the terminal close receipt has no matching session guard",
            ));
        };
        if guard.project_root() != operation.project_root()
            || guard.record().operation_id() != operation.operation_id()
        {
            heartbeat.clear();
            return Err(ProjectCloseError::new(
                operation.clone(),
                ProjectSessionEffect::Session,
                None,
                "the active session guard belongs to a different close operation",
            ));
        }
        if let Err(error) = guard.release() {
            heartbeat.clear();
            drop(guard_slot);
            drop(heartbeat);
            return Err(self.require_project_close_recovery(
                operation,
                ProjectSessionEffect::Session,
                format!("cannot release the terminal session guard: {error}"),
            ));
        }
        guard_slot.take();
        heartbeat.clear();
        Ok(())
    }

    pub(super) fn cleanup_closed_project_session_ledger(
        &self,
        operation: &ProjectCloseOperation,
    ) -> Result<bool, ProjectSessionEffectLedgerError> {
        ProjectSessionEffectLedgerStore::load(operation.project_root(), operation.operation_id())?
            .cleanup_if_closed()
    }

    fn load_project_close_ledger(
        &self,
        operation: &ProjectCloseOperation,
        effect: ProjectSessionEffect,
    ) -> Result<ProjectSessionEffectLedgerStore, ProjectCloseError> {
        self.validate_project_close_operation(operation, effect)?;
        ProjectSessionEffectLedgerStore::load(operation.project_root(), operation.operation_id())
            .map_err(|error| {
                self.require_project_close_recovery(
                    operation,
                    effect,
                    format!("cannot load the session effect ledger: {error}"),
                )
            })
    }

    fn validate_project_close_operation(
        &self,
        operation: &ProjectCloseOperation,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectCloseError> {
        let guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(guard) = guard_slot.as_ref() else {
            return Err(ProjectCloseError::new(
                operation.clone(),
                effect,
                None,
                "project close has no active session guard",
            ));
        };
        if guard.project_root() != operation.project_root()
            || guard.record().operation_id() != operation.operation_id()
            || guard.record().lifecycle() != ProjectSessionAdmissionLifecycleV1::Closing
        {
            return Err(ProjectCloseError::new(
                operation.clone(),
                effect,
                None,
                "project close capability does not match the active Closing session",
            ));
        }
        Ok(())
    }

    fn mark_project_close_guard_recovery(
        &self,
        operation: &ProjectCloseOperation,
    ) -> Result<(), String> {
        let mut heartbeat = self
            .project_session_heartbeat
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        heartbeat.clear();
        let Some(guard) = guard_slot.as_mut() else {
            return Err("no active project session guard is available".to_string());
        };
        if guard.project_root() != operation.project_root()
            || guard.record().operation_id() != operation.operation_id()
        {
            return Err(
                "the active project session guard belongs to another operation".to_string(),
            );
        }
        guard
            .mark_recovery_required()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

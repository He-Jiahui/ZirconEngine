use super::*;
use crate::core::logging::{LogEntry, LogJump, LogSeverity, LogSource};
use crate::core::recovery::{
    AutosaveDocumentOutcome, AutosaveDocumentOutcomeKind, AutosaveDocumentRequest,
    AutosaveDocumentState,
};

impl RetainedEditorHost {
    /// Captures only document-bound autosave admission requests for the final
    /// shutdown drain. Serialization remains deferred to the recovery worker.
    pub(in crate::ui::retained_host::app) fn final_autosave_requests(
        &self,
    ) -> Result<Vec<AutosaveDocumentRequest>, String> {
        let Some(project_root) = self
            .startup_session
            .project
            .as_ref()
            .map(|project| project.root_path.clone())
        else {
            return Ok(Vec::new());
        };
        self.editor_manager
            .dirty_document_toolkits()
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|document| {
                let identity = self
                    .editor_manager
                    .autosave_document_identity(document.document_id, &project_root)
                    .map_err(|error| error.to_string())?;
                self.editor_manager
                    .autosave_document_request(
                        document.document_id,
                        document.dirty_generation,
                        identity,
                    )
                    .map_err(|error| error.to_string())
            })
            .collect()
    }

    pub(in crate::ui::retained_host::app) fn poll_editor_autosave(&mut self) {
        if self.editor_manager.project_recovery_is_active() {
            return;
        }
        let project_root = self
            .startup_session
            .project
            .as_ref()
            .map(|project| project.root_path.clone());
        let poll = self
            .editor_manager
            .context()
            .autosave()
            .poll_project(project_root.as_deref());
        let completion = poll.completion();
        for outcome in completion.outcomes() {
            self.report_autosave_outcome(outcome);
        }
        for issue in poll.diagnostic_persistence_issues() {
            self.set_status_line(format!(
                "Autosave diagnostic persistence failed for {} in {}: {}",
                issue.document().as_str(),
                issue.project_root().display(),
                issue.message()
            ));
        }
        if !poll.is_due() {
            return;
        }
        let Some(project_root) = project_root else {
            return;
        };
        match self
            .editor_manager
            .context()
            .autosave()
            .preflight_schedule(poll.now())
        {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        }
        let dirty_documents = match self.editor_manager.dirty_document_toolkits() {
            Ok(documents) => documents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let intents = match dirty_documents
            .into_iter()
            .map(|document| {
                self.editor_manager
                    .autosave_document_identity(document.document_id, &project_root)
                    .map(|identity| (document, identity))
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(intents) => intents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let documents = intents
            .iter()
            .map(|(_, identity)| {
                AutosaveDocumentState::from_dirty_projection(identity.document().clone(), true)
            })
            .collect::<Vec<_>>();
        if documents.is_empty() {
            return;
        }

        let editor_manager = Arc::clone(&self.editor_manager);
        let mut request_error = None;
        let result = self.editor_manager.context().autosave().schedule(
            poll.now(),
            &documents,
            |_| std::mem::size_of::<AutosaveDocumentRequest>().max(1),
            |autosave_document| {
                let (dirty, identity) = intents
                    .iter()
                    .find(|(_, identity)| identity.document() == autosave_document)?;
                match editor_manager.autosave_document_request(
                    dirty.document_id,
                    dirty.dirty_generation,
                    identity.clone(),
                ) {
                    Ok(request) => Some(request),
                    Err(error) => {
                        request_error = Some(error.to_string());
                        None
                    }
                }
            },
        );
        if let Some(error) = request_error {
            self.set_status_line(error);
        } else if let Err(error) = result {
            self.set_status_line(error.to_string());
        }
    }

    fn report_autosave_outcome(&mut self, outcome: &AutosaveDocumentOutcome) {
        let (stage, error_chain, usable_snapshot, severity, action) = match outcome.kind() {
            AutosaveDocumentOutcomeKind::Failed {
                stage,
                error_chain,
                usable_snapshot,
                ..
            } => (
                stage,
                error_chain,
                usable_snapshot.as_ref(),
                LogSeverity::Error,
                "failed",
            ),
            AutosaveDocumentOutcomeKind::Cancelled {
                stage, error_chain, ..
            } => (
                stage,
                error_chain,
                None,
                LogSeverity::Warning,
                "was cancelled",
            ),
            AutosaveDocumentOutcomeKind::Saved { .. } => return,
        };
        let detail = error_chain
            .first()
            .map(String::as_str)
            .unwrap_or("autosave ended without an error message");
        let snapshot = usable_snapshot
            .as_ref()
            .map(|path| format!("; usable snapshot at {}", path.display()))
            .unwrap_or_default();
        let message = format!(
            "Autosave for {} ({}) {} at {}: {}{}",
            outcome.document().as_str(),
            outcome.source_path().as_path().display(),
            action,
            stage,
            detail,
            snapshot,
        );
        let jump = LogJump::asset(outcome.source_path().as_path().display().to_string()).ok();
        let entry = LogEntry::new(LogSource::editor(), severity, &message, 0, jump).or_else(|_| {
            LogEntry::new(
                LogSource::editor(),
                severity,
                "Autosave diagnostic exceeded the editor log-entry limit.",
                0,
                None,
            )
        });
        if let Ok(entry) = entry {
            let _ = self.editor_manager.context().logs().emit(entry);
        }
        self.set_status_line(message);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn autosave_admission_is_fenced_while_project_recovery_remains_active() {
        let source = include_str!("autosave.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("autosave source should contain its production section");
        let recovery_gate = production
            .find("self.editor_manager.project_recovery_is_active()")
            .expect("autosave must read the manager recovery lifecycle");
        let poll = production
            .find(".autosave()\n            .poll_project")
            .expect("autosave service polling should remain after the recovery gate");

        assert!(recovery_gate < poll);
    }
}

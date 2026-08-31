use crate::core::asset::SaveDirtyViewOutcomeStatus;
use crate::core::extension::SaveReason;
use crate::ui::host::{DirtyDocumentSaveOwner, DirtyDocumentSaveStart};

use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn request_document_save_all(&mut self) {
        if self.pending_document_save_all || self.queued_document_save_all {
            self.set_status_line("Document save is already in progress.".to_string());
            return;
        }
        if self.pending_close_prompt.is_some() {
            self.set_status_line("Resolve the active close decision before Save All.".to_string());
            return;
        }
        self.try_start_document_save_all();
    }

    fn try_start_document_save_all(&mut self) {
        let documents = match self.editor_manager.dirty_document_toolkits() {
            Ok(documents) => documents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        match self.editor_manager.begin_dirty_document_save(
            DirtyDocumentSaveOwner::SaveAll,
            documents.into_iter().map(|document| document.document_id),
            SaveReason::SaveAll,
        ) {
            Ok(DirtyDocumentSaveStart::NoDirtyDocuments) => {
                self.set_status_line("No dirty documents to save.".to_string());
            }
            Ok(DirtyDocumentSaveStart::Scheduled) => {
                self.pending_document_save_all = true;
                self.set_status_line("Saving all dirty documents...".to_string());
            }
            Ok(DirtyDocumentSaveStart::Busy { owner }) => {
                self.queued_document_save_all = true;
                self.set_status_line(format!("Save All is waiting for {owner} to finish."));
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(in crate::ui::retained_host::app) fn poll_document_save_all(&mut self) {
        if self.pending_document_save_all {
            let result = match self
                .editor_manager
                .poll_dirty_document_save(DirtyDocumentSaveOwner::SaveAll)
            {
                Ok(Some(result)) => result,
                Ok(None) => return,
                Err(error) => {
                    self.pending_document_save_all = false;
                    self.set_status_line(format!("Documents could not be saved: {error}"));
                    return;
                }
            };
            self.pending_document_save_all = false;
            let saved = result
                .outcomes()
                .iter()
                .filter(|outcome| {
                    matches!(outcome.status(), SaveDirtyViewOutcomeStatus::Saved { .. })
                })
                .count();
            let remaining = result.outcomes().len().saturating_sub(saved);
            if remaining == 0 {
                self.set_status_line(format!("Saved {saved} document(s)."));
            } else {
                self.set_status_line(format!(
                    "Saved {saved} document(s); {remaining} document(s) require attention."
                ));
            }
        }

        if self.queued_document_save_all
            && self.editor_manager.dirty_document_save_owner().is_none()
        {
            self.queued_document_save_all = false;
            self.try_start_document_save_all();
        }
    }
}

use super::super::super::{close_prompt, RetainedEditorHost};
use crate::core::extension::SaveReason;
use crate::ui::host::{DirtyDocumentSaveOwner, DirtyDocumentSaveStart};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn close_prompt_action_clicked(
        &mut self,
        action_id: &str,
    ) {
        let Some(action) = close_prompt::close_action_id(action_id) else {
            return;
        };
        let Some(prompt) = self.pending_close_prompt.clone() else {
            return;
        };
        if prompt.save_in_flight()
            || self.pending_document_save_all
            || self.queued_document_save_all
        {
            return;
        }
        if let Some(owner) = self.editor_manager.dirty_document_save_owner() {
            self.set_status_line(format!("Close is waiting for {owner} to finish saving."));
            return;
        }
        match action {
            "cancel" => {
                self.clear_close_prompt(&prompt.target);
                self.pending_close_prompt = None;
            }
            "discard" => {
                self.discard_prompted_close(prompt);
            }
            "save" => {
                if prompt.has_dirty_project_scene() {
                    if let Err(error) = self.save_project_scene() {
                        self.set_status_line(format!("Project scene could not be saved: {error}"));
                        self.show_close_prompt(&prompt);
                        return;
                    }
                }
                match self.editor_manager.begin_dirty_document_save(
                    DirtyDocumentSaveOwner::ClosePrompt,
                    prompt.dirty_views.iter().map(|view| view.document_id),
                    SaveReason::Close,
                ) {
                    Ok(DirtyDocumentSaveStart::NoDirtyDocuments) => {
                        self.reconcile_prompted_close_after_save(
                            prompt,
                            "All documents are already saved.",
                        );
                    }
                    Ok(DirtyDocumentSaveStart::Scheduled) => {
                        let mut saving_prompt = prompt;
                        saving_prompt.begin_save();
                        self.show_close_prompt(&saving_prompt);
                        self.pending_close_prompt = Some(saving_prompt);
                        self.set_status_line("Saving document changes...".to_string());
                    }
                    Ok(DirtyDocumentSaveStart::Busy { owner }) => {
                        self.set_status_line(format!(
                            "Close is waiting for {owner} to finish saving."
                        ));
                        self.show_close_prompt(&prompt);
                    }
                    Err(error) => {
                        self.set_status_line(error.to_string());
                        self.show_close_prompt(&prompt);
                    }
                }
            }
            _ => {}
        }
    }
}

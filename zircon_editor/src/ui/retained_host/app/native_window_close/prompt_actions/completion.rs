use super::super::super::{
    close_prompt::{self, ClosePromptTarget, PendingClosePrompt},
    RetainedEditorHost,
};
use crate::ui::host::DirtyDocumentSaveOwner;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::native_window_close) fn finish_prompted_close(
        &mut self,
        prompt: PendingClosePrompt,
    ) {
        self.pending_close_prompt = None;
        match prompt.target {
            ClosePromptTarget::Project => {
                if let Err(error) = self.commit_project_close() {
                    self.set_status_line(error.to_string());
                }
            }
            ClosePromptTarget::MainWindow => self.ui.request_exit(),
            ClosePromptTarget::FloatingWindow(window_id) => {
                let _ =
                    self.close_floating_window_without_prompt(&window_id, prompt.close_instances);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn discard_prompted_close(
        &mut self,
        mut prompt: PendingClosePrompt,
    ) {
        let dirty_views = match self.dirty_views_for_prompt(&prompt) {
            Ok(dirty_views) => dirty_views,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let dirty_project_scene_generation = match self.dirty_project_scene_generation() {
            Ok(generation) => generation,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        if !prompt.permits_discard(&dirty_views, dirty_project_scene_generation) {
            prompt.finish_save(dirty_views, dirty_project_scene_generation);
            self.show_close_prompt(&prompt);
            self.pending_close_prompt = Some(prompt);
            self.set_status_line(
                "Document changes were updated while closing; review the refreshed decision."
                    .to_string(),
            );
            return;
        }
        self.clear_close_prompt(&prompt.target);
        self.finish_prompted_close(prompt);
    }

    pub(in crate::ui::retained_host::app) fn poll_prompted_close_save(&mut self) {
        let Some(prompt) = self.pending_close_prompt.clone() else {
            return;
        };
        if !prompt.save_in_flight() {
            return;
        }
        let result = match self
            .editor_manager
            .poll_dirty_document_save(DirtyDocumentSaveOwner::ClosePrompt)
        {
            Ok(Some(result)) => result,
            Ok(None) => return,
            Err(error) => {
                let status = format!("Documents could not be saved: {error}");
                self.reconcile_prompted_close_after_save(prompt, &status);
                return;
            }
        };
        let status = if result.all_saved() {
            "Save batch completed; checking for newer changes."
        } else {
            "Some documents could not be saved; resolve them, Discard, or Cancel."
        };
        self.reconcile_prompted_close_after_save(prompt, status);
    }

    pub(in crate::ui::retained_host::app) fn reconcile_prompted_close_after_save(
        &mut self,
        mut prompt: PendingClosePrompt,
        status: &str,
    ) {
        let dirty_views = match self.dirty_views_for_prompt(&prompt) {
            Ok(dirty_views) => dirty_views,
            Err(error) => {
                let dirty_project_scene_generation =
                    self.dirty_project_scene_generation().ok().flatten();
                prompt.finish_save(prompt.dirty_views.clone(), dirty_project_scene_generation);
                self.show_close_prompt(&prompt);
                self.pending_close_prompt = Some(prompt);
                self.set_status_line(error.to_string());
                return;
            }
        };
        let dirty_project_scene_generation = match self.dirty_project_scene_generation() {
            Ok(generation) => generation,
            Err(error) => {
                prompt.finish_save(prompt.dirty_views.clone(), None);
                self.show_close_prompt(&prompt);
                self.pending_close_prompt = Some(prompt);
                self.set_status_line(error);
                return;
            }
        };
        if dirty_views.is_empty() && dirty_project_scene_generation.is_none() {
            self.clear_close_prompt(&prompt.target);
            self.finish_prompted_close(prompt);
            return;
        }
        prompt.finish_save(dirty_views, dirty_project_scene_generation);
        self.show_close_prompt(&prompt);
        self.pending_close_prompt = Some(prompt);
        self.set_status_line(status.to_string());
    }

    fn dirty_views_for_prompt(
        &self,
        prompt: &PendingClosePrompt,
    ) -> Result<Vec<close_prompt::DirtyCloseView>, crate::ui::host::EditorError> {
        let dirty_documents = self.editor_manager.dirty_document_toolkits()?;
        Ok(match &prompt.target {
            ClosePromptTarget::Project | ClosePromptTarget::MainWindow => {
                close_prompt::all_dirty_close_views(&dirty_documents)
            }
            ClosePromptTarget::FloatingWindow(_) => {
                close_prompt::dirty_close_views(&dirty_documents, prompt.close_instances.clone())
            }
        })
    }
}

use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::layout::MainPageId;

use super::close_prompt::ClosePromptTarget;
use super::hierarchy_pointer::HierarchyTerminalReason;
use super::*;

mod floating_window;
mod prompt_actions;

impl RetainedEditorHost {
    pub(super) fn native_main_window_close_requested(&mut self) -> CloseRequestResponse {
        self.retire_hierarchy_drag_with_reason(HierarchyTerminalReason::WindowClose);
        if self.document_save_blocks_native_close() {
            return CloseRequestResponse::KeepWindowShown;
        }
        self.recompute_if_dirty();
        let dirty_documents = match self
            .editor_manager
            .enter()
            .map_err(crate::ui::host::EditorError::from)
            .and_then(|manager| manager.dirty_document_toolkits())
        {
            Ok(documents) => documents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return CloseRequestResponse::KeepWindowShown;
            }
        };
        let dirty = close_prompt::all_dirty_close_views(&dirty_documents);
        let dirty_project_scene_token = match self.dirty_project_scene_token() {
            Ok(token) => token,
            Err(error) => {
                self.set_status_line(error);
                return CloseRequestResponse::KeepWindowShown;
            }
        };
        if !dirty.is_empty() || dirty_project_scene_token.is_some() {
            let close_instances = match self.runtime.current_view_instance_ids() {
                Ok(instance_ids) => instance_ids,
                Err(error) => {
                    self.set_status_line(error.to_string());
                    return CloseRequestResponse::KeepWindowShown;
                }
            };
            let mut prompt = super::close_prompt::PendingClosePrompt::new(
                ClosePromptTarget::MainWindow,
                close_instances,
                dirty,
            );
            if let Some(token) = dirty_project_scene_token {
                prompt = prompt.with_dirty_project_scene(token);
            }
            self.begin_close_prompt_plan(prompt);
            return CloseRequestResponse::KeepWindowShown;
        }
        CloseRequestResponse::HideWindow
    }

    pub(super) fn native_floating_window_close_requested(
        &mut self,
        window_id: &MainPageId,
    ) -> CloseRequestResponse {
        self.cancel_hierarchy_drag_for_window(
            &Some(window_id.clone()),
            HierarchyTerminalReason::WindowClose,
        );
        if self.document_save_blocks_native_close() {
            return CloseRequestResponse::KeepWindowShown;
        }
        self.recompute_if_dirty();
        let instance_ids = match self.floating_window_close_instance_ids(window_id) {
            Ok(Some(instance_ids)) => instance_ids,
            Ok(None) => return CloseRequestResponse::KeepWindowShown,
            Err(error) => {
                self.set_status_line(error.to_string());
                return CloseRequestResponse::KeepWindowShown;
            }
        };

        let dirty_documents = match self
            .editor_manager
            .enter()
            .map_err(crate::ui::host::EditorError::from)
            .and_then(|manager| manager.dirty_document_toolkits())
        {
            Ok(documents) => documents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return CloseRequestResponse::KeepWindowShown;
            }
        };
        let dirty = close_prompt::dirty_close_views(&dirty_documents, instance_ids.clone());
        if !dirty.is_empty() {
            self.begin_close_prompt(
                ClosePromptTarget::FloatingWindow(window_id.clone()),
                instance_ids,
                dirty,
            );
            return CloseRequestResponse::KeepWindowShown;
        }

        self.close_floating_window_without_prompt(window_id, instance_ids)
    }

    fn document_save_blocks_native_close(&mut self) -> bool {
        if let Some(prompt) = self.pending_close_prompt.clone() {
            self.show_close_prompt(&prompt);
            self.set_status_line(if prompt.has_failed_save() {
                "The previous save failed; resolve the close decision, Discard, or Cancel."
                    .to_string()
            } else {
                "A close decision is already active.".to_string()
            });
            return true;
        }
        let save_owner = match self
            .editor_manager
            .enter()
            .map_err(crate::ui::host::EditorError::from)
            .map(|manager| manager.dirty_document_save_owner())
        {
            Ok(owner) => owner,
            Err(error) => {
                self.set_status_line(error.to_string());
                return true;
            }
        };
        if let Some(owner) = save_owner {
            self.set_status_line(format!("Close is waiting for {owner} to finish saving."));
            return true;
        }
        if self.queued_document_save_all {
            self.set_status_line("Close is waiting for queued Save All.".to_string());
            return true;
        }
        false
    }
}

#[cfg(test)]
#[path = "tests/native_window_close_performance_tests.rs"]
mod performance_tests;

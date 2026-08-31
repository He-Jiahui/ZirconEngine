use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::layout::MainPageId;

use super::close_prompt::ClosePromptTarget;
use super::*;

mod floating_window;
mod prompt_actions;

impl RetainedEditorHost {
    pub(super) fn native_main_window_close_requested(&mut self) -> CloseRequestResponse {
        if self.document_save_blocks_native_close() {
            return CloseRequestResponse::KeepWindowShown;
        }
        self.recompute_if_dirty();
        let instances = self.runtime.current_view_instances();
        let dirty_documents = match self.editor_manager.dirty_document_toolkits() {
            Ok(documents) => documents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return CloseRequestResponse::KeepWindowShown;
            }
        };
        let dirty = close_prompt::all_dirty_close_views(&dirty_documents);
        let dirty_project_scene_generation = match self.dirty_project_scene_generation() {
            Ok(generation) => generation,
            Err(error) => {
                self.set_status_line(error);
                return CloseRequestResponse::KeepWindowShown;
            }
        };
        if !dirty.is_empty() || dirty_project_scene_generation.is_some() {
            let close_instances = instances
                .into_iter()
                .map(|instance| instance.instance_id)
                .collect();
            let mut prompt = super::close_prompt::PendingClosePrompt::new(
                ClosePromptTarget::MainWindow,
                close_instances,
                dirty,
            );
            if let Some(generation) = dirty_project_scene_generation {
                prompt = prompt.with_dirty_project_scene(generation);
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
        if self.document_save_blocks_native_close() {
            return CloseRequestResponse::KeepWindowShown;
        }
        self.recompute_if_dirty();
        let Some(instance_ids) = self.floating_window_close_instance_ids(window_id) else {
            return CloseRequestResponse::KeepWindowShown;
        };

        let dirty_documents = match self.editor_manager.dirty_document_toolkits() {
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
        if let Some(owner) = self.editor_manager.dirty_document_save_owner() {
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
mod performance_tests {
    #[test]
    fn main_window_close_reuses_the_view_instance_snapshot() {
        let source = include_str!("native_window_close.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        let snapshot_call = ["current_view_", "instances()"].concat();

        assert_eq!(production.matches(&snapshot_call).count(), 1);
        assert!(production.contains("all_dirty_close_views(&dirty_documents)"));
    }
}

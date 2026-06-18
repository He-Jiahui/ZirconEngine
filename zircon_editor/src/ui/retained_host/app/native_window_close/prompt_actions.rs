use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    close_prompt::{self, ClosePromptTarget, PendingClosePrompt},
    RetainedEditorHost, UiHostWindow,
};

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
        match action {
            "cancel" => {
                self.clear_close_prompt(&prompt.target);
                self.pending_close_prompt = None;
            }
            "discard" => {
                self.clear_close_prompt(&prompt.target);
                self.finish_prompted_close(prompt);
            }
            "save" => {
                if let Err(error) = self.save_dirty_prompt_views(&prompt.dirty_views) {
                    self.set_status_line(error);
                    self.show_close_prompt(&prompt);
                    return;
                }
                self.clear_close_prompt(&prompt.target);
                self.finish_prompted_close(prompt);
            }
            _ => {}
        }
    }

    pub(super) fn begin_close_prompt(
        &mut self,
        target: ClosePromptTarget,
        close_instances: Vec<ViewInstanceId>,
        dirty_views: Vec<close_prompt::DirtyCloseView>,
    ) {
        let prompt = PendingClosePrompt::new(target, close_instances, dirty_views);
        self.show_close_prompt(&prompt);
        self.pending_close_prompt = Some(prompt);
    }

    fn show_close_prompt(&self, prompt: &PendingClosePrompt) {
        let ui = self.close_prompt_ui(&prompt.target);
        close_prompt::show_prompt(&ui, prompt);
    }

    fn clear_close_prompt(&self, target: &ClosePromptTarget) {
        let ui = self.close_prompt_ui(target);
        close_prompt::clear_prompt(&ui);
    }

    fn close_prompt_ui(&self, target: &ClosePromptTarget) -> UiHostWindow {
        match target {
            ClosePromptTarget::MainWindow => self.ui.clone_strong(),
            ClosePromptTarget::FloatingWindow(window_id) => self
                .native_window_presenters
                .window(window_id)
                .unwrap_or_else(|| self.ui.clone_strong()),
        }
    }

    fn finish_prompted_close(&mut self, prompt: PendingClosePrompt) {
        self.pending_close_prompt = None;
        match prompt.target {
            ClosePromptTarget::MainWindow => self.ui.request_exit(),
            ClosePromptTarget::FloatingWindow(window_id) => {
                let _ =
                    self.close_floating_window_without_prompt(&window_id, prompt.close_instances);
            }
        }
    }

    fn save_dirty_prompt_views(
        &self,
        views: &[close_prompt::DirtyCloseView],
    ) -> Result<(), String> {
        for view in views {
            match view.descriptor_id.0.as_str() {
                "editor.ui_asset" => {
                    self.editor_manager
                        .save_ui_asset_editor(&view.instance_id)
                        .map_err(|error| error.to_string())?;
                }
                "editor.animation_sequence" | "editor.animation_graph" => {
                    self.editor_manager
                        .save_animation_editor(&view.instance_id)
                        .map_err(|error| error.to_string())?;
                }
                _ => {
                    return Err(format!(
                        "Cannot save {} automatically; use Discard or Cancel",
                        view.title
                    ));
                }
            }
        }
        Ok(())
    }
}

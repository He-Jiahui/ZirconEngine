use crate::ui::workbench::view::ViewInstanceId;

use super::super::super::{
    RetainedEditorHost, UiHostWindow,
    close_prompt::{self, ClosePromptTarget, PendingClosePrompt},
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::native_window_close) fn begin_close_prompt(
        &mut self,
        target: ClosePromptTarget,
        close_instances: Vec<ViewInstanceId>,
        dirty_views: Vec<close_prompt::DirtyCloseView>,
    ) {
        let prompt = PendingClosePrompt::new(target, close_instances, dirty_views);
        self.show_close_prompt(&prompt);
        self.pending_close_prompt = Some(prompt);
    }

    pub(in crate::ui::retained_host::app::native_window_close) fn show_close_prompt(
        &self,
        prompt: &PendingClosePrompt,
    ) {
        let ui = self.close_prompt_ui(&prompt.target);
        close_prompt::show_prompt(&ui, prompt);
    }

    pub(in crate::ui::retained_host::app::native_window_close) fn clear_close_prompt(
        &self,
        target: &ClosePromptTarget,
    ) {
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
}

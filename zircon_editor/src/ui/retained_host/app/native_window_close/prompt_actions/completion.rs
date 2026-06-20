use super::super::super::{
    close_prompt::{ClosePromptTarget, PendingClosePrompt},
    RetainedEditorHost,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::native_window_close) fn finish_prompted_close(
        &mut self,
        prompt: PendingClosePrompt,
    ) {
        self.pending_close_prompt = None;
        match prompt.target {
            ClosePromptTarget::MainWindow => self.ui.request_exit(),
            ClosePromptTarget::FloatingWindow(window_id) => {
                let _ =
                    self.close_floating_window_without_prompt(&window_id, prompt.close_instances);
            }
        }
    }
}

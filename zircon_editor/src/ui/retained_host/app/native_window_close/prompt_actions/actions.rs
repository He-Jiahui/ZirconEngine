use super::super::super::{RetainedEditorHost, close_prompt};

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
}

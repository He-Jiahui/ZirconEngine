use super::super::super::data::HostClosePromptData;
use super::super::super::redraw::HostRedrawRequest;
use super::super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_close_prompt(&self, prompt: HostClosePromptData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let close_prompt = &state.host_presentation.close_prompt;
            let damage = if close_prompt.visible {
                close_prompt.overlay_frame.clone()
            } else {
                prompt.overlay_frame.clone()
            };
            state.replace_close_prompt(prompt);
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_close_prompt(&self) {
        self.set_close_prompt(HostClosePromptData::default());
    }
}

#[cfg(test)]
#[path = "tests/close_prompt_performance_tests.rs"]
mod performance_tests;

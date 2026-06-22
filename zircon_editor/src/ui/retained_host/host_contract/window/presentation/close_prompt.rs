use super::super::super::data::HostClosePromptData;
use super::super::super::redraw::HostRedrawRequest;
use super::super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_close_prompt(&self, prompt: HostClosePromptData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let current = state.host_presentation.close_prompt.clone();
            let damage = if current.visible {
                current.overlay_frame
            } else {
                prompt.overlay_frame.clone()
            };
            state.host_presentation.close_prompt = prompt;
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_close_prompt(&self) {
        self.set_close_prompt(HostClosePromptData::default());
    }
}

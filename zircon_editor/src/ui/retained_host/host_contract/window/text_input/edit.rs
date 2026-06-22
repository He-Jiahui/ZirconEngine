mod dispatch;
mod redraw;

use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use dispatch::dispatch_text_focus_value;

impl UiHostWindow {
    pub(crate) fn text_input_focus_active(&self) -> bool {
        self.state.borrow().text_input_focus.is_active()
    }

    pub(crate) fn dispatch_focused_text_insert(&self, text: &str) -> NativePointerDispatchResult {
        let text: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if text.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            value.push_str(&text);
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        dispatch_text_focus_value(self, focus.clone(), focus.edit_target_id(), value)
    }

    pub(crate) fn dispatch_focused_text_backspace(&self) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            if value.pop().is_none() {
                return NativePointerDispatchResult::idle();
            }
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        dispatch_text_focus_value(self, focus.clone(), focus.edit_target_id(), value)
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_text_commit(
        &self,
    ) -> NativePointerDispatchResult {
        let focus = self.state.borrow().text_input_focus.clone();
        if !focus.is_active() || focus.commit_action_id.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        dispatch_text_focus_value(
            self,
            focus.clone(),
            focus.commit_target_id(),
            focus.value_text.to_string(),
        )
    }
}

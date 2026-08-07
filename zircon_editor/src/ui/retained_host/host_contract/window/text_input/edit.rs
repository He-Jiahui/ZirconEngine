mod dispatch;
mod redraw;

use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::primitives::SharedString;
use dispatch::dispatch_text_focus_value;

impl UiHostWindow {
    pub(crate) fn text_input_focus_active(&self) -> bool {
        self.state.borrow().text_input_focus.is_active()
    }

    pub(crate) fn dispatch_focused_text_insert(&self, text: &str) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.as_ref().clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            let previous_len = value.len();
            value.extend(text.chars().filter(|ch| !ch.is_control()));
            if value.len() == previous_len {
                return NativePointerDispatchResult::idle();
            }
            let value: SharedString = value.into();
            let mut next_focus = focus.clone();
            next_focus.value_text = value.clone();
            state.replace_text_input_focus(next_focus);
            (focus, value)
        };
        let target_id = focus.edit_target_id();
        dispatch_text_focus_value(self, focus, target_id, value)
    }

    pub(crate) fn dispatch_focused_text_backspace(&self) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.as_ref().clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            if value.pop().is_none() {
                return NativePointerDispatchResult::idle();
            }
            let value: SharedString = value.into();
            let mut next_focus = focus.clone();
            next_focus.value_text = value.clone();
            state.replace_text_input_focus(next_focus);
            (focus, value)
        };
        let target_id = focus.edit_target_id();
        dispatch_text_focus_value(self, focus, target_id, value)
    }

    pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_text_commit(
        &self,
    ) -> NativePointerDispatchResult {
        let focus = self.state.borrow().text_input_focus.as_ref().clone();
        if !focus.is_active() || focus.commit_action_id.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        let target_id = focus.commit_target_id();
        let value = focus.value_text.clone();
        dispatch_text_focus_value(self, focus, target_id, value)
    }
}
